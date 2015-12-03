#pragma warning disable 0420 // Volatile ref to interlocked is OK
/* A pure C# implementation of llring
 *
 */
// If use this to disable fixed enqueueing 
#define ENQUEUE_FIXED 
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

namespace E2D2.Collections.Concurrent
{
    // A fixed size thread-safe ring buffer
    // TODO: Extend IProducerConsumer
    internal class LLRingExternal {
        [System.Runtime.InteropServices.DllImport("LLRingDll.dll", CallingConvention = CallingConvention.Cdecl)]
        internal static extern UIntPtr llring_alloc_and_init(uint slots, int sp, int sc);

        [System.Runtime.InteropServices.DllImport("LLRingDll.dll", CallingConvention = CallingConvention.Cdecl)]
        internal static extern int free_ring(UIntPtr ring);

        [System.Runtime.InteropServices.DllImport("LLRingDll.dll", CallingConvention = CallingConvention.Cdecl)]
        internal static extern int llring_enqueue_bulk(UIntPtr ring, [In][MarshalAs(UnmanagedType.LPArray)]UIntPtr[] objects, uint length);

        [System.Runtime.InteropServices.DllImport("LLRingDll.dll", CallingConvention = CallingConvention.Cdecl)]
        internal static extern int llring_dequeue_burst(UIntPtr ring, [In,Out][MarshalAs(UnmanagedType.LPArray)]UIntPtr[] objects, uint length);
        //LLRING_API int llring_dequeue_burst(struct llring *r, void **obj_table, unsigned n);

    }
    public class LLRing {
        UIntPtr ring;
        public LLRing(uint slots)
        {
            ring = LLRingExternal.llring_alloc_and_init(slots, 0, 0);
        }
        ~LLRing() {
            LLRingExternal.free_ring(ring);
        }
        public int MultiProducerEnqueue(UIntPtr[] objs) {
            return LLRingExternal.llring_enqueue_bulk(ring, objs, (uint)objs.Length);
        }
        public int MultiConsumerDequeue(ref UIntPtr[] array) {
            //Object[] objArray = new Object[array.Length];
            int ret = LLRingExternal.llring_dequeue_burst(ring, array, (uint)array.Length);
            //Array.Copy(objArray, array, ret);
            return ret;
        }
    }
}
