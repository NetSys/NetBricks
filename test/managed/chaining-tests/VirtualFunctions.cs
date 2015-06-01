using System;
using E2D2.SNApi;
using E2D2;
using E2D2.Collections;
using System.Runtime.CompilerServices; 
using System.IO;
using System.Net;
namespace E2D2 {
	public sealed class NoOpVF : IE2D2Component {
		public NoOpVF() {
		}

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void PushBatch(ref PacketBuffer packets) {
			// Do nothing
		}
	}
	public sealed class IPLookupVF : IE2D2Component {
		IPLookup m_lookup;
		public IPLookupVF(IPLookup lookup) {
			m_lookup = lookup;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void PushBatch(ref PacketBuffer packets) {
			int len = packets.Length;
			for (int i = 0; i < len; i++) {
				uint addr = packets[i].ipHdr.SrcIP;
				IPLookup.RouteLookupStatic(m_lookup, addr);
			}
		}
	}
}
