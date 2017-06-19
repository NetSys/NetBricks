#ifndef __MEMPOOL_H__
#define __MEMPOOL_H__
#include <rte_config.h>
#include <rte_cycles.h>
#include <rte_eal.h>
#include <rte_ethdev.h>
#include <rte_ether.h>
#include <rte_mbuf.h>
#include <rte_timer.h>
RTE_DECLARE_PER_LCORE(int, _mempool_core);

typedef struct rte_mbuf* restrict* restrict mbuf_array_t;
/* Called by system initialization */
int init_mempool_core(int core);
int init_mempool(int master_core, unsigned int mempool_size, unsigned int mcache_size, unsigned short slots);
int init_secondary_mempool(const char* mempool_name);
int find_secondary_mempool();
struct rte_mbuf* mbuf_alloc();
void mbuf_free(struct rte_mbuf* buf);
int mbuf_alloc_bulk(mbuf_array_t array, uint16_t len, int cnt);
int mbuf_free_bulk(mbuf_array_t array, int cnt);
struct rte_mempool* get_pframe_pool(int coreid, int sid);
struct rte_mempool* get_mempool_for_core(int coreid);
#endif
