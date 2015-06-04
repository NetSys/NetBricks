using System;
using E2D2.SNApi;
using E2D2;
using E2D2.Collections;
using System.Runtime.CompilerServices; 
using System.IO;
using System.Net;
using System.Threading;
using System.Diagnostics; 
namespace E2D2 {
	public sealed class NoOpChainingTest {
		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		internal static void CopyAndSendPacket(ref PacketBuffer pktIn, ref PacketBuffer pktOut) {
			SoftNic.CopyBatch(ref pktIn, ref pktOut);
			SoftNic.ReleasePackets(ref pktIn, 0, pktIn.Length);
			pktIn.m_available = 0;
		}
		public static void Main(string[] args) {
			SoftNic.init_softnic (2, "test");

			int length = 2;
			if(args.Length < 1) {
				Console.WriteLine("Usage: IPLookupChainingTest <rib>");
				return;
			}
			if (args.Length == 2) {
				length += Convert.ToInt32(args[1]);
			}

			Console.WriteLine("Chain length is {0}", length);
			IPLookup[] lookups = new IPLookup[length];
			for (int i = 0; i < lookups.Length; i++) {
				lookups[i] = new IPLookup();

			}
			using (StreamReader ribReader = new StreamReader(args[0])) {
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
