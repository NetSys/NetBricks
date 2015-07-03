using System;
using E2D2.SNApi;
using E2D2;
using E2D2.Collections;
using System.Runtime.CompilerServices; 
using System.IO;
using System.Net;
using System.Threading;
namespace E2D2 {
	public sealed class IpLookupChainingTest {
		internal static void RunCoreChain(int core, 
										  int length,
										  string inport, 
										  string outport, 
										  int baseVf,
										  int baseRx, 
										  int nrxq, 
										  int baseTx, 
										  int ntxq,
										  IPLookup[] lookups) {
			Console.WriteLine("Starting thread with core {0} baserx {1} basetx {2} basevf {3} chain {4}", core, baseRx, baseTx, baseVf, length);
			SoftNic.sn_init_thread(core);
#if UNIQUE_CHECK
			Console.WriteLine("UNIQUENESS ENABLED");
			var vfIds = new int[length];
			for (int i = 0; i < vfIds.Length; i++) {
				// Should use some sort of randomness instead of doing it this way.
				vfIds[i] = i + baseVf;
			}
#endif

			IE2D2Component[] vfs = new IE2D2Component[length];
			for (int i = 0; i < vfs.Length; i++) {
				vfs[i] = new IPLookupVF(lookups[i + baseVf]);
			}
			IntPtr port1 = SoftNic.init_port (inport);
			IntPtr port2 = SoftNic.init_port (outport);
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			int pollRx = 0;
			int pollTx = 0;
			Console.WriteLine("Chain length is {0}", length);
			while (true) {
				int rcvd = SoftNic.ReceiveBatch(port1, baseRx + pollRx, ref pkts);
				pollRx = (pollRx + 1) % nrxq;
				if (rcvd > 0) {
					for (int i = 0; i < vfs.Length; i++) {
						try {
#if UNIQUE_CHECK
							SoftNic.SetVF(i + baseVf);
							PacketBuffer.setOwnerStatic(pkts, i + baseVf);
#endif
							vfs[i].PushBatch(ref pkts);
						} catch (Exception e) {
							Console.WriteLine("Encountered error, quitting " + e.Message);
							Environment.Exit(1);
						}
					}
					SoftNic.SendBatch(port2, baseTx + pollTx, ref pkts);
				}
				pollTx = (pollTx + 1) % ntxq;
			}

		}

		public static void Main(string[] args) {
			var options = E2D2OptionParser.ParseOptions(args);
			int nrxq = options.numRxq;
			int ntxq = options.numTxq;
			SoftNic.init_softnic (2, "test");
			int length = 0;
			if(args.Length - options.endIdx < 3) {
				Console.WriteLine("Usage: IPLookupChainingTest [opt] -- <rib> <threads> <length>");
				return;
			}

			int threads = Convert.ToInt32(args[options.endIdx + 1]);
			length = Convert.ToInt32(args[options.endIdx + 2]);

			Console.WriteLine("Chain length is {0}", length);
			IPLookup[] lookups = new IPLookup[length * threads];
			for (int i = 0; i < lookups.Length; i++) {
				lookups[i] = new IPLookup();

			}

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

			int baseCore = 0;
			Thread[] executors = new Thread[threads];
			for (int i = 0; i < threads; i++) {
				int currCore = baseCore;
				int currVf = i * length;
				String vport = String.Format("vport{0}", i * 100);
				executors[i] = new Thread(new ThreadStart(() => 
							RunCoreChain(currCore, 
								length, vport, vport, 
								currVf, 0, nrxq, 
								0, ntxq, lookups)));
				baseCore += 1;
			}

			for (int i = 0; i < executors.Length; i++) {
				executors[i].Start();
			}
			for (int i = 0; i < executors.Length; i++) {
				executors[i].Join();
			}
		}
	}
}
