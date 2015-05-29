using System;
using System.Runtime.InteropServices;

namespace test_sn
{
	class MainClass
	{
		[DllImport("sn")] 
		static extern void init_softnic(UInt64 cpumask, string name);

		[DllImport("sn")]
		static unsafe extern IntPtr init_port (string ifname);

		[DllImport("sn")]
		static unsafe extern int receive_pkts (IntPtr port, int rxq, IntPtr pkts, int cnt );
		[DllImport("sn")]
		static unsafe extern int send_pkts (IntPtr port, int txq, IntPtr pkts, int cnt );
		[DllImport("sn")]
		static unsafe extern void snbuf_free (IntPtr pkt);

		public static unsafe void freepkts (IntPtr pkts, int start, int end)
		{
			void** p = (void**)pkts;
			while (start < end) {
				snbuf_free ((IntPtr)p [start]);
				start++;
			}
		}

		public struct Stats {
			public UInt64 rx_pkts;
			public UInt64 rx_bytes;
			public UInt64 rx_batch;

			public UInt64 tx_pkts;
			public UInt64 tx_bytes;
			public UInt64 tx_batch;

		};



		public sealed class Packet
		{
			IntPtr buf_addr;
			IntPtr pkt;
			public unsafe UInt16 buf_len {get {return *(ushort*)((pkt + 16)); }}
			public unsafe UInt16 data_off { get { return *(ushort*)(pkt + 18); } }
			public unsafe UInt16 data_len { get { return *(ushort*)(pkt + 34); } }
			public unsafe UInt32 pkt_len { get { return *(uint*)(pkt + 36); } }
			public EthHdr ethHdr;
			public Ipv4Hdr ipHdr;

			public Packet()
			{
				buf_addr = IntPtr.Zero;
				pkt = IntPtr.Zero;
				ethHdr = new EthHdr(IntPtr.Zero);
				ipHdr = new Ipv4Hdr(IntPtr.Zero);

			}

			public unsafe Packet(IntPtr pkt)
			{
				buf_addr = (IntPtr)(*((void**)pkt));
				#if false
				for (int i = 0; i < 48; i+=4) {
					Console.WriteLine("{0}:{1:X}",i, *(int*)(pkt + i));
				}
				#endif

				this.pkt = pkt;
			}

			public unsafe void SetPacket(IntPtr pkt)
			{
				buf_addr = (IntPtr)(*((void**)pkt));
				this.pkt = pkt;
				//this.ethHdr.SetBase(buf_addr + (int)this.data_off);
				//this.ipHdr.SetBase(buf_addr + (int)this.data_off + 14);

				//Print();
			}

			public unsafe void Print()
			{
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

            public unsafe struct EthHdr {
                IntPtr m_baseAddr;
                public EthHdr(IntPtr baseAddr) {
                    m_baseAddr = baseAddr;
                }
                public void SetBase(IntPtr baseAddr) {
                    m_baseAddr = baseAddr;
                }
                public UInt64 DestMac {
                    get {return ((*(UInt64*)(m_baseAddr)) & 0xffffffffffff0000 >> 8); }
                    set {UInt64* ptr = (UInt64*)m_baseAddr;
                         *ptr = ((value & 0xffffffffffff) << 8) | (*ptr & 0xffff); }
                }
                public UInt64 SrcMac {
                    get {return ((*(UInt64*)(m_baseAddr + 6)) & 0xffffffffffff0000 >> 8); }
                    set {UInt64* ptr = (UInt64*)(m_baseAddr + 6);
                         *ptr = ((value & 0xffffffffffff) << 8) | (*ptr & 0xffff); }
                }
                public UInt16 EthType {
                    get { return *(UInt16*)(m_baseAddr + 12); }
                    set { UInt16* ptr = (UInt16*)(m_baseAddr + 12);
                          *ptr = value; }
                }
            }

            public unsafe struct Ipv4Hdr {
                IntPtr m_baseAddr;
                public Ipv4Hdr(IntPtr baseAddr) {
                    m_baseAddr = baseAddr;
                }
                public void SetBase(IntPtr baseAddr) {
                    m_baseAddr = baseAddr;
                }
                public UInt32 SrcIP {
                    get { return *((UInt32*)(m_baseAddr + 12)); }
                }
                public UInt32 DestIP {
                    get { return *((UInt32*)(m_baseAddr + 16)); }
                }
            }
		}


		static Stats stat = new Stats();
		static Packet pp = new Packet();
		static volatile uint j;
		public static unsafe void record_stat(IntPtr pkts, int cnt)
		{

			int i = 0;
			void** p = (void**)pkts;
			stat.rx_pkts += (ulong)cnt;
			stat.rx_batch += 1;
            for (i = 0; i < cnt; i++) {
                pp.SetPacket((IntPtr)(p[i]));
                j = pp.buf_len;
            }
		}

		public static void  Main (string[] args)
		{
			Console.WriteLine("This is the counting version");
			init_softnic (2, "test");
			IntPtr port1 = init_port ("vport0");
			IntPtr port2 = init_port ("vport1");

			System.IntPtr pkts = Marshal.AllocHGlobal(32 * 8);

			while (true) {
				int rcvd = receive_pkts (port1, 0, pkts, 32);
				if (rcvd > 0) {
                    record_stat (pkts, rcvd);
					int sent = send_pkts (port2, 0, pkts, rcvd);
					if (sent < rcvd) {
						freepkts (pkts, sent, rcvd);
					}
				}
			}
			Console.WriteLine ("Hello World!");
		}
	}
}
