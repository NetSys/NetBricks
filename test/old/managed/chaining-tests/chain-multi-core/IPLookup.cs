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
	public sealed class IpLookupChainingTest {
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
			//SoftNic.init_softnic (core, "test");
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
					SoftNic.SendBatch(port2, 0, ref pkts);
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
			while (true) {
				uint rcvd = ringRecv.SingleConsumerDequeuePackets(ref pkts);
				if (rcvd > 0) {
					try {
						vf.PushBatch(ref pkts);
					} catch (Exception) {
					}
					int sent = (int)(ringSend.SingleProducerEnqueuePackets(ref pkts) & (~LLRingPacket.RING_QUOT_EXCEED)) ;
					if (sent < pkts.m_available) {
						totalDrops += (ulong)(pkts.m_available - sent);
						SoftNic.ReleasePackets(ref pkts, sent, pkts.m_available);
					}
					pkts.ZeroAll();
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
			if(args.Length < 1) {
				Console.WriteLine("Usage: IPLookupChainingTest <rib>");
				return;
			}
			if (args.Length == 2) {
				length += Convert.ToInt32(args[1]);
			}

			Console.WriteLine("Chain length is {0}", length);
			IPLookup[] lookups = new IPLookup[length];
			for (int i = 0; i < lookups.Length; i++) {
				lookups[i] = new IPLookup();

			}
			using (StreamReader ribReader = new StreamReader(args[0])) {
				while (ribReader.Peek() >= 0) {
					String line = ribReader.ReadLine();
					String[] parts = line.Split(' ');
					String[] addrParts = parts[0].Split('/');
					UInt16 dest = Convert.ToUInt16(parts[1]);
					UInt16 len = Convert.ToUInt16(addrParts[1]);
					IPAddress addr = IPAddress.Parse(addrParts[0]);
					UInt32 addrAsInt = 
						(UInt32)IPAddress.NetworkToHostOrder(
								BitConverter.ToInt32(
									addr.GetAddressBytes(), 0));
					foreach (IPLookup lookup in lookups) {
						lookup.AddRoute(addrAsInt, len, dest);
					}
				}
			}
	  		
			IE2D2Component[] vfs = new IE2D2Component[length];
			for (int i = 0; i < vfs.Length; i++) {
				vfs[i] = new IPLookupVF(lookups[i]);
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
    						     ThreadIntermediate(vfs[j], 2 + j, ref rings[j - 1], ref rings[j])));
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
