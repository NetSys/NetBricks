using System;
using System.Security;
using System.Runtime.InteropServices;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices;
using System.Collections.Generic;
using System.Collections;
using System.Runtime.ConstrainedExecution;
namespace ZCSI.DPDK {
	// FIXME:
	// [Unique]
	public sealed class Packet : IDisposable {
		internal IntPtr _mbufAddress;
		static internal IntPtr zero;

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[DllImport("zcsi")]
		private static extern IntPtr mbuf_alloc();
		
		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[DllImport("zcsi")]
		private static extern void  mbuf_free(IntPtr buf);

		[DllImport("zcsi")]
		private static extern void dump_pkt(IntPtr array);
		
		[DllImport("zcsi")]
		internal static extern void set_packet_data(IntPtr array, int cnt, int offset, IntPtr data, int size);

		[DllImport("zcsi")]
		internal static unsafe extern void set_packet_data_at_offset(IntPtr array, int* offsets, int cnt, 
				IntPtr data, int size);  
		
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal unsafe static void SetPacketData(PacketBatch batch, int start, int end, int offset, 
				ushort data) {
			int cnt = end - start;
			start = batch.Start + start;
			if (start > batch.End || end > batch.Length) {
				throw new IndexOutOfRangeException("Accessing beyond the end of a batch");
			}
			set_packet_data(batch._packetPointers + (8 * start), cnt, offset, new IntPtr((void*)&data), 2);
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal unsafe static void SetPacketData(PacketBatch batch, int start, int end, int offset, 
				byte[] chunk) {
			int cnt = end - start;
			start = batch.Start + start;
			if (start > batch.End || end > batch.Length) {
				throw new IndexOutOfRangeException("Accessing beyond the end of a batch");
			}
			int length = chunk.Length;
			fixed (byte* data = chunk) {
				set_packet_data(batch._packetPointers + (8 * start), cnt, offset, 
						new IntPtr((void*)data), length);
			}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal unsafe static void SetPacketData(PacketBatch batch, int[] offsets, int start, 
				int end, byte[] chunk) {
			int cnt = end - start;
			start = batch.Start + start;
			if (start > batch.End || end > batch.Length) {
				throw new IndexOutOfRangeException("Accessing beyond the end of a batch");
			}
			int length = chunk.Length;
			fixed (byte* data = chunk) {
				fixed(int* offsetsP = offsets) {
					set_packet_data_at_offset(batch._packetPointers + (8 * start), 
							offsetsP + (4 * start), cnt, 
							new IntPtr((void*)data), length);
				}
			}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public unsafe static PacketBatch.TransformBatchDelegate PacketDataOperator(int start, int end,
				int offset, ushort data) {
			return (batch => SetPacketData(batch, start, end, offset, data));
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public unsafe static PacketBatch.TransformBatchDelegate PacketDataOperator(int offset, ushort data) {
			return (batch => SetPacketData(batch, 0, batch.Length, offset, data));
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public unsafe static PacketBatch.TransformBatchDelegate PacketDataOperator(int start, int end,
				int offset, byte[] data) {
			return (batch => SetPacketData(batch, start, end, offset, data));
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public unsafe static PacketBatch.TransformBatchDelegate PacketDataOperator(int offset, byte[] data) {
			return (batch => SetPacketData(batch, 0, batch.Length, offset, data)); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public unsafe static PacketBatch.TransformBatchDelegate PacketDataOperator(int start, int end, 
				int[] offsets, byte[] data) {
			return (batch => SetPacketData(batch, offsets, start, end, data)); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public unsafe static PacketBatch.TransformBatchDelegate PacketDataOperator(int[] offsets, byte[] data) {
			return (batch => SetPacketData(batch, offsets, 0, batch.Length, data)); 
		}
		
		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static Packet AllocatePacket() {
			IntPtr address = mbuf_alloc();
			return new Packet(address);
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void Dump() {
			dump_pkt(_mbufAddress);
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		public void Dispose() {
			Dispose(true);
			GC.SuppressFinalize(this);
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		void Dispose(bool disposing) {
			if (_mbufAddress != zero) {
				mbuf_free(_mbufAddress);
				_mbufAddress = zero;
			}
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		~Packet() {
			Dispose(false);
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal Packet() {
			zero = IntPtr.Zero;
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal Packet(IntPtr address) : this() {
			_mbufAddress = address;
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal void WrapMbuf(IntPtr address) {
			_mbufAddress = address;
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal void Clear() {
			_mbufAddress = zero;
		}

		public unsafe IntPtr DataAddress {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return BufAddr + DataOff; } 
		}
		
		// Cache line 0
		internal unsafe IntPtr BufAddr {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((IntPtr*)(_mbufAddress + 0)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((IntPtr*)(_mbufAddress + 0)) = value; }
		}

		// No one needs physical address in our case
		private unsafe IntPtr PhysAddr {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((IntPtr*)(_mbufAddress + 8)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			set { *((IntPtr*)(_mbufAddress + 8)) = value; }
		}

		public unsafe ushort BufLen {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(_mbufAddress + 16)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(_mbufAddress + 16)) = value; }
		}

		internal unsafe ushort DataOff {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(_mbufAddress + 18)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(_mbufAddress + 18)) = value; }
		}

		// Skipping refcnt (16-bits) since access without using
		// rte_mbuf* methods is prohibited.

		internal unsafe byte NbSegs {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((byte*)(_mbufAddress + 22)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((byte*)(_mbufAddress + 22)) = value; }
		}

		internal unsafe byte Port {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((byte*)(_mbufAddress + 23)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((byte*)(_mbufAddress + 23)) = value; }
		}

		internal unsafe ulong OffloadFlags {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ulong*)(_mbufAddress + 24)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ulong*)(_mbufAddress + 24)) = value; }
		}

		// Assuming RTE_NEXT_ABI is true.
		internal unsafe uint PacketType {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((uint*)(_mbufAddress + 32)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((uint*)(_mbufAddress + 32)) = value; }
		}

		public unsafe uint PacketLen {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get{ return *((uint*)(_mbufAddress + 36)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((uint*)(_mbufAddress + 36)) = value; }
		}

		internal unsafe ushort DataLen {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(_mbufAddress + 40)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(_mbufAddress + 40)) = value; }
		}

		internal unsafe ushort VlanTci {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(_mbufAddress + 42)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(_mbufAddress + 42)) = value; }
		}

		internal unsafe ulong Hash {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ulong*)(_mbufAddress + 44)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ulong*)(_mbufAddress + 44)) = value; }
		}

		internal unsafe uint Seqn {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((uint*)(_mbufAddress + 52)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((uint*)(_mbufAddress + 52)) = value; }
		}

		internal unsafe ushort VlanTciOuter  {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(_mbufAddress + 56)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(_mbufAddress + 56)) = value; }
		}

		// Cache line 1
		// FIXME: Do we need to also add stuff for userdata
		internal unsafe ulong UserData {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ulong*)(_mbufAddress + 64)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ulong*)(_mbufAddress + 64)) = value; }
		}

		// rte_mempool: not putting this in since no one needs to know
		private unsafe IntPtr NextMbuf {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((IntPtr*)(_mbufAddress + 80)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			set { *((IntPtr*)(_mbufAddress + 80)) = value; }
		}

		internal unsafe ulong TxOffload {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ulong*)(_mbufAddress + 88)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ulong*)(_mbufAddress + 88)) = value; }
		}

		internal unsafe ushort PrivSize  {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(_mbufAddress + 96)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(_mbufAddress + 96)) = value; }
		}

		internal unsafe ushort TimeSync  {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *((ushort*)(_mbufAddress + 98)); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			private set { *((ushort*)(_mbufAddress + 98)) = value; }
		}
	}
}
