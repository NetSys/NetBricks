using System;
using System.Security;
using System.Runtime.InteropServices;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices;
using System.Collections.Generic;
using System.Collections;
namespace ZCSI.DPDK {
	// FIXME:
	// [Unique]
	public sealed class Packet : IDisposable {
		internal IntPtr mbufAddress;
		static internal IntPtr zero;

		public void Dispose() {
			Dispose(true);
			GC.SuppressFinalize(this);
		}

		void Dispose(bool disposing) {
			if (mbufAddress != zero) {
				DPDK.mbuf_free(mbufAddress);
				mbufAddress = zero;
			}
		}

		~Packet() {
			Dispose(false);
		}

		public void Dump() {
			if (mbufAddress != zero) {
				Console.WriteLine("Dumping packet at {0}", mbufAddress.ToString("X"));
				DPDK.dump_pkt(mbufAddress);
			}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal Packet() {
			zero = IntPtr.Zero;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal Packet(IntPtr address) : this() {
			mbufAddress = address;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal void wrapMbuf(IntPtr address) {
			mbufAddress = address;
		}
		
		// Cache line 0
		internal unsafe IntPtr BufAddr {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((IntPtr*)(mbufAddress + 0)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((IntPtr*)(mbufAddress + 0)) = value; }
		}

		// No one needs physical address in our case
		private unsafe IntPtr PhysAddr {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((IntPtr*)(mbufAddress + 8)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			set { *((IntPtr*)(mbufAddress + 8)) = value; }
		}

		public unsafe ushort BufLen {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(mbufAddress + 16)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(mbufAddress + 16)) = value; }
		}

		internal unsafe ushort DataOff {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(mbufAddress + 18)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(mbufAddress + 18)) = value; }
		}

		// Skipping refcnt (16-bits) since access without using
		// rte_mbuf* methods is prohibited.

		internal unsafe byte NbSegs {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((byte*)(mbufAddress + 22)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((byte*)(mbufAddress + 22)) = value; }
		}

		internal unsafe byte Port {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((byte*)(mbufAddress + 23)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((byte*)(mbufAddress + 23)) = value; }
		}

		internal unsafe ulong OffloadFlags {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ulong*)(mbufAddress + 24)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ulong*)(mbufAddress + 24)) = value; }
		}

		// Assuming RTE_NEXT_ABI is true.
		internal unsafe uint PacketType {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((uint*)(mbufAddress + 32)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((uint*)(mbufAddress + 32)) = value; }
		}

		public unsafe uint PacketLen {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get{ return *((uint*)(mbufAddress + 36)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((uint*)(mbufAddress + 36)) = value; }
		}

		internal unsafe ushort DataLen {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(mbufAddress + 40)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(mbufAddress + 40)) = value; }
		}

		internal unsafe ushort VlanTci {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(mbufAddress + 42)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(mbufAddress + 42)) = value; }
		}

		internal unsafe ulong Hash {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ulong*)(mbufAddress + 44)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ulong*)(mbufAddress + 44)) = value; }
		}

		internal unsafe uint Seqn {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((uint*)(mbufAddress + 52)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((uint*)(mbufAddress + 52)) = value; }
		}

		internal unsafe ushort VlanTciOuter  {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(mbufAddress + 56)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(mbufAddress + 56)) = value; }
		}

		// Cache line 1
		// FIXME: Do we need to also add stuff for userdata
		internal unsafe ulong UserData {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ulong*)(mbufAddress + 64)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ulong*)(mbufAddress + 64)) = value; }
		}

		// rte_mempool: not putting this in since no one needs to know
		private unsafe IntPtr NextMbuf {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((IntPtr*)(mbufAddress + 80)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			set { *((IntPtr*)(mbufAddress + 80)) = value; }
		}

		internal unsafe ulong TxOffload {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ulong*)(mbufAddress + 88)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ulong*)(mbufAddress + 88)) = value; }
		}

		internal unsafe ushort PrivSize  {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(mbufAddress + 96)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(mbufAddress + 96)) = value; }
		}

		internal unsafe ushort TimeSync  {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(mbufAddress + 98)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(mbufAddress + 98)) = value; }
		}
	}
}
