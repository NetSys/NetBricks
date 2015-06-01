using System;
using E2D2.SNApi;
using E2D2;
using E2D2.Collections;
using System.Runtime.CompilerServices; 
using System.IO;
using System.Net;
namespace E2D2 {
	public sealed class IpLookupChainingTest {
		public static void Main(string[] args) {
			if(args.Length < 1) {
				Console.WriteLine("Usage: IPLookupChainingTest <rib>");
				return;
			}
			IPLookup lookup1 = new IPLookup();
			IPLookup lookup2 = new IPLookup();
	  		
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
			SoftNic.init_softnic (2, "test");
			IntPtr port1 = SoftNic.init_port ("vport0");
			IntPtr port2 = SoftNic.init_port ("vport1");
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			while (true) {
				int rcvd = SoftNic.ReceiveBatch(port1, 0, ref pkts);
				try {
					vf1.PushBatch(ref pkts);
				} catch (Exception) {
				}
				try {
					vf2.PushBatch(ref pkts);
				} catch (Exception) {
				}
				if (rcvd > 0) {
					SoftNic.SendBatch(port2, 0, ref pkts);
				}
			}
		}
	}
}
