using System;
using System.Runtime.InteropServices;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices; 
namespace E2D2.SoftNic {
    public sealed class Packet {
        internal IntPtr buf_addr;
        internal IntPtr pkt;

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
        }

        public unsafe UInt32 pkt_len { 
            [MethodImpl(MethodImplOptions.AggressiveInlining)]
            get { return *(uint*)(pkt + 36); } 
        }
        public EthHdr ethHdr;
        public Ipv4Hdr ipHdr;

        public Packet() {
            buf_addr = IntPtr.Zero;
            pkt = IntPtr.Zero;
            ethHdr = new EthHdr(IntPtr.Zero);
            ipHdr = new Ipv4Hdr(IntPtr.Zero);

        }

        public unsafe Packet(IntPtr pkt) {
            buf_addr = (IntPtr)(*((void**)pkt));
            #if false
            for (int i = 0; i < 48; i+=4) {
                Console.WriteLine("{0}:{1:X}",i, *(int*)(pkt + i));
            }
            #endif

            this.pkt = pkt;
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        internal unsafe void ZeroPacket() {
            buf_addr = IntPtr.Zero;
            pkt = IntPtr.Zero;
            ethHdr.SetBase(IntPtr.Zero);
            ipHdr.SetBase(IntPtr.Zero);
            
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        internal unsafe void SetPacket(IntPtr pkt) {
            buf_addr = (IntPtr)(*((void**)pkt));
            this.pkt = pkt;
            this.ethHdr.SetBase(buf_addr + (int)this.data_off);
            this.ipHdr.SetBase(buf_addr + (int)this.data_off + 14);
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
            IntPtr m_baseAddr;

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
                set { UInt16* ptr = (UInt16*)(m_baseAddr + 12);
                      *ptr = value; 
                }
            }
        }

        public struct Ipv4Hdr {
            IntPtr m_baseAddr;
            
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
                set { UInt32* ptr = (UInt32*)(m_baseAddr + 12);
                      *ptr = value;
                }
            }
            
            public unsafe UInt32 DestIP {
                [MethodImpl(MethodImplOptions.AggressiveInlining)]
                get { return *((UInt32*)(m_baseAddr + 16)); }
                [MethodImpl(MethodImplOptions.AggressiveInlining)]
                set { UInt32* ptr = (UInt32*)(m_baseAddr + 16);
                      *ptr = value;
                }
            }
        }
    }

    public sealed class PacketBuffer {
        internal IntPtr m_pktPointers;
        internal Packet[] m_packets;
        internal int m_available;
        internal int m_length;
        
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        internal unsafe PacketBuffer(int size) {
            m_pktPointers = Marshal.AllocHGlobal(size * sizeof(UInt64));
            m_packets = new Packet[size];
            for (int i = 0; i < m_packets.Length; i++) {
                m_packets[i] = new Packet();
            }
            m_available = 0;
            m_length = size;
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        internal unsafe void PopulatePackets(int rcvd) {
            Contract.Requires(m_available + rcvd < m_packets.Length, "Don't add packets from other places");
			void** p = (void**)m_pktPointers;
            for (int i = 0; i < rcvd; i++, m_available++) {
                m_packets[m_available].SetPacket((IntPtr)(p[i]));
            }
        }

        private unsafe Packet PopulatePacket (int i) {
			void** p = (void**)m_pktPointers;
			m_packets[i].SetPacket((IntPtr)(p[i]));
			return m_packets[i];
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void ZeroPackets (int start, int end) {
		    Contract.Requires(start > 0, "Non negative index");
		    Contract.Requires(end < m_packets.Length, "End must be less than length of array");
            for (int i = start; i < end; i++) {
                m_packets[i].ZeroPacket();
            }
            int oldLimit = m_available;
            m_available = start;
            for (int i = end; i < oldLimit; i++) {
                Packet temp = m_packets[i];
                m_packets[i] = m_packets[m_available];
                m_packets[m_available] = temp;
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public void ZeroAll () {
            foreach (Packet p in m_packets) {
                p.ZeroPacket();
            }
            m_available = 0;
        }


        public Packet this[int i] {
            [MethodImpl(MethodImplOptions.AggressiveInlining)]
            get {
                if (i > m_available) {
                    throw new IndexOutOfRangeException();
                }
                return PopulatePacket (i);
            }

        }

    }
    
    public sealed class SoftNic {
		[DllImport("sn")] 
		public static extern void init_softnic(UInt64 cpumask, string name);

		[DllImport("sn")]
		public static unsafe extern IntPtr init_port (string ifname);

		[DllImport("sn")]
		static unsafe extern int receive_pkts (IntPtr port, int rxq, IntPtr pkts, int cnt );

		[DllImport("sn")]
		static unsafe extern int send_pkts (IntPtr port, int txq, IntPtr pkts, int cnt );

		[DllImport("sn")]
		static unsafe extern void snbuf_free (IntPtr pkt);

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal static unsafe void ReleasePackets (ref PacketBuffer pkts, int start, int end) {
			void** p = (void**)pkts.m_pktPointers;
		    while (start < end) {
		        snbuf_free ((IntPtr)p[start]);
		        start++;
            }
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static PacketBuffer CreatePacketBuffer (int bsize) {
            return new PacketBuffer(bsize);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static int ReceiveBatch (IntPtr port, int rxq, ref PacketBuffer pkts) {
            int rcvd = receive_pkts(port, rxq, pkts.m_pktPointers, pkts.m_length);
            pkts.m_available = rcvd;
            return rcvd;
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static int SendBatch (IntPtr port, int txq, ref PacketBuffer pkts) {
            int sent = send_pkts(port, txq, pkts.m_pktPointers, pkts.m_available);
            // For now free everything else, but this is the wrong thing to do here
            if (sent < pkts.m_available) {
                ReleasePackets(ref pkts, sent, pkts.m_available);
            }
            pkts.m_available = 0;
            return sent;
        }
    }

    public sealed class SoftNicTest {
        public static void Main(string[] args) {
			SoftNic.init_softnic (2, "test");
			IntPtr port1 = SoftNic.init_port ("vport0");
			IntPtr port2 = SoftNic.init_port ("vport1");
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			while (true) {
			    int rcvd = SoftNic.ReceiveBatch(port1, 0, ref pkts);
			    if (rcvd > 0) {
			        int sent = SoftNic.SendBatch(port2, 0, ref pkts);
                }
            }
        }
    }
}
