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

			if (args.Length - options.endIdx > 0) {
				length += Convert.ToInt32(args[options.endIdx]);
			}

			Console.WriteLine("Chain length is {0}", length);

#if UNIQUE_CHECK
			Console.WriteLine("UNIQUENESS ENABLED");
			var vfIds = new int[length];
			for (int i = 0; i < vfIds.Length; i++) {
				// Should use some sort of randomness instead of doing it this way.
				vfIds[i] = i;
			}
#endif

			IE2D2Component[] vfs = new IE2D2Component[length];
			for (int i = 0; i < vfs.Length; i++) {
				vfs[i] = new BaseLineVF();
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
						} catch (Exception e) {
							Console.WriteLine("Encountered error, quitting " + e.Message);
							Environment.Exit(1);
						}
					}
					SoftNic.SendBatch(port2, pollTx, ref pkts);
					pollTx = (pollTx + 1) % ntxq;
				}
			}
		}
	}
}
