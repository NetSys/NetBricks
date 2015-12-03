using System;
using System.Security;
using System.Runtime.InteropServices;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices;
using System.Collections.Generic;
using System.Collections;
using System.Runtime.ConstrainedExecution;
namespace ZCSI.DPDK {
	// [Unique]
	public sealed unsafe class PacketBatch : IDisposable {

		internal IntPtr _packetPointers;
		internal IntPtr *_packetPointerArray;

		private IntPtr _scratchPointers;
		private IntPtr *_scratchPointerArray;
		internal int _length;
		internal int _start;
		internal int _available;
		private Packet[] _pkts;
		private const int UNROLL_BY = 8;

		public delegate void TransformDelegate(Packet packet);
		public delegate bool FilterDelegate(Packet packet);

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void Transform(TransformDelegate transform) {
			// FIXME: _start
			for (int i = 0; i < (_available & UNROLL_BY); 
					i += UNROLL_BY) {
				_pkts[0]._mbufAddress = _packetPointerArray[i];
				transform(_pkts[0]);
				_pkts[1]._mbufAddress = _packetPointerArray[i + 1];
				transform(_pkts[1]);
				_pkts[2]._mbufAddress = _packetPointerArray[i + 2];
				transform(_pkts[2]);
				_pkts[3]._mbufAddress = _packetPointerArray[i + 3];
				transform(_pkts[3]);
				_pkts[4]._mbufAddress = _packetPointerArray[i + 4];
				transform(_pkts[4]);
				_pkts[5]._mbufAddress = _packetPointerArray[i + 5];
				transform(_pkts[5]);
				_pkts[6]._mbufAddress = _packetPointerArray[i + 6];
				transform(_pkts[6]);
				_pkts[7]._mbufAddress = _packetPointerArray[i + 7];
				transform(_pkts[7]);
			}

			for (int i = (_available & UNROLL_BY); 
					i < (_available & (~UNROLL_BY)); i++) {
				_pkts[0]._mbufAddress = _packetPointerArray[i];
				transform(_pkts[0]);
			}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void Filter(FilterDelegate filter) {
			int keep = 0;
			int discard = 0;
			for (int i = _start; i < _available; i++) {
				var ptr = _packetPointerArray[i];
				_pkts[0]._mbufAddress = ptr;
				if (filter(_pkts[0])) {
					_packetPointerArray[keep++] =
						ptr;
				} else {
					_scratchPointerArray[discard++] =
						ptr;
				}
		        }
		        _available = keep;
		        mbuf_free_bulk(_scratchPointers, discard);
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[DllImport("zcsi")]
		private static extern int 
			mbuf_alloc_bulk(IntPtr array, ushort len, int cnt);

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[DllImport("zcsi")]
		private static extern int mbuf_free_bulk(IntPtr array, int cnt);

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		private static void initializePacketBatch(PacketBatch batch,
				ushort size, int count) {
			mbuf_alloc_bulk(batch._packetPointers, 
						size, count);
			batch._available = count;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public unsafe PacketBatch(int length) {
			_packetPointers = 
				Marshal.AllocHGlobal(length * sizeof(UInt64));
			_packetPointerArray = (IntPtr*)_packetPointers;

			_scratchPointers = 
				Marshal.AllocHGlobal(length * sizeof(UInt64));
			_scratchPointerArray = (IntPtr*)_scratchPointers;
			_length = length;
			_pkts = new Packet[UNROLL_BY];
			for (int i = 0; i < UNROLL_BY; i++) {
				_pkts[i] = new Packet();
			}
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch AllocatePacketBatch(
				 PacketBatch batch, ushort size, int count) {
			
			Contract.Requires(count <= batch._length);
			Contract.Requires(batch._available == 0);

			initializePacketBatch(batch, size, count);

			return batch;
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch AllocatePacketBatch(
				 PacketBatch batch, ushort count) {
			
			Contract.Requires(count <= batch._length);
			Contract.Requires(batch._available == 0);

			initializePacketBatch(batch, 60, count);
			return batch;
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch AllocatePacketBatch(PacketBatch batch) {
			initializePacketBatch(batch, 60, batch._length);
			return batch;
		}

		[ReliabilityContract(Consistency.WillNotCorruptState, Cer.Success)]
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		private static void FreePacketBatch( PacketBatch batch) {
			mbuf_free_bulk(batch._packetPointers + (batch._start * sizeof(IntPtr)), 
					batch._available - batch._start);
			batch._available = 0;	
			batch._start = 0;
		}

		public int Length {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get {return _available - _start;}
		}

		// This is not the same as dispose for many reasons.
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void ClearBatch() {
			Dispose(true);
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void Dispose() {
			Dispose(true);
			GC.SuppressFinalize(this);
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		void Dispose(bool disposing) {
			if ((_available - _start) > 0) {
				FreePacketBatch(this);
			}
		}

		~PacketBatch() {
			Dispose(false);
		}
	}
}
