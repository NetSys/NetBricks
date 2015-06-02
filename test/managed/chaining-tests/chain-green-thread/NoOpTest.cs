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
		static LLRingPacket ring;
		//static LLRing<IntPtr> ring;
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal static IEnumerator Source(IE2D2Component vf, IntPtr port1, IntPtr port2) {
			Console.WriteLine("Source VPORT is {0}", port1);
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			while (true) {
				int rcvd = SoftNic.ReceiveBatch(port1, 0, ref pkts);
				if (rcvd > 0) {
					try {
						vf.PushBatch(ref pkts);
					} catch (Exception) {
					}
					//pkts.ToArray(batch, (uint)rcvd);
					uint sent = ring.SingleProducerEnqueuePackets(ref pkts);
					if (sent < pkts.m_available) {					
						totalDrops += (ulong)(pkts.m_available - sent);
						SoftNic.ReleasePackets(ref pkts, (int)sent, pkts.m_available);
					}
					pkts.ZeroAll();
				}
				yield return 1;
			}
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal static IEnumerator Destination(IE2D2Component vf, IntPtr port2) {
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			while (true) {
				uint rcvd = ring.SingleConsumerDequeuePackets(ref pkts);
				//uint rcvd = ring.SingleConsumerDequeue(ref batch);
				//pkts.FromArray(batch, rcvd);
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

		public static void sched(IE2D2Component srcVF, IE2D2Component destVF, IntPtr svf, IntPtr dvf) {
			// First run once to get enumerable
			IEnumerator src = Source(srcVF, svf, dvf);
			IEnumerator dest = Destination(destVF, dvf);
			while (true) {
				src.MoveNext();
				dest.MoveNext();
			}
		}

		static void OnExit (object sender, EventArgs e) {
			Console.WriteLine("Lifetime packet drops from ring {0} in {1} ticks (freq {2})", 
					totalDrops, stopWatch.ElapsedTicks, Stopwatch.Frequency);
		}

		public static void Main(string[] args) {
			Console.CancelKeyPress += new ConsoleCancelEventHandler(OnExit);
			ring = new LLRingPacket(64, true, true);
	  		
			IE2D2Component vf1 = new NoOpVF();
			IE2D2Component vf2 = new NoOpVF();
			SoftNic.init_softnic (2, "test");
			IntPtr port1 = SoftNic.init_port ("vport0");
			IntPtr port2 = SoftNic.init_port ("vport1");
			Console.WriteLine("VPORT Src {0} Dest {1}", port1, port2);
			stopWatch = Stopwatch.StartNew();
			sched(vf1, vf2, port1, port2);
		}
	}
}
