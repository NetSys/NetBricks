using System;
using E2D2.SNApi;
using E2D2;
using System.Runtime.CompilerServices; 
namespace E2D2 {
	public sealed class NoOpChainingTest {
		public static void Main(string[] args) {
			SoftNic.init_softnic (2, "test");
			int length = 2;
			if (args.Length == 1) {
				length += Convert.ToInt32(args[0]);
			}

			Console.WriteLine("Chain length is {0}", length);

			IE2D2Component[] vfs = new IE2D2Component[length];
			for (int i = 0; i < vfs.Length; i++) {
				vfs[i] = new NoOpVF();
			}

			IntPtr port1 = SoftNic.init_port ("vport0");
			IntPtr port2 = SoftNic.init_port ("vport1");
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			while (true) {
			    int rcvd = SoftNic.ReceiveBatch(port1, 0, ref pkts);
				if (rcvd > 0) {
					for (int i = 0; i < vfs.Length; i++) {
						try {
							vfs[i].PushBatch(ref pkts);
						} catch (Exception) {
						}
					}
					SoftNic.SendBatch(port2, 0, ref pkts);
				}
            }
		}
	}
}
