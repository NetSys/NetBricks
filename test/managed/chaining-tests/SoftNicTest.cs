using E2D2.SNApi;
using System;
using System.Runtime.CompilerServices; 
namespace E2D2 {
    public sealed class SoftNicTest {
        public static void Main(string[] args) {
			SoftNic.init_softnic (2, "test");
			IntPtr port1 = SoftNic.init_port ("vport0");
			IntPtr port2 = SoftNic.init_port ("vport1");
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			while (true) {
			    int rcvd = SoftNic.ReceiveBatch(port1, 0, ref pkts);
			    if (rcvd > 0) {
			        SoftNic.SendBatch(port2, 0, ref pkts);
                }
            }
        }
    }
}
