using System;
using E2D2.SNApi;
using E2D2;
using E2D2.Collections;
using System.Runtime.CompilerServices; 
using System.Diagnostics; 
using System.IO;
using System.Net;
using System.Threading;
using System.Collections;
//using E2D2.Collections.Concurrent;
namespace E2D2.SNApi {
	public sealed class IpLookupChainingTest {
		private static UInt64 totalDrops = 0;
		private static Stopwatch stopWatch;

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal static void CopyAndSendPacket(ref PacketBuffer pkts, ref PacketBuffer pktOut, ref LLRingPacket ring) {
			SoftNic.CopyBatch(ref pkts, ref pktOut);
			SoftNic.ReleasePackets(ref pkts, 0, pkts.Length);
			pkts.m_available = 0;

			uint sent = ring.SingleProducerEnqueuePackets(ref pktOut);
			if (sent < pkts.m_available) {					
				totalDrops += (ulong)(pkts.m_available - sent);
				SoftNic.ReleasePackets(ref pkts, (int)sent, pkts.m_available);
			}
			pkts.ZeroAll();
		}

		struct SourceElement {
			LLRingPacket ring;
			IE2D2Component vf;
			IntPtr port1;
			internal SourceElement(LLRingPacket _ring, IE2D2Component _vf, IntPtr _port) {
				ring = _ring;
				vf = _vf;
				port1 = _port;
			}
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			internal IEnumerator Run() {
				Console.WriteLine("Source VPORT is {0}", port1);
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
					yield return 1;
				}
			}
		}

		struct DestElement {
			LLRingPacket ring;
			IE2D2Component vf;
			IntPtr port2;
			internal DestElement(LLRingPacket _ring, IE2D2Component _vf, IntPtr _port) {
				ring = _ring;
				vf = _vf;
				port2 = _port;
			}
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			internal IEnumerator Run() {
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
					yield return 1;
				}
			}
		}

		struct IntermediateElement {
			LLRingPacket ringSrc;
			LLRingPacket ringDst;
			IE2D2Component vf;
			internal IntermediateElement(LLRingPacket _ringSrc, IE2D2Component _vf, LLRingPacket _ringDst) {
				ringSrc = _ringSrc;
				ringDst = _ringDst;
				vf = _vf;
			}
			[MethodImpl(MethodImplOptions.AggressiveInlining)]
			internal IEnumerator Run() {
				PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
				PacketBuffer pktOut = SoftNic.CreatePacketBuffer(32);
				while (true) {
					uint rcvd = ringSrc.SingleConsumerDequeuePackets(ref pkts);
					if (rcvd > 0) {
						try {
							vf.PushBatch(ref pkts);
						} catch (Exception) {
						}
						CopyAndSendPacket(ref pkts, ref pktOut, ref ringDst);
					}
					yield return 1;
				}
			}
		}

		static void sched(SourceElement srcVF, DestElement destVF, IntermediateElement[] mids) {
			// First run once to get enumerable
			IEnumerator[] tasks = new IEnumerator[mids.Length + 2];
			tasks[0] = srcVF.Run();
			tasks[tasks.Length - 1] = destVF.Run();
			for (int i = 0; i < mids.Length; i++) {
				tasks[i + 1] = mids[i].Run();
			}
			while (true) {
				for (int i=0; i<tasks.Length; i++){
					tasks[i].MoveNext();
				}
			}
		}

		static void OnExit (object sender, EventArgs e) {
			Console.WriteLine("Lifetime packet drops from ring {0} in {1} ticks (freq {2})", 
					totalDrops, stopWatch.ElapsedTicks, Stopwatch.Frequency);
		}

		public static void Main(string[] args) {
			Console.CancelKeyPress += new ConsoleCancelEventHandler(OnExit);
			SoftNic.init_softnic (2, "test");
			int length = 2;
			// Optionally take number of intermediate nodes to chain, note chain is thus n+2
			if(args.Length < 1) {
				Console.WriteLine("Usage: IPLookupChainingTest <rib> [intermediates]");
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
			IntPtr port1 = SoftNic.init_port ("vport0");
			IntPtr port2 = SoftNic.init_port ("vport1");
			Console.WriteLine("VPORT Src {0} Dest {1}", port1, port2);

			//LLRingPacket rings = new LLRingPacket(64, true, true);
			LLRingPacket[] rings = new LLRingPacket[length - 1];
			for (int i = 0; i < rings.Length; i++) {
				rings[i] = new LLRingPacket(64, true, true);
			}

			SourceElement src = new SourceElement(rings[0], vfs[0], port1);
			DestElement dst = new DestElement(rings[rings.Length - 1], vfs[length - 1], port2);
			IntermediateElement[] mids = new IntermediateElement[length - 2];
			for (int i=0; i<mids.Length; i++) {
				mids[i] = new IntermediateElement(rings[i], vfs[i + 1], rings[i + 1]);
			}
			stopWatch = Stopwatch.StartNew();
			sched(src, dst, mids);
		}
	}
}
