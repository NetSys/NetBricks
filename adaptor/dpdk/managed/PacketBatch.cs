using System;
using System.Security;
using System.Runtime.InteropServices;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices;
using System.Collections.Generic;
using System.Collections;
namespace ZCSI.DPDK {
	// [Unique]
	public sealed unsafe class PacketBatch : IDisposable {
		internal IntPtr _packetPointers;
		internal IntPtr *_packetPointerArray;
		private int _length;
		private int _available;
		private Packet _packet;
		private int _pindex;

		public Packet this[int index] {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get {
				if (index > _available) {
					throw new IndexOutOfRangeException();
				}
				if (_packet.mbufAddress != Packet.zero) {
					_packetPointerArray[_pindex] = _packet.mbufAddress;
				}
				_packet.wrapMbuf(_packetPointerArray[index]);
				_packetPointerArray[index] = Packet.zero; 
				_pindex = index;
				return _packet;
			}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal unsafe PacketBatch(int size) {
			_packet = new Packet();
			_packetPointers = 
				Marshal.AllocHGlobal(size * sizeof(UInt64));
			_packetPointerArray = (IntPtr*)_packetPointers;
			_length = size;
			_pindex = -1;
		}

		public int Length {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get {return _available;}
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			internal set {_available = value;}
		}

		public int MaxLength {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get {return _length;}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal void Invalidate() {
			_available = 0;
		}

		public void Dispose() {
			Dispose(true);
			GC.SuppressFinalize(this);
		}

		void Dispose(bool disposing) {
			if (_available > 0) {
				Console.WriteLine("Disposing packet batch");
				// Compact everything
				int j = 0;
				for (int i = 0; i < _available; i++) {
					if (_packetPointerArray[i] != Packet.zero) {
						_packetPointerArray[j++] = _packetPointerArray[i]; 
					} else {
						_available--;
					}
				}
				DPDK.FreePacketBatch(this);
			}
		}

		~PacketBatch() {
			Dispose(false);
		}
	}
}
