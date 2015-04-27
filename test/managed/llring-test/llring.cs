#pragma warning disable 0420 // Volatile ref to interlocked is OK
/* A pure C# implementation of llring
 *
 */

using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Diagnostics.Contracts;
using System.Runtime.ConstrainedExecution;
using System.Runtime.InteropServices;
using System.Runtime.Serialization;
using System.Security;
using System.Security.Permissions;
using System.Threading;
using System.Collections.Concurrent;

namespace E2D2.Collections.Concurrent
{
    // A fixed size thread-safe ring buffer
    // TODO: Extend IProducerConsumer
    public class LLRing<T> {
        private const int CACHELINE_SIZE = 64;
        public const UInt32 RING_QUOT_EXCEED = 1u << 31;
        #if (!__MonoCS__)
        [StructLayout(LayoutKind.Sequential, Pack = CACHELINE_SIZE)]
        private struct Common {
        #else
        [StructLayout(LayoutKind.Sequential)]
        private unsafe struct Common {
        #endif
            public UInt32 slots;
            // Padding to fake the Pack parameter
            #if (__MonoCS__)
            fixed byte _pad0[CACHELINE_SIZE];
            #endif
            public UInt32 mask;
            #if (__MonoCS__)
            fixed byte _pad1[CACHELINE_SIZE];
            #endif
            public UInt32 watermark;
            #if (__MonoCS__)
            fixed byte _pad2[CACHELINE_SIZE];
            #endif
            public bool sp_enqueue;
            #if (__MonoCS__)
            fixed byte _pad3[CACHELINE_SIZE];
            #endif
            public bool sc_dequeue;
        }
        #if (!__MonoCS__)
        [StructLayout(LayoutKind.Sequential, Pack = CACHELINE_SIZE)]
        private struct Producer {
        #else
        [StructLayout(LayoutKind.Sequential)]
        private unsafe struct Producer {
        #endif
            public volatile UInt32 head; // The head marks the maximum reserved by any 
                                         // producer.
            #if (__MonoCS__)
            fixed byte _pad0[CACHELINE_SIZE];
            #endif
            public volatile UInt32 tail; // The tail marks the maximum committed by any producer.
        }
        #if (!__MonoCS__)
        [StructLayout(LayoutKind.Sequential, Pack = CACHELINE_SIZE)]
        private struct Consumer {
        #else
        [StructLayout(LayoutKind.Sequential)]
        private unsafe struct Consumer {
        #endif
            public volatile UInt32 head; // Similarly, the head marks the maximum reserved to be read by any consumer.
            #if (__MonoCS__)
            fixed byte _pad0[CACHELINE_SIZE];
            #endif
            public volatile UInt32 tail; // Tail marks what has been committed.
        }
        private Common common;
        private Producer prod;
        private Consumer cons;
        // TODO: Can make this happen with unsafe, but should we, in particular
        // ring is not guaranteed to be even in the same place.
        //private fixed byte padding[CACHELINE_SIZE];
        private T[] ring;
        
        /// Construct
        public LLRing(uint slots, bool sp, bool sc) {
            Contract.Requires((slots & (slots - 1)) == 0,
                              "LLRing only supports power of 2 lengths");
            common.slots = slots;
            common.mask = slots - 1;
            common.watermark = slots;
            common.sp_enqueue = sp;
            common.sc_dequeue = sc;
            // Actually unnecessary, but helps with readability
            prod.head = 0;
            prod.tail = 0;
            cons.head = 0;
            cons.tail = 0;
            ring = new T[slots];
        }

        public UInt32 EnqueueBatch (ref T[] objects) {
            if(common.sp_enqueue) {
                return SingleProducerEnqueue(ref objects);
            } else {
                return MultiProducerEnqueue(ref objects);
            }
        }

