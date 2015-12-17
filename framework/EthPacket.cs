using System;
using System.Security;
using System.Runtime.InteropServices;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices;
using System.Collections.Generic;
using System.Collections;
using System.Runtime.ConstrainedExecution;
namespace ZCSI.DPDK {
	public class MacAddressView {
		internal IntPtr _base;
		public unsafe byte this[int index] {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get {
				if (index >= 8) {
					throw new IndexOutOfRangeException();
				} 
				return *(byte*)(_base + index); 
			}

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			set {
				if (index >= 8) {
					throw new IndexOutOfRangeException();
				} 
				*(byte*)(_base + index) = value; 
			}
		}
	}

	// FIXME:
	// [Unique]
	public sealed class EthPacket {
		private IntPtr _hdrAddr;
		private MacAddressView _src;
		private MacAddressView _dst;
		public EthPacket() {
			_hdrAddr = Packet.zero;
			_src = new MacAddressView();
			_dst = new MacAddressView();
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public EthPacket FromPacket(Packet packet) {
			_hdrAddr = packet.DataAddress;
			_dst._base = _hdrAddr;
			_src._base = _hdrAddr + 6;
			return this;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void Reset() {
			_hdrAddr = _dst._base = _src._base = Packet.zero;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public unsafe void SetEtherType(ushort ether) {
			*(ushort*)(_hdrAddr + 12) = ether;
		}

		// d_addr 6
		public MacAddressView Destination {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return _dst; }
		}
		// s_addr 6
		public MacAddressView Source {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return _src; }
		}
		// Note this is not doing endianness conversion, since it might
		// save time to precompute correct endianness values rather than
		// converting at each instance.
		public unsafe ushort EtherType {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *(ushort*)(_hdrAddr + 12); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			set { *(ushort*)(_hdrAddr + 12) = value; }
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static unsafe void SetEtherTypeS(IntPtr data, ushort type) {
			//*(ushort*)(data + 12) = type;
		}
	}
}
