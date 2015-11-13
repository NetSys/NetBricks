using System;
using System.Security;
using System.Runtime.InteropServices;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices;
using System.Collections.Generic;
using System.Collections;
using ZCSI.DPDK;
namespace Test {
	public class AllocTest {
		public static void pktBatchAllocTest() {
			for (int i = 0; i < 100; i++) {
				var batch = DPDK.AllocatePacketBatch(64, 32);
				Console.WriteLine("Allocated batch length {0}", batch.Length);
				for (int j = 0; j < batch.Length; j++) {
					Console.WriteLine("Packet len = {0}", batch[j].PacketLen);
				}
			}
		}
		public static void pktAllocTest() {
			for (int i = 0; i < 100; i++) {
				var pkt = DPDK.AllocatePacket();
				Console.WriteLine("Size: {0}", pkt.PacketLen);
			}
		}
		public static int Main(string[] argv) {
			// Initialize DPDK
			DPDK.init_system(2);
			pktBatchAllocTest();
			return 1;
		}
	}
}
