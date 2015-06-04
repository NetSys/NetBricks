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

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal static void CopyAndSendPacket(ref PacketBuffer pkts, ref PacketBuffer pktOut, ref LLRingPacket ring) {
			SoftNic.CopyBatch(ref pkts, ref pktOut);
			SoftNic.ReleasePackets(ref pkts, 0, pkts.Length);
			uint sent = ring.SingleProducerEnqueuePackets(ref pktOut);
			if (sent < pkts.m_available) {					
				totalDrops += (ulong)(pkts.m_available - sent);
				SoftNic.ReleasePackets(ref pktOut, (int)sent, pktOut.m_available);
			}
			pkts.ZeroAll();
		}

		internal static void ThreadSource(IE2D2Component vf, int core, string vport, ref LLRingPacket ring) {
			SoftNic.sn_init_thread(core);
			IntPtr port1 = SoftNic.init_port (vport);
			Console.WriteLine("DPDK LCORE setting {0}", SoftNic.sn_get_lcore_id());
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			PacketBuffer pktOut = SoftNic.CreatePacketBuffer(32);
			while (true) {
				int rcvd = SoftNic.ReceiveBatch(port1, 0, ref pkts);
				if (rcvd > 0) {
					try {
						vf.PushBatch(ref pkts);
					} catch (Exception) {
					}
					CopyAndSendPacket(ref pkts, ref pktOut, ref ring);
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

		internal static void ThreadIntermediate(IE2D2Component vf, 
												int core, 
												ref LLRingPacket ringRecv, 
												ref LLRingPacket ringSend) {
			SoftNic.sn_init_thread(core);
			Console.WriteLine("DPDK LCORE setting {0}", SoftNic.sn_get_lcore_id());
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			PacketBuffer pktOut = SoftNic.CreatePacketBuffer(32);
			while (true) {
				uint rcvd = ringRecv.SingleConsumerDequeuePackets(ref pkts);
				if (rcvd > 0) {
					try {
						vf.PushBatch(ref pkts);
					} catch (Exception) {
					}
					CopyAndSendPacket(ref pkts, ref pktOut, ref ringSend);
				}
			}
		}

		static void OnExit (object sender, EventArgs e) {
			Console.WriteLine("Lifetime packet drops from ring {0} in {1} ticks (freq {2})", 
					totalDrops, stopWatch.ElapsedTicks, Stopwatch.Frequency);
		}
		public static void Main(string[] args) {
			Console.CancelKeyPress += new ConsoleCancelEventHandler(OnExit);
			SoftNic.init_softnic (0, "test");
			int length = 2;
			if (args.Length == 1) {
				length += Convert.ToInt32(args[0]);
			}

			Console.WriteLine("Chain length is {0}", length);

			IE2D2Component[] vfs = new IE2D2Component[length];
			for (int i = 0; i < vfs.Length; i++) {
				vfs[i] = new NoOpVF();
			}

			LLRingPacket[] rings = new LLRingPacket[length - 1];
			for (int i = 0; i < rings.Length; i++) {
				rings[i] = new LLRingPacket(64, true, true);
			}

			Thread[] threads = new Thread[length];

    		threads[0] = new Thread(new ThreadStart(() => ThreadSource(vfs[0], 2, "vport0", ref rings[0])));
    		threads[threads.Length - 1] = new Thread(new ThreadStart(() => 
    							ThreadDestination(vfs[vfs.Length - 1], 2 + length, "vport1", ref rings[rings.Length - 1])));
    		for (int i = 1; i < length - 1; i++) {
    			int j = i;
    			threads[i] = new Thread(new ThreadStart(() => 
    						     ThreadIntermediate(vfs[j], 2 + i, ref rings[j - 1], ref rings[i])));
			}
			stopWatch = Stopwatch.StartNew();
			for (int i = 0; i < threads.Length; i++) {
				threads[i].Start();
			}
			for (int i = 0; i < threads.Length; i++) {
				threads[i].Join();
			}

		}
	}
}
