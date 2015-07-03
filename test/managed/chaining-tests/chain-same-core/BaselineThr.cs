using System;
using E2D2.SNApi;
using E2D2;
using E2D2.Collections;
using System.Runtime.CompilerServices; 
using System.IO;
using System.Net;
using System.Threading;
namespace E2D2 {
	public sealed class BaselineChainingTest {
		internal static void RunCoreChain(int core, 
										  int length,
										  string inport, 
										  string outport, 
										  int baseVf,
										  int baseRx, 
										  int nrxq, 
										  int baseTx, 
										  int ntxq) {
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
				vfs[i] = new BaseLineVF();
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
			SoftNic.init_softnic (1, "test");
			if (args.Length - options.endIdx < 2) {
				Console.WriteLine("Must specify threads and length");
				Environment.Exit(1);
			}
			int threads = Convert.ToInt32(args[options.endIdx]);
			int length = Convert.ToInt32(args[options.endIdx + 1]);
			Console.WriteLine("Chain length is {0}", length);
			Console.WriteLine("Threads are {0}", threads);
			int baseCore = 0;
			int rxQueuesPerCore = nrxq / threads;
			int txQueuesPerCore = ntxq / threads;
			Thread[] executors = new Thread[threads];
			int baseRx = 0;
			int baseTx = 0;
			for (int i = 0; i < threads; i++) {
				int currRx = baseRx;
				int currTx = baseTx;
				int currCore = baseCore;
				int currVf = i * length;
				String vport = String.Format("vport{0}", i * 100);
				//executors[i] = new Thread(new ThreadStart(() => 
							//RunCoreChain(currCore, 
								//length, vport, vport, 
								//currVf, currRx, rxQueuesPerCore, 
								//currTx, txQueuesPerCore)));
				executors[i] = new Thread(new ThreadStart(() => 
							RunCoreChain(currCore, 
								length, vport, vport, 
								currVf, 0, nrxq, 
								0, ntxq)));
				baseRx += rxQueuesPerCore;
				baseTx += txQueuesPerCore;
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
