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
		private static IntPtr free;

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
			Console.WriteLine("Source core {0} vport {1} ring {2}", core, vport, ring);
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
			Console.WriteLine("Destination core {0} vport {1} ring {2}", core, vport, ring);
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
												LLRingPacket ringRecv, 
												LLRingPacket ringSend) {
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

		static Thread CreateThread(int tid, IE2D2Component vf, int core, LLRingPacket recv, LLRingPacket send) {
			Console.WriteLine("Starting Intermediate {0}", tid);
			return new Thread(new ThreadStart(() => {
						ThreadIntermediate(vf, core, recv, send);
						}));

		}
		public static void Main(string[] args) {
			Console.CancelKeyPress += new ConsoleCancelEventHandler(OnExit);
			SoftNic.init_softnic (0, "test");
			free = SoftNic.init_port ("vport_free"); 
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
    							ThreadDestination(vfs[vfs.Length - 1], 2 + (length - 1), "vport1", ref rings[rings.Length - 1])));
    		Console.WriteLine("Thread length {0}, ring length {1}", threads.Length, rings.Length);
			for (int i = 1; i < length - 1; i++) {
				threads[i] =  CreateThread(i,
								vfs[i],
								2 + i,
								rings[i - 1],
								rings[i]);
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
