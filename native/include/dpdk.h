#ifndef __DPDK_H__
#define __DPDK_H__
/* Called from all secondary threads on ZCSI */
int init_thread(int tid, int core);
#endif
