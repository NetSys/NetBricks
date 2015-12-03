using System;
using System.Security;
using System.Runtime.InteropServices;
using System.Diagnostics.Contracts;
using System.Runtime.CompilerServices;
using System.Collections.Generic;
using System.Collections;
namespace ZCSI.DPDK {
	// FIXME: Spread this around
	public static class DPDK {
		// Called once when the system is called. The calling thread is
		// affinitized to core. Subsequent calls to init_thread can be
		// used for reaffinitization.
		[DllImport("zcsi")]
		public static extern int init_system(int core);

		// Set the tid (used by DPDK when allocating and freeing memory,
		// etc.) and the core affinity for the calling thread.
		[DllImport("zcsi")]
		public static extern void init_thread(int tid, int core);

	}
}