        public UInt32 DequeueBatch (ref T[] array) {
            if(common.sc_dequeue) {
                return SingleConsumerDequeue(ref array);
            } else {
                return MultiConsumerDequeue(ref array);
            }
        }
        // TODO: Currently I am just implementing LLRING_QUEUE_VARIABLE, i.e., queue as many
        // as possible. It is not hard to implement the other one, but laziness.
        private UInt32 SingleProducerEnqueue(ref T[] objects) {
            // We don't care about no overflows
            unchecked {
                UInt32 phead = prod.head;
                UInt32 ctail = cons.tail;
                UInt32 mask = common.mask;
                UInt32 n = (UInt32)objects.Length;
                UInt32 free = 0;
            
                // The idea here is that phead and ctail cannot be separated by more than slot (since
                // we would never allow inserts in that way). As a result this can only return between
                // 0 and slots - 1
                free = mask + ctail - phead;
                if (free == 0) {
                    return 0;
                }
                n = Math.Min(free, n);
                UInt32 pnext = phead + n;

                // Single producer, don't need to do anything interesting here (no more than one writer, etc.)
                prod.head = pnext;
                UInt32 slots = common.slots;
                UInt32 idx = phead & mask;
                if ((idx + n) < slots) {
                    int i;
                    //TODO: For the C code this loop is unrolled, seems a bit premature to do this now.
                    for (i = 0; i < (n & (~3u)); i+=4, idx+=4) {
                        ring[idx] = objects[i];
                        ring[idx + 1] = objects[i + 1];
                        ring[idx + 2] = objects[i + 2];
                        ring[idx + 3] = objects[i + 3];
                    }
                    switch(n & 3) {
                      case 3: ring[idx++] = objects[i++]; goto case 2;
                      case 2: ring[idx++] = objects[i++]; goto case 1;
                      case 1: ring[idx++] = objects[i++]; break;
                    }
                } else {
                    int i;
                    for (i = 0; idx < slots; i++, idx++) {
                        ring[idx] = objects[i];
                    }
                    for (idx = 0; i < n; i++, idx++) {
                        ring[idx] = objects[i];
                    }
                }

                if (((mask + 1) - free + n) > common.watermark) {
                    n = n | RING_QUOT_EXCEED;
                }
                prod.tail = pnext;
                return n;
            }
        }

        // CLS compliance dictates that Interlocked.CompareExchange in C# only handles signed variables
        // (yes there were stupid decisions). As a result need to build one that does unsigned for this class.
        unsafe UInt32 CompareExchange(ref UInt32 target, UInt32 val, UInt32 cmp) {
            fixed(UInt32* p = &target) {
                return (UInt32)Interlocked.CompareExchange(ref *(int*)p, (int)val, (int)cmp);
            }
        }

        private UInt32 MultiProducerEnqueue(ref T[] objects) {
            // We don't care about no overflows
            unchecked {
                UInt32 phead = prod.head;
                UInt32 ctail = cons.tail;
                UInt32 mask = common.mask;
                UInt32 max = (UInt32)objects.Length;
                UInt32 n = max;
                UInt32 free = 0;
                UInt32 pnext = 0;

                // Same operation as above, but atomically.
                do {
                    phead = prod.head;
                    ctail = cons.tail;
                    n = max;
                    // The idea here is that phead and ctail cannot be separated by more than slot (since
                    // we would never allow inserts in that way). As a result this can only return between
                    // 0 and slots - 1
                    free = mask + ctail - phead;
                    if (free == 0) {
                        return 0;
                    }
                    n = Math.Min(free, n);
                    pnext = phead + n;

                } while (CompareExchange(ref prod.head, pnext, phead) != phead);

                UInt32 slots = common.slots;
                UInt32 idx = phead & mask;
                if ((idx + n) < slots) {
                    int i;
                    //TODO: For the C code this loop is unrolled, seems a bit premature to do this now.
                    for (i = 0; i < (n & (~3u)); i+=4, idx+=4) {
                        ring[idx] = objects[i];
                        ring[idx + 1] = objects[i + 1];
                        ring[idx + 2] = objects[i + 2];
                        ring[idx + 3] = objects[i + 3];
                    }
                    switch(n & 3) {
                      case 3: ring[idx++] = objects[i++]; goto case 2;
                      case 2: ring[idx++] = objects[i++]; goto case 1;
                      case 1: ring[idx++] = objects[i++]; break;
                    }
                } else {
                    int i;
                    for (i = 0; idx < slots; i++, idx++) {
                        ring[idx] = objects[i];
                    }
                    for (idx = 0; i < n; i++, idx++) {
                        ring[idx] = objects[i];
                    }
                }

                if (((mask + 1) - free + n) > common.watermark) {
                    n = n | RING_QUOT_EXCEED;
                }
                // If there is someone else who is simultaneously enqueueing (and started before us)
                // they would have set prod.head but not prod.tail. We should wait until the enqueue
                // before us has completed before setting.
                // Allocating a spin wait here to account for producers on different cores.
                SpinWait spin = new SpinWait();
                while (prod.tail != phead) {
                    spin.SpinOnce();
                }
                prod.tail = pnext;
                return n;
            }
        }

