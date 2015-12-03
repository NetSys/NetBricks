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

		internal static void ThreadIntermediate(IE2D2Component vf, int core, ref LLRingPacket ringRecv, ref LLRingPacket ringSend) {
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
				    SoftNic.SendBatch(port2, 0, ref pkts);
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
			if(args.Length < 1) {
				Console.WriteLine("Usage: IPLookupChainingTest <rib>");
				return;
			}
			IPLookup lookup1 = new IPLookup();
			IPLookup lookup2 = new IPLookup();
			IPLookup lookup3 = new IPLookup();
			LLRingPacket ringS2I = new LLRingPacket(32, true, true);
			LLRingPacket ringI2D = new LLRingPacket(32, true, true);
	  		
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
					lookup1.AddRoute(addrAsInt, len, dest);
					lookup2.AddRoute(addrAsInt, len, dest);
					lookup3.AddRoute(addrAsInt, len, dest);
				}
			}
			IE2D2Component vf1 = new IPLookupVF(lookup1);
			IE2D2Component vf2 = new IPLookupVF(lookup2);
			IE2D2Component vf3 = new IPLookupVF(lookup2);
    		Thread source = new Thread(new ThreadStart(() => ThreadSource(vf1, 2, "vport0", ref ringS2I)));
    		Thread interm = new Thread(new ThreadStart(() => ThreadIntermediate(vf2, 3, ref ringS2I, ref ringI2D)));
    		Thread consum = new Thread(new ThreadStart(() => ThreadDestination(vf3, 4, "vport1", ref ringI2D)));
    		source.Start();
    		interm.Start();
    		consum.Start();
			stopWatch = Stopwatch.StartNew();
    		source.Join();
    		interm.Join();
    		consum.Join();
		}
	}
}
