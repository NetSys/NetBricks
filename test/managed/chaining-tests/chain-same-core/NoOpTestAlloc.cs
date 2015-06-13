using System;
using E2D2.SNApi;
using E2D2;
using System.Runtime.CompilerServices; 
namespace E2D2 {
	public sealed class NoOpChainingTest {
		// Move to SoftNIC api
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal static void CopyAndSendPacket(ref PacketBuffer pktIn, ref PacketBuffer pktOut) {
			//SoftNic.CopyBatch(ref pktIn, ref pktOut);
			SoftNic.AllocBatch(ref pktOut, pktIn.Length, pktIn[0].data_len);
			SoftNic.ReleasePackets(ref pktIn, 0, pktIn.Length);
			pktIn.m_available = 0;
		}
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
			PacketBuffer[] pktBufs = new PacketBuffer[2];
			pktBufs[0] = SoftNic.CreatePacketBuffer(32);
			pktBufs[1] = SoftNic.CreatePacketBuffer(32);
			while (true) {
				int rcvd = SoftNic.ReceiveBatch(port1, 0, ref pktBufs[0]);
				if (rcvd > 0) {
					for (int i = 0; i < vfs.Length; i++) {
						try {
							vfs[i].PushBatch(ref pktBufs[i & 0x1]);
						} catch (Exception) {
						}
						CopyAndSendPacket(ref pktBufs[i & 0x1], ref pktBufs[((i & 0x1) + 1) & 0x1]);
					}
					SoftNic.SendBatch(port2, 0, ref pktBufs[vfs.Length & 0x1]);
				}
			}
		}
	}
}
