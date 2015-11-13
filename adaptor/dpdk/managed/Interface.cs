using System;
using System.Security;
using System.Runtime.InteropServices;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices;
using System.Collections.Generic;
using System.Collections;
namespace ZCSI.DPDK {
	public static class DPDK {
		// Called once when the system is called. The calling thread is
		// affinitized to core. Subsequent calls to init_thread can be
		// used for reaffinitization.
		[DllImport("zcsi")]
		public static extern int init_system(int core);

		// Set the tid (used by DPDK when allocating and freeing memory,
		// etc.) and the core affinity for the calling thread.
		[DllImport("zcsi")]
		public static extern void init_thread(int tid, int core);

		[DllImport("zcsi")]
		private static extern IntPtr mbuf_alloc();
		
		[DllImport("zcsi")]
		internal static extern void  mbuf_free(IntPtr buf);

		[DllImport("zcsi")]
		private static extern int 
			mbuf_alloc_bulk(IntPtr array, ushort len, int cnt);

		[DllImport("zcsi")]
		private static extern int mbuf_free_bulk(IntPtr array, int cnt);

		[DllImport("zcsi")]
		internal static extern void dump_pkt(IntPtr array);

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static Packet AllocatePacket() {
			IntPtr address = mbuf_alloc();
			return new Packet(address);
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch AllocatePacketBatch(
				 PacketBatch batch, ushort size, int count) {
			if (batch.MaxLength < count) {
				return null;
			}
			mbuf_alloc_bulk(batch._packetPointers, 
						size, count);
			batch.Length = count;
			return batch;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch AllocatePacketBatch(ushort size, 
				int count) {
			// FIXME: Unbounded?
			PacketBatch batch = new PacketBatch(count);
			return AllocatePacketBatch(batch, size, count);
		}


		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch AllocatePacketBatch(
				 PacketBatch batch, ushort size) {
			return AllocatePacketBatch(batch, size, batch.MaxLength);
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch AllocatePacketBatch(
				 PacketBatch batch) {
			const ushort size = 60;
			return AllocatePacketBatch(batch, size, batch.MaxLength);
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static void FreePacketBatch( PacketBatch batch) {
			mbuf_free_bulk(batch._packetPointers, batch.Length);
			batch.Length = 0;
		}
	}
}
