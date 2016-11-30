#ifndef __DPDK_H__
#define __DPDK_H__
/* Call this from the main thread on ZCSI to initialize things */
int init_system(int core);

/* Called from all secondary threads on ZCSI */
int init_thread(int tid, int core);
#include "mempool.h"
#endif
