using System;
using System.Security;
using System.Runtime.InteropServices;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices; 
using System.Collections.Generic;
using System.Collections;
namespace E2D2.SNApi {
	public sealed class Packet {
		internal IntPtr buf_addr;
		internal IntPtr pkt;
		private IntPtr zero;
		private EthHdr _ethHdr;
		private Ipv4Hdr _ipHdr;
#if UNIQUE_CHECK
		int owner;
		bool owner_checked;
#endif

#if UNIQUE_CHECK
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		private static bool checkOwner(Packet packet) {
			packet.owner_checked = (packet.owner == SoftNic.CurrentVF());
			return packet.owner_checked;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal void setOwner(int nextOwner) {
			owner = nextOwner;
			owner_checked = false;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		static internal void setOwnerStatic (Packet pkt, int nextOwner) {
			pkt.owner = nextOwner;
			pkt.owner_checked = false;
		}

		public EthHdr ethHdr { 
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { if (owner_checked  || checkOwner(this)) { return _ethHdr;}
				  else { throw new SecurityException("Bad access"); }
			}
		}

		public Ipv4Hdr ipHdr {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { if (owner_checked || checkOwner(this)) { return _ipHdr;}
				  else {throw new SecurityException("Bad access");}
			}
		}

		public unsafe UInt16 buf_len {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { if (owner_checked || checkOwner(this)) { return *(ushort*)((pkt + 16)); }
				  else {throw new SecurityException("Bad access");}
			}
		}

		public unsafe UInt16 data_off { 
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { if (owner_checked || checkOwner(this)) { return *(ushort*)(pkt + 18); } 
				  else {throw new SecurityException("Bad access");}
			}
		}

		public unsafe UInt16 data_len { 
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { if (owner_checked || checkOwner(this)) { return *(ushort*)(pkt + 34); } 
				  else {throw new SecurityException("Bad access");}
			}
			set { if (owner_checked || checkOwner(this)) { 
				  	ushort* ptr = (ushort*)(pkt + 34);
				  	*ptr = (ushort)value;}
				  else {throw new SecurityException("Bad access");}
			}
		}

		public unsafe UInt16 pkt_len { 
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { if (owner_checked || checkOwner(this)) { return *(ushort*)(pkt + 36); } 
				  else {throw new SecurityException("Bad access");}
			}
			set { if (owner_checked || checkOwner(this)) { 
				  	ushort* ptr = (ushort*)(pkt + 36);
				  	*ptr = (ushort)value; }
				  else {throw new SecurityException("Bad access");}
			}
		}
#else
		public EthHdr ethHdr { 
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return _ethHdr; }
		}

		public Ipv4Hdr ipHdr {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return _ipHdr;}
		}

		public unsafe UInt16 buf_len {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get {return *(ushort*)((pkt + 16)); }
		}

		public unsafe UInt16 data_off { 
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *(ushort*)(pkt + 18); } 
		}

		public unsafe UInt16 data_len { 
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *(ushort*)(pkt + 34); } 
			set { ushort* ptr = (ushort*)(pkt + 34);
				  *ptr = (ushort)value;}
		}

		public unsafe UInt32 pkt_len { 
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return *(uint*)(pkt + 36); }
			set { uint* ptr = (uint*)(pkt + 36);
				  *ptr = value;}
		} 


