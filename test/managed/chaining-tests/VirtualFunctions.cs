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

	public sealed class FixedGCAlloc : IE2D2Component {
		int m_bytes;
		public FixedGCAlloc(int bytes) {
			m_bytes = bytes;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void PushBatch(ref PacketBuffer packets) {
			int len = packets.Length;
			byte[][] fillers = new byte[len][];
			for (int i = 0; i < len; i++) {
				fillers[i] = new byte[m_bytes];
			}
		}
	}

	public sealed class VarGCAlloc : IE2D2Component {
		int m_maxBytes;
		public VarGCAlloc(int maxBytes) {
			m_maxBytes = maxBytes;
		}

		[MethodImpl(MethodImplOptions.AggressiveInlining)]
		public void PushBatch(ref PacketBuffer packets) {
			int len = packets.Length;
			byte[][] fillers = new byte[len][];
			for (int i = 0; i < len; i++) {
				int size = (int)packets[i].ipHdr.SrcIP % m_maxBytes;
				fillers[i] = new byte[size];
			}
		}
	}
}
