using System;
using E2D2.SNApi;
using System.Runtime.CompilerServices; 
namespace E2D2 {
	interface IE2D2Component {
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
		void PushBatch(ref PacketBuffer packets);
	}
}
