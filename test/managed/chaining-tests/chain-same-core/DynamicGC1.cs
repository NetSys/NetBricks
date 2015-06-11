using System;
using E2D2.SNApi;
using E2D2;
using E2D2.Collections;
using System.Runtime.CompilerServices; 
using System.IO;
using System.Net;
namespace E2D2 {
	public sealed class DynamicGCTest {
		public static void Main(string[] args) {
			IE2D2Component vf1 = new VarGCAlloc(256);
			SoftNic.init_softnic (2, "test");
			IntPtr port1 = SoftNic.init_port ("vport0");
			IntPtr port2 = SoftNic.init_port ("vport1");
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			Console.WriteLine("DPDK LCORE setting {0}", SoftNic.sn_get_lcore_id());
			while (true) {
				int rcvd = SoftNic.ReceiveBatch(port1, 0, ref pkts);
				try {
					vf1.PushBatch(ref pkts);
				} catch (Exception) {
				}
				if (rcvd > 0) {
					SoftNic.SendBatch(port2, 0, ref pkts);
				}
			}
		}
	}
}