        private UInt32 SingleConsumerDequeue(ref T[] array) {
            unchecked {
                UInt32 chead = cons.head;
                UInt32 ptail = prod.tail;
                UInt32 mask = common.mask;
                UInt32 entries = ptail - chead; 
                UInt32 n = (UInt32)array.Length;
                if (entries == 0) {
                    return 0;
                }
                n = Math.Min(n, entries);
                UInt32 cnext = chead + n;
                cons.head = cnext;
                UInt32 idx = chead & mask;
                UInt32 slots = common.slots;
                if (idx + n < slots) {
                    int i = 0;
                    for (i = 0; i < (n & (~3u)); i+=4, idx+=4) {
                        array[i] = ring[idx];
                        array[i + 1] = ring[idx + 1];
                        array[i + 2] = ring[idx + 2];
                        array[i + 3] = ring[idx + 3];
                    }
                    switch(n & 3) {
                      case 3: array[i++] = ring[idx++]; goto case 2;
                      case 2: array[i++] = ring[idx++]; goto case 1;
                      case 1: array[i++] = ring[idx++]; break;
                    }
                } else {
                    int i;
                    for (i = 0; idx < slots; i++, idx++) {
                        array[i] = ring[idx];
                    }
                    for (idx = 0; i < n; i++, idx++) {
                        array[i] = ring[idx];
                    }
                }
                cons.tail = cnext;
                return n;
            }
        }

        private UInt32 MultiConsumerDequeue(ref T[] array) {
            unchecked {
                UInt32 chead = 0;
                UInt32 ptail = 0;
                UInt32 mask = common.mask;
                UInt32 entries = 0; 
                UInt32 cnext = 0;
                UInt32 n = 0;
                UInt32 max = (UInt32)array.Length;
                // Just atomically increment cons.head instead of doing it as above.
                do {
                    n = max;
                    chead = cons.head;
                    ptail = prod.tail;
                    entries = ptail - chead;
                    if (entries == 0) {
                        return 0; // Short circuit, nothing to read
                    }
                    n = Math.Min(n, entries);
                    cnext = chead + n;
                } while (CompareExchange(ref cons.head, cnext, chead) != chead);

                UInt32 idx = chead & mask;
                UInt32 slots = common.slots;
                if (idx + n < slots) {
                    int i = 0;
                    for (i = 0; i < (n & (~3u)); i+=4, idx+=4) {
                        array[i] = ring[idx];
                        array[i + 1] = ring[idx + 1];
                        array[i + 2] = ring[idx + 2];
                        array[i + 3] = ring[idx + 3];
                    }
                    switch(n & 3) {
                      case 3: array[i++] = ring[idx++]; goto case 2;
                      case 2: array[i++] = ring[idx++]; goto case 1;
                      case 1: array[i++] = ring[idx++]; break;
                    }
                } else {
                    int i;
                    for (i = 0; idx < slots; i++, idx++) {
                        array[i] = ring[idx];
                    }
                    for (idx = 0; i < n; i++, idx++) {
                        array[i] = ring[idx];
                    }
                }
                // If there is someone else who is simultaneously dequeuinq (and started before us)
                // they would have set cons.head but not cons.tail. We should wait until the dequeue
                // before us has completed before setting.
                // Allocating a spin wait here to account for producers on different cores.
                SpinWait spin = new SpinWait();
                while (cons.tail != chead) {
                    spin.SpinOnce();
                }
                cons.tail = cnext;
                return n;
            }
        }
    }
}
