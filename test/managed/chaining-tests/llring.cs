#pragma warning disable 0420 // Volatile ref to interlocked is OK
/* A pure C# implementation of llring
 *
 */
// If use this to disable fixed enqueueing 
//#define ENQUEUE_FIXED 
//#define DEQUEUE_FIXED

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
using System.Runtime.CompilerServices; 

namespace E2D2.SNApi
{
    // A fixed size thread-safe ring buffer
    // TODO: Extend IProducerConsumer
    public class LLRingPacket {
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
        private IntPtr[] ring;
        
        /// Construct
        public LLRingPacket(uint slots, bool sp, bool sc) {
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
            ring = new IntPtr[slots];
        }

        public void SetWatermark (uint count) {
            Contract.Requires((count < common.slots),
                              "Can't set watermark higher than number of slots");
            if (count == 0) {
                count = common.slots;
            }
            common.watermark = count;
        }

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal unsafe UInt32 SingleProducerEnqueuePackets(ref PacketBuffer buffer) {
			unchecked {
				IntPtr* pktPointerArray = buffer.m_pktPointerArray;
                UInt32 phead = prod.head;
                UInt32 ctail = cons.tail;
                UInt32 mask = common.mask;
                UInt32 n = (UInt32)buffer.m_available;
                UInt32 free = 0;
            
                // The idea here is that phead and ctail cannot be separated by more than slot (since
                // we would never allow inserts in that way). As a result this can only return between
                // 0 and slots - 1
                free = mask + ctail - phead;
                if (n > free) {
                #if (ENQUEUE_FIXED)
                   // Do not insert anything, just return 0 for now.
                   return 0;
                #else
                    if (free == 0) {
                        return 0;
                    }
                    n = free;
                #endif
                }

                UInt32 pnext = phead + n;

                // Single producer, don't need to do anything interesting here (no more than one writer, etc.)
                prod.head = pnext;
                UInt32 slots = common.slots;
                UInt32 idx = phead & mask;
                if ((idx + n) < slots) {
                    //Array.Copy(objects, 0, ring, idx, n);
                    int i = 0;
                    for (i = 0; i < (n & 0x3); i+=4, idx+=4) {
                    	ring[idx] = pktPointerArray[i];
                    	ring[idx + 1] = pktPointerArray[i + 1];
                    	ring[idx + 2] = pktPointerArray[i + 2];
                    	ring[idx + 3] = pktPointerArray[i + 3];
					}
					switch (idx % 4) {
						case 3: ring[idx++] = pktPointerArray[i++]; goto case 2;
						case 2: ring[idx++] = pktPointerArray[i++]; goto case 1;
						case 1: ring[idx++] = pktPointerArray[i++]; break;
					}
                }
                else {
                    int i;
                    for (i = 0; idx < slots; i++, idx++) {
                    	ring[idx] = pktPointerArray[i];
					}
					for (idx = 0; i < n; i++, idx++) {
						ring[idx] = pktPointerArray[i];
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
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        unsafe UInt32 CompareExchange(ref UInt32 target, UInt32 val, UInt32 cmp) {
            fixed(UInt32* p = &target) {
                return (UInt32)Interlocked.CompareExchange(ref *(int*)p, (int)val, (int)cmp);
            }
        }

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal unsafe UInt32 SingleConsumerDequeuePackets(ref PacketBuffer buffer) {
            unchecked {
				IntPtr* pktPointerArray = buffer.m_pktPointerArray;
                UInt32 chead = cons.head;
                UInt32 ptail = prod.tail;
                UInt32 mask = common.mask;
                UInt32 n = (UInt32)buffer.Length;
                UInt32 entries = ptail - chead; 
                if (n > entries) {
                   #if (DEQUEUE_FIXED)
                   return 0;
                   #else
                   if (entries == 0) {
                       return 0;
                   }
                   n = entries;
                   #endif
                }
                UInt32 cnext = chead + n;
                cons.head = cnext;
                UInt32 idx = chead & mask;
                UInt32 slots = common.slots;
                if ((idx + n) < slots) {
                    //Array.Copy(objects, 0, ring, idx, n);
                    int i;
                    for (i = 0; i < (n & 0x3); i+=4, idx+=4) {
                    	pktPointerArray[i] = ring[idx];
                    	pktPointerArray[i + 1] = ring[idx + 1];
                    	pktPointerArray[i + 2] = ring[idx + 2];
                    	pktPointerArray[i + 3] = ring[idx + 3];
					}
					switch (idx % 4) {
						case 3: pktPointerArray[i++] = ring[idx++]; goto case 2;
						case 2: pktPointerArray[i++] = ring[idx++]; goto case 1;
						case 1: pktPointerArray[i++] = ring[idx++]; break;
					}
                }
                else {
                    int i;
                    for (i = 0; idx < slots; i++, idx++) {
                    	pktPointerArray[i] = ring[idx];
					}
					for (idx = 0; i < n; i++, idx++) {
						pktPointerArray[i] = ring[idx];
					}
                }
                
                cons.tail = cnext;
                return n;
            }
		}
    }
}
