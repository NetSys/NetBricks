using System;
using System.Security;
using System.Runtime.InteropServices;
using System.Diagnostics;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices;
using System.Collections.Generic;
using System.Collections;
using System.Net.NetworkInformation;
using ZCSI.DPDK;
namespace Test {
	public class AllocTest {
		public static int Main(string[] argv) {
			// Initialize DPDK
			const int core = 1;
#if NATIVE
			ushort type = (ushort)0x8100;
#else
			ushort type = DPDK.ChangeEndianness((ushort)0x8100);
#endif
			DPDK.init_system(core);
			DPDK.init_thread(1, core);
			Console.WriteLine("Size {0}", PMDPorts.SizeofEthDevInfo()); 
			Console.WriteLine("Found {0} PMD ports", PMDPorts.NumPMDPorts()); 
			var ports = PMDPorts.GetPMDPortInfo();
			foreach (var port in ports) {
				var pci = port.PCIDev;
				Console.WriteLine(
				  "port {0} RXQ {7} TXQ {8} {1:x4}:{2:x2}:{3:x2}.{4:x2} {5:x4}:{6:x4}",
						   port.DriverName,
						   pci.Address.Domain,
						   pci.Address.Bus,
						   pci.Address.DevId,
						   pci.Address.Function,
						   pci.Id.VendorId,
						   pci.Id.DeviceId,
						   port.MaxRxQueues,
						   port.MaxTxQueues);
			}
#if SEND
			if (ports.Length == 0) {
				Console.WriteLine("No PMD port is found");
			} else {
				//var pmdPort = PMDPort.LoopbackPort(0, 1);
				var sendPort = PMDPort.Port(0, 1);
#endif
#if SEND && RECV
				var recvPort = PMDPort.Port(1, 1);
#endif
				int sent = 0; int received = 0;
				var sw = Stopwatch.StartNew();
				long lastSecond = sw.Elapsed.Seconds;
				var sendBatch = new PacketBatch(32);
#if NATIVE
				var source = PhysicalAddress.Parse("01-02-03-04-05-06");
				var dest = PhysicalAddress.Parse("06-05-04-03-02-01");
				var transformer = EthPacket.HeaderOperator(source, dest, type);
#else
				var transformer = EthPacket.SlowTypeOperator(type);
#endif
#if CHECK_VLAN
				var ethPacket = new EthPacket();
#endif
				const int CHECK_BATCHES = 100;
				while (true) {
					for (int i = 0; i < CHECK_BATCHES; i++) {
						PacketBatch.AllocatePacketBatch(sendBatch);
						sendBatch.Transform(transformer);
#if CHECK_VLAN
						sendBatch.Assert((pkt) => 
								{return ethPacket.FromPacket(pkt).VlanTagged;});
#endif
#if SEND
						sent += sendPort.SendPacketBatch(sendBatch, 0);
#else
						sent += sendBatch.Length;
#endif
						sendBatch.ClearBatch();
#if RECV
						recvPort.ReceivePacketBatch(sendBatch, 32, 0);
						received += sendBatch.Length; 
						sendBatch.ClearBatch();
#endif
					}
					if (lastSecond != sw.Elapsed.Seconds) {
						var current = sw.Elapsed.Seconds;
						Console.WriteLine("{0} {1} rx={2} tx={3}",
								core,
								(current - lastSecond),
								received, sent);
						sw.Restart();
						lastSecond = sw.Elapsed.Seconds;
						sent = 0;
						received = 0;
					}
				}
#if SEND
			}
			return 1;
#endif
		}
	}
}