#endif

		internal Packet() {
			zero = IntPtr.Zero;
			ZeroPacket();
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal unsafe void ZeroPacket() {
			buf_addr = zero;
			pkt = zero;
			_ethHdr.m_baseAddr = zero;
			_ipHdr.m_baseAddr = zero;
#if UNIQUE_CHECK
			owner = 0;
			owner_checked = false;
#endif
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal unsafe void SetPacket(IntPtr pkt) {
			buf_addr = (IntPtr)(*((void**)pkt));
			this.pkt = pkt;
			this._ethHdr.SetBase(buf_addr + (int)this.data_off);
			this._ipHdr.SetBase(buf_addr + (int)this.data_off + 14);
		}

		public unsafe void Print() {
			string hex_data = "";
			if (buf_addr == IntPtr.Zero)
				return;
			IntPtr pkt_addr = buf_addr + data_off;

			for (int i = 0; i < data_len; i++) {
				hex_data += string.Format("{0:X2} ", *(byte*)(pkt_addr + i));
				if (i % 4 == 3)
					hex_data += " ";
				if (i % 16 == 15)
					hex_data += "\n";
			}
			hex_data += "\n";
			Console.WriteLine(hex_data);
		}

		public struct EthHdr {
			internal IntPtr m_baseAddr;

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			internal EthHdr(IntPtr baseAddr) {
				m_baseAddr = baseAddr;
			}

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			internal void SetBase(IntPtr baseAddr) {
				m_baseAddr = baseAddr;
			}

			public unsafe UInt64 DestMac {
				[MethodImpl(MethodImplOptions.AggressiveInlining)]
				get {return ((*(UInt64*)(m_baseAddr)) & 0xffffffffffff0000 >> 8); }
				[MethodImpl(MethodImplOptions.AggressiveInlining)]
				set {UInt64* ptr = (UInt64*)m_baseAddr;
					*ptr = ((value & 0xffffffffffff) << 8) | (*ptr & 0xffff); 
				}
			}

			public unsafe UInt64 SrcMac {
				[MethodImpl(MethodImplOptions.AggressiveInlining)]
				get {return ((*(UInt64*)(m_baseAddr + 6)) & 0xffffffffffff0000 >> 8); }
				[MethodImpl(MethodImplOptions.AggressiveInlining)]
				set {UInt64* ptr = (UInt64*)(m_baseAddr + 6);
					*ptr = ((value & 0xffffffffffff) << 8) | (*ptr & 0xffff); 
				}
			}

			public unsafe UInt16 EthType {
				[MethodImpl(MethodImplOptions.AggressiveInlining)]
				get { return *(UInt16*)(m_baseAddr + 12); }
				[MethodImpl(MethodImplOptions.AggressiveInlining)]
				set {
					UInt16* ptr = (UInt16*)(m_baseAddr + 12);
					*ptr = value; 
				}
			}
		}

		public struct Ipv4Hdr {
			internal IntPtr m_baseAddr;

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			internal Ipv4Hdr(IntPtr baseAddr) {
				m_baseAddr = baseAddr;
			}

			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			internal void SetBase(IntPtr baseAddr) {
				m_baseAddr = baseAddr;
			}

			public unsafe UInt32 SrcIP {
				[MethodImpl(MethodImplOptions.AggressiveInlining)]
				get { return *((UInt32*)(m_baseAddr + 12)); }
				[MethodImpl(MethodImplOptions.AggressiveInlining)]
				set { 
					UInt32* ptr = (UInt32*)(m_baseAddr + 12);
					*ptr = value;
				}
			}

			public unsafe UInt32 DestIP {
				[MethodImpl(MethodImplOptions.AggressiveInlining)]
				get { return *((UInt32*)(m_baseAddr + 16)); }
				[MethodImpl(MethodImplOptions.AggressiveInlining)]
				set {
					UInt32* ptr = (UInt32*)(m_baseAddr + 16);
					*ptr = value;
				}
			}
		}
	}

	public unsafe sealed class PacketBuffer : System.Collections.Generic.IEnumerable<Packet> {
		internal IntPtr m_pktPointers;
		internal Packet m_packet;
		internal int m_available;
		internal int m_length;
		internal IntPtr* m_pktPointerArray;

		public int Length {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get { return m_available; }
		}

		internal void PrintArray() {
			for (int i = 0; i < m_available; i++) {
				Console.WriteLine("Addr = {0}", m_pktPointerArray[i]);
			}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal unsafe PacketBuffer(int size) {
			m_pktPointers = Marshal.AllocHGlobal(size * sizeof(UInt64));
			m_pktPointerArray = (IntPtr*)m_pktPointers;
			m_packet = new Packet();
			m_available = 0;
			m_length = size;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		private unsafe Packet PopulatePacket (int i) {
			m_packet.SetPacket(m_pktPointerArray[i]);
			return m_packet;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void ZeroAll () {
			m_available = 0;
			m_packet.ZeroPacket();
		}


		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public System.Collections.Generic.IEnumerator<Packet> GetEnumerator() {
			for (int i = 0; i < m_available; i++) {
				PopulatePacket(i);
				yield return m_packet;
			}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		private IEnumerator GetEnumerator1() {
			return this.GetEnumerator();
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		IEnumerator IEnumerable.GetEnumerator() {
			return GetEnumerator1();
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal UInt32 FromArray(IntPtr[] array, UInt32 length) {
			// Use this to try and elide bounds check
			Contract.Requires(length < array.Length, "Cannot have a smaller array than length");
			for (int i = 0; i < length; i++) {
				m_pktPointerArray[i] = array[i];
			}
			m_available = (int)length;
			return length;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal UInt32 ToArray(IntPtr[] array, UInt32 length) {
			// Use this to try and elide bounds check
			Contract.Requires(length < array.Length, "Cannot have a smaller array than length");
			for (int i = 0; i < length; i++) {
				array[i] = m_pktPointerArray[i];
			}
			return length;
		}

		public Packet this[int i] {
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			get {
				// We need this because otherwise access is unsafe, potentially violating
				// isolation
				// TODO Uncomment
				//if (i > m_available) {
					//throw new IndexOutOfRangeException();
				//}
				return PopulatePacket (i);
			}

		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void IteratePackets(Action<Packet> proc) {
			for (int i = 0; i < m_available; i++) {
				m_packet.SetPacket(m_pktPointerArray[i]);
				proc(m_packet);
			}
		}

#if UNIQUE_CHECK
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal void setOwner(int owner) {
			Packet.setOwnerStatic(m_packet, owner);
		}
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal static void setOwnerStatic(PacketBuffer pkt, int owner) {
			Packet.setOwnerStatic(pkt.m_packet, owner);
		}
#endif
	}

	public sealed class SoftNic {

		[DllImport("sn")] 
		public static extern void init_softnic(UInt64 cpumask, string name);

		[DllImport("sn")]
		public static unsafe extern IntPtr init_port (string ifname);

		[DllImport("sn")]
		static unsafe extern int sn_receive_pkts (IntPtr port, int rxq, IntPtr pkts, int cnt );

		[DllImport("sn")]
		static unsafe extern int sn_send_pkts (IntPtr port, int txq, IntPtr pkts, int cnt );

		[DllImport("sn")]
		static unsafe extern void sn_snb_free (IntPtr pkt);

		[DllImport("sn")]
		static unsafe extern void sn_snb_free_bulk_range(IntPtr pkts, int start, int cnt);

		[DllImport("sn")]
		public static unsafe extern void sn_init_thread(int core);

		[DllImport("sn")]
		public static unsafe extern uint sn_get_lcore_id();

		[DllImport("sn")]
		internal static unsafe extern void sn_snb_copy_batch(IntPtr src, IntPtr dest, int cnt);

		[DllImport("sn")]
		internal static unsafe extern void sn_snb_alloc_bulk(IntPtr snbs, int cnt);
		
		[DllImport("sn")]
		public static extern UInt64 sn_rdtsc();

		[DllImport("sn")]
		public static extern void sn_wait(UInt64 cycles);

  		[DllImport("libc.so.6")]
  		private static extern long sysconf(int conf);

  		public static long GetCpuFreq() {
  			return sysconf(2);
		}
		
#if UNIQUE_CHECK
		// Should not be int
		[ThreadStatic] private static int currentVF = 0xfffffff;
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static int CurrentVF() {
			return currentVF;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal static void SetVF(int vf) {
			currentVF = vf;
		}
#endif

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static unsafe void CopyBatch(ref PacketBuffer src, ref PacketBuffer dst) {
			sn_snb_copy_batch(src.m_pktPointers, dst.m_pktPointers, src.m_available);
			dst.m_available = src.m_available;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static unsafe void AllocBatch(ref PacketBuffer snbs, int count, ushort size) {
			sn_snb_alloc_bulk(snbs.m_pktPointers, count);
			snbs.m_available = count;
			for (int i = 0; i < count; i++) {
				Packet p = snbs[i];
				p.pkt_len = p.data_len = size;
			}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static unsafe void ReleasePackets (ref PacketBuffer pkts, int start, int end) {
			sn_snb_free_bulk_range(pkts.m_pktPointers, start, end - start);
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static PacketBuffer CreatePacketBuffer (int bsize) {
			return new PacketBuffer(bsize);
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static int ReceiveBatch (IntPtr port, int rxq, ref PacketBuffer pkts) {
			int rcvd = sn_receive_pkts(port, rxq, pkts.m_pktPointers, pkts.m_length);
			pkts.m_available = rcvd;
			return rcvd;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static int SendBatch (IntPtr port, int txq, ref PacketBuffer pkts) {
			int sent = sn_send_pkts(port, txq, pkts.m_pktPointers, pkts.m_available);
			// For now free everything else, but this is the wrong thing to do here
			if (sent < pkts.m_available) {
				ReleasePackets(ref pkts, sent, pkts.m_available);
			}
			pkts.ZeroAll();
			return sent;
		}
	}
}
