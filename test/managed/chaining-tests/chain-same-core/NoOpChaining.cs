using System;
using E2D2.SNApi;
using E2D2;
using System.Runtime.CompilerServices; 
namespace E2D2 {
	public sealed class NoOpVF : IE2D2Component {
		public NoOpVF() {
		}

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void PushBatch(ref PacketBuffer packets) {
			// Do nothing
		}
	}
	public sealed class NoOpChainingTest {
		public static void Main(string[] args) {
			SoftNic.init_softnic (2, "test");
			IE2D2Component vf1 = new NoOpVF();
			IE2D2Component vf2 = new NoOpVF();
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
