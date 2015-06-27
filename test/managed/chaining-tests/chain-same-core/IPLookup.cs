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
			var options = E2D2OptionParser.ParseOptions(args);
			int nrxq = options.numRxq;
			int ntxq = options.numTxq;
			SoftNic.init_softnic (2, "test");
			int length = 0;
			if(args.Length - options.endIdx < 1) {
				Console.WriteLine("Usage: IPLookupChainingTest [opt] -- <rib> [<length>]");
				return;
			}

			if (args.Length - options.endIdx == 2) {
				length += Convert.ToInt32(args[options.endIdx + 1]);
			}

			Console.WriteLine("Chain length is {0}", length);
			IPLookup[] lookups = new IPLookup[length];
			for (int i = 0; i < lookups.Length; i++) {
				lookups[i] = new IPLookup();

			}

#if UNIQUE_CHECK
			Console.WriteLine("UNIQUENESS ENABLED");
			var vfIds = new int[length];
			for (int i = 0; i < vfIds.Length; i++) {
				// Should use some sort of randomness instead of doing it this way.
				vfIds[i] = i;
			}
#endif
			using (StreamReader ribReader = new StreamReader(args[options.endIdx])) {
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
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			int pollRx= 0;
			int pollTx = 0;
			while (true) {
				int rcvd = SoftNic.ReceiveBatch(port1, pollRx, ref pkts);
				pollRx = (pollRx + 1) % nrxq;
				if (rcvd > 0) {
					for (int i = 0; i < vfs.Length; i++) {
						try {
#if UNIQUE_CHECK
							SoftNic.SetVF(i);
							PacketBuffer.setOwnerStatic(pkts, i);
#endif
							vfs[i].PushBatch(ref pkts);
						} catch (Exception) {
						}
					}
					SoftNic.SendBatch(port2, pollTx, ref pkts);
					pollTx = (pollTx + 1) % ntxq;
				}
			}
		}
	}
}
