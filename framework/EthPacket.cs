using System;
using System.Security;
using System.Runtime.InteropServices;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices;
using System.Collections.Generic;
using System.Collections;
using System.Runtime.ConstrainedExecution;
using System.Net.NetworkInformation;
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

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch.TransformBatchDelegate EtherTypeOperator(ushort type) {
			return Packet.PacketDataOperator(_typeOffset, type); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch.TransformBatchDelegate EtherTypeOperator(ushort type, int start, int end) {
			return Packet.PacketDataOperator(start, end, _typeOffset, type); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch.TransformBatchDelegate SrcMacOperator(PhysicalAddress address, 
				int start, int end) {
			return Packet.PacketDataOperator(start, end, _srcOffset, address.GetAddressBytes()); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch.TransformBatchDelegate SrcMacOperator(PhysicalAddress address) {
			return Packet.PacketDataOperator(_srcOffset, address.GetAddressBytes()); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch.TransformBatchDelegate DstMacOperator(PhysicalAddress address,
				int start, int end) {
			return Packet.PacketDataOperator(start, end, _dstOffset, address.GetAddressBytes()); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch.TransformBatchDelegate DstMacOperator(PhysicalAddress address) {
			return Packet.PacketDataOperator(_dstOffset, address.GetAddressBytes()); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch.TransformBatchDelegate AddressOperator(PhysicalAddress src,
				PhysicalAddress dst, int start, int end) {
			byte[] data = new byte[12];
			Buffer.BlockCopy(dst.GetAddressBytes(), 0, data, 0, 6);
			Buffer.BlockCopy(src.GetAddressBytes(), 0, data, 6, 6);
			return Packet.PacketDataOperator(start, end, _dstOffset, data); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch.TransformBatchDelegate AddressOperator(PhysicalAddress src,
				PhysicalAddress dst) {
			byte[] data = new byte[12];
			Buffer.BlockCopy(dst.GetAddressBytes(), 0, data, 0, 6);
			Buffer.BlockCopy(src.GetAddressBytes(), 0, data, 6, 6);
			return Packet.PacketDataOperator(_dstOffset, data); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch.TransformBatchDelegate HeaderOperator(PhysicalAddress src,
				PhysicalAddress dst, ushort type, int start, int end) {
			byte[] data = new byte[14];
			Buffer.BlockCopy(dst.GetAddressBytes(), 0, data, 0, 6);
			Buffer.BlockCopy(src.GetAddressBytes(), 0, data, 6, 6);
			data[12] = (byte)(type & 0xff);
			data[13] = (byte)((type & 0xff00) >> 8);
			return Packet.PacketDataOperator(start, end, _dstOffset, data); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBatch.TransformBatchDelegate HeaderOperator(PhysicalAddress src,
				PhysicalAddress dst, ushort type) {
			byte[] data = new byte[14];
			Buffer.BlockCopy(dst.GetAddressBytes(), 0, data, 0, 6);
			Buffer.BlockCopy(src.GetAddressBytes(), 0, data, 6, 6);
			data[13] = (byte)(type & 0xff);
			data[12] = (byte)((type & 0xff00) >> 8);
			return Packet.PacketDataOperator(_dstOffset, data); 
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static unsafe PacketBatch.TransformBatchDelegate SlowTypeOperator(ushort type) 
		{
			return (batch => {
				batch.Transform((pkt) => {
				    IntPtr typeAddr = pkt.DataAddress + _typeOffset;
				    *((ushort*)typeAddr) = type;
				});
			});
		}

		private IntPtr _hdrAddr;
		private MacAddressView _src;
		private MacAddressView _dst;
		private const int _dstOffset = 0;
		private const int _srcOffset = 6;
		private const int _typeOffset = 12;
		private const int _dstSize = 6;
		private const int _srcSize = 6;
		private const int _typeSize = 2;

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

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		private unsafe ushort getFirstTag() {
			return *(ushort*)(_hdrAddr + 12);
		}
		// Note this is not doing endianness conversion, since it might
		// save time to precompute correct endianness values rather than
		// converting at each instance.
		public unsafe ushort EtherType {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return getFirstTag(); }

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			set { *(ushort*)(_hdrAddr + 12) = value; }
		}

		public bool VlanTagged {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { var tag = getFirstTag();
			      return ((tag == 0x0081) || (tag == 0x0091)); }
		}
		

	}
}
