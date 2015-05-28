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



		public struct Packet
		{
			IntPtr buf_addr;
			IntPtr pkt;
			public unsafe UInt16 buf_len {get {return *(ushort*)((pkt + 16)); }}
			public unsafe UInt16 data_off { get { return *(ushort*)(pkt + 18); } }
			public unsafe UInt16 data_len { get { return *(ushort*)(pkt + 34); } }
			public unsafe UInt32 pkt_len { get { return *(uint*)(pkt + 36); } }

			public Packet()
			{
				buf_addr = IntPtr.Zero;
				pkt = IntPtr.Zero;
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
				//buf_len = *(ushort*)((char*)pkt + 16);
				//data_off = *(ushort*)((char*)pkt + 18);
				//data_len = *(ushort*)((char*)pkt + 34);
				//pkt_len = *(uint*)((char*)pkt + 36);
			}

			public unsafe void SetPacket(IntPtr pkt)
			{
				buf_addr = (IntPtr)(*((void**)pkt));
				this.pkt = pkt;

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
			struct eth_hdr
			{
			}
			struct ip_hdr
			{
			}
			struct tcp_hdr
			{
			}
			struct udp_hdr
			{
			}
		}


		static Stats stat = new Stats();
		static Packet pp = new Packet();
		public static unsafe void record_stat(IntPtr pkts, int cnt)
		{

			int i = 0;
			void** p = (void**)pkts;
			stat.rx_pkts += (ulong)cnt;
			stat.rx_batch += 1;
			while (i < cnt) {
				pp.SetPacket((IntPtr)(p[i]));
				stat.rx_bytes += pp.data_len;
				//Console.WriteLine("{0}, {1}, {2}, {3}", pp.buf_len, pp.data_off, pp.data_len, pp.pkt_len);
				i++;
			}
		}

		public static void  Main (string[] args)
		{
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
