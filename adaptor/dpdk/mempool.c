#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <rte_config.h>
#include <rte_cycles.h>
#include <rte_timer.h>
#include <rte_ethdev.h>
#include <rte_eal.h>
#include <rte_ether.h>

/* Largely taken from SoftNIC (snbuf.c) */
#define NUM_PFRAMES	(16384 - 1) // Number of pframes in the mempool
#define NUM_MEMPOOL_CACHE 512 // Size of per-core object cache.

/* Currently creating only one pool per NUMA node. */
static struct rte_mempool *pframe_pool[RTE_MAX_NUMA_NODES];

/* Get mempool for calling thread's socket */
static inline struct rte_mempool *current_pframe_pool()
{
	return pframe_pool[rte_socket_id()];
}

static int init_mempool_socket(int sid)
{
	char name[256];
	sprintf(name, "prframe%d", sid);
	pframe_pool[sid] = rte_pktmbuf_pool_create(name,
			NUM_PFRAMES,
			NUM_MEMPOOL_CACHE,
			0,
			RTE_MBUF_DEFAULT_BUF_SIZE,
			sid);
	return pframe_pool[sid] != NULL;
}

int init_mempool()
{
	int initialized[RTE_MAX_NUMA_NODES];
	for (int i = 0; i < RTE_MAX_NUMA_NODES; i++)
		initialized[i] = 0;

	/* Loop through all cores, to see if any of them belong to this
	 * socket. */
	for (int i = 0; i < RTE_MAX_LCORE; i++) {
		int sid = rte_lcore_to_socket_id(i);
		if (!initialized[sid]) {
			if (!init_mempool_socket(sid)) {
				goto fail;
			}
			initialized[sid] = 1;
		}
	}
	return 1;
fail:
	/* FIXME: Should ideally free up the pools here, but have no way of
	 * doing so currently */
	return 0;
}

struct rte_mbuf* mbuf_alloc()
{
	return rte_pktmbuf_alloc(current_pframe_pool());
}
