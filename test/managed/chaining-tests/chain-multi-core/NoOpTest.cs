using System;
using E2D2.SNApi;
using E2D2;
using E2D2.Collections;
using System.Runtime.CompilerServices; 
using System.IO;
using System.Net;
using System.Threading;
using System.Diagnostics; 
namespace E2D2.SNApi {
	public sealed class NoOpTest {
		private static Stopwatch stopWatch;
		private static UInt64 totalDrops = 0;
		internal static void ThreadSource(IE2D2Component vf, int core, string vport, ref LLRingPacket ring) {
			SoftNic.sn_init_thread(core);
			IntPtr port1 = SoftNic.init_port (vport);
			Console.WriteLine("DPDK LCORE setting {0}", SoftNic.sn_get_lcore_id());
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			while (true) {
				int rcvd = SoftNic.ReceiveBatch(port1, 0, ref pkts);
				if (rcvd > 0) {
					try {
						vf.PushBatch(ref pkts);
					} catch (Exception) {
					}
					int sent = (int)(ring.SingleProducerEnqueuePackets(ref pkts) & (~LLRingPacket.RING_QUOT_EXCEED)) ;
					if (sent < pkts.m_available) {					
						totalDrops += (ulong)(pkts.m_available - sent);
						SoftNic.ReleasePackets(ref pkts, sent, pkts.m_available);
					}
					pkts.ZeroAll();
				}
			}
		}

		internal static void ThreadDestination(IE2D2Component vf, int core, string vport, ref LLRingPacket ring) {
			SoftNic.sn_init_thread(core);
			IntPtr port2 = SoftNic.init_port (vport);
			Console.WriteLine("DPDK LCORE setting {0}", SoftNic.sn_get_lcore_id());
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			while (true) {
				uint rcvd = ring.SingleConsumerDequeuePackets(ref pkts);
				if (rcvd > 0) {
					try {
						vf.PushBatch(ref pkts);
					} catch (Exception) {
					}
					if (rcvd > 0) {
						SoftNic.SendBatch(port2, 0, ref pkts);
					}
				}
			}
		}

		static void OnExit (object sender, EventArgs e) {
			Console.WriteLine("Lifetime packet drops from ring {0} in {1} ticks (freq {2})", 
					totalDrops, stopWatch.ElapsedTicks, Stopwatch.Frequency);
		}
		public static void Main(string[] args) {
			Console.CancelKeyPress += new ConsoleCancelEventHandler(OnExit);
			var ring = new LLRingPacket(64, true, true);
			SoftNic.init_softnic (1, "test");
			IE2D2Component vf1 = new NoOpVF();
			IE2D2Component vf2 = new NoOpVF();
    		Thread source = new Thread(new ThreadStart(() => ThreadSource(vf1, 2, "vport0", ref ring)));
    		Thread consum = new Thread(new ThreadStart(() => ThreadDestination(vf2, 3, "vport1", ref ring)));
    		source.Start();
    		consum.Start();
			stopWatch = Stopwatch.StartNew();
    		source.Join();
    		consum.Join();
		}
	}
}
