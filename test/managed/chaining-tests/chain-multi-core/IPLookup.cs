using System;
using E2D2.SNApi;
using E2D2;
using E2D2.Collections;
using System.Runtime.CompilerServices; 
using System.IO;
using System.Net;
using System.Threading;
namespace E2D2.SNApi {
	public sealed class IpLookupChainingTest {
		internal static void ThreadSource(IE2D2Component vf, ulong core, string vport, ref LLRingPacket ring) {
			SoftNic.init_softnic (core, "test");
			IntPtr port1 = SoftNic.init_port (vport);
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
						SoftNic.ReleasePackets(ref pkts, sent, pkts.m_available);
					}
					pkts.ZeroAll();
				}
			}
		}

		internal static void ThreadDestination(IE2D2Component vf, ulong core, string vport, ref LLRingPacket ring) {
			SoftNic.init_softnic (core, "test");
			IntPtr port2 = SoftNic.init_port (vport);
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
		public static void Main(string[] args) {
			if(args.Length < 1) {
				Console.WriteLine("Usage: IPLookupChainingTest <rib>");
				return;
			}
			IPLookup lookup1 = new IPLookup();
			IPLookup lookup2 = new IPLookup();
			LLRingPacket ring = new LLRingPacket(32, true, true);
	  		
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
				}
			}
			IE2D2Component vf1 = new IPLookupVF(lookup1);
			IE2D2Component vf2 = new IPLookupVF(lookup2);
    		Thread source = new Thread(new ThreadStart(() => ThreadSource(vf1, 2, "vport0", ref ring)));
    		Thread consum = new Thread(new ThreadStart(() => ThreadDestination(vf2, 3, "vport1", ref ring)));
    		source.Start();
    		consum.Start();
    		source.Join();
    		consum.Join();
		}
	}
}
