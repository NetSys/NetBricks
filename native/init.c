#include <assert.h>
#include <numa.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <rte_config.h>
#include <rte_cycles.h>
#include <rte_eal.h>
#include <rte_ethdev.h>
#include <rte_timer.h>

#include "mempool.h"
/* Declared within eal_thread.c, but not exposed */
RTE_DECLARE_PER_LCORE(unsigned, _socket_id);

/* Called by each secondary threads on ZCSI, responsible for affinitization,
 * etc.*/
int init_thread(int tid, int core) {
    /* Among other things this affinitizes the thread */
    /* Set thread ID correctly */
    RTE_PER_LCORE(_lcore_id)     = tid;
    RTE_PER_LCORE(_mempool_core) = core;
    return 1;
}
