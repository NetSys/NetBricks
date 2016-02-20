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

#include <simd.h>
#include <mempool.h>

#define PER_CORE 0

/* Largely taken from SoftNIC (snbuf.c) */
#define NUM_PFRAMES	(16384 - 1) // Number of pframes in the mempool
#define NUM_MEMPOOL_CACHE 512 // Size of per-core object cache.

RTE_DEFINE_PER_LCORE(int, _mempool_core) = 0;

#if PER_CORE
/* Creating one pool per core. */
static struct rte_mempool *pframe_pool[RTE_MAX_LCORE];
#else 
/* Creating one pool per NUMA node. */
static struct rte_mempool *pframe_pool[RTE_MAX_NUMA_NODES];
#endif

/*Needed for bulk allocation */
struct rte_mbuf mbuf_template;

#if PER_CORE
#define MEMPOOL_ID RTE_PER_LCORE(_mempool_core)
#else
#define MEMPOOL_ID rte_socket_id()
#endif

/* Get mempool for calling thread's socket */
static inline struct rte_mempool *current_pframe_pool()
{
	return pframe_pool[MEMPOOL_ID];
}

struct rte_mempool *get_pframe_pool(int coreid, int sid) {
#if PER_CORE
	return pframe_pool[coreid];
#else
	return pframe_pool[sid];
#endif
}

static int init_mempool_socket(int coreid, int sid)
{
	char name[256];
	sprintf(name, "pframe%d", coreid);
#if PER_CORE
	pframe_pool[coreid] = rte_pktmbuf_pool_create(name,
			NUM_PFRAMES,
			NUM_MEMPOOL_CACHE,
			0,
			RTE_MBUF_DEFAULT_BUF_SIZE,
			sid);
	return pframe_pool[coreid] != NULL;
#else
	pframe_pool[sid] = rte_pktmbuf_pool_create(name,
			NUM_PFRAMES,
			NUM_MEMPOOL_CACHE,
			0,
			RTE_MBUF_DEFAULT_BUF_SIZE,
			sid);
	return pframe_pool[sid] != NULL;
#endif
}

int init_mempool()
{
#if (!PER_CORE)
	int initialized[RTE_MAX_NUMA_NODES];
	for (int i = 0; i < RTE_MAX_NUMA_NODES; i++) {
		initialized[i] = 0;
	}
#endif
	/* Loop through all cores, to see if any of them belong to this
	 * socket. */
	for (int i = 0; i < RTE_MAX_LCORE; i++) {
		int sid = rte_lcore_to_socket_id(i);
#if (!PER_CORE)
		if (!initialized[sid]) {
#endif
			struct rte_mbuf *mbuf;
			if (!init_mempool_socket(i, sid)) {
				goto fail;
			}
			/* Initialize mbuf template */
#if PER_CORE
			mbuf = rte_pktmbuf_alloc(pframe_pool[i]);
#else
			mbuf = rte_pktmbuf_alloc(pframe_pool[sid]);
#endif
			mbuf_template = *mbuf;
			rte_pktmbuf_free(mbuf);
#if (!PER_CORE)
			initialized[sid] = 1;
		}
#endif
	}
	return 0;
fail:
	/* FIXME: Should ideally free up the pools here, but have no way of
	 * doing so currently */
	return -ENOMEM;
}

struct rte_mbuf* mbuf_alloc()
{
	return rte_pktmbuf_alloc(current_pframe_pool());
}

void mbuf_free(struct rte_mbuf* buf)
{
	rte_pktmbuf_free(buf);
}

/* Using AVX for now. Revisit this decision someday */
/* mbuf_alloc_bulk: Bulk alloc packets.
 *	array: Array to allocate into.
 *	len: Length
 *	cnt: Count
 */
int mbuf_alloc_bulk(mbuf_array_t array, uint16_t len, int cnt)
{
	int ret;
	int i;

	__m128i template;	/* 256-bit write was worse... */
	__m128i rxdesc_fields;

	struct rte_mbuf tmp;
	/* DPDK 2.1 specific
	 * packet_type 0 (32 bits)
	 * pkt_len len (32 bits)
	 * data_len len (16 bits)
	 * vlan_tci 0 (16 bits)
	 * rss 0 (32 bits)
	 */
	rxdesc_fields = _mm_setr_epi32(0, len, len, 0);

	ret = rte_mempool_get_bulk(current_pframe_pool(),
			(void**)array, cnt);
	if (ret != 0) {
		return ret;
	}

	template = *((__m128i*)&mbuf_template.buf_len);

	if (cnt & 1) {
		array[cnt] = &tmp;
	}

	/* 4 at a time didn't help */
	for (i = 0; i < cnt; i+=2) {
		/* since the data is likely to be in the store buffer
		 * as 64-bit writes, 128-bit read will cause stalls */
		struct rte_mbuf *mbuf0 = array[i];
		struct rte_mbuf *mbuf1 = array[i + 1];

		_mm_store_si128((__m128i *)&mbuf0->buf_len, template);
		_mm_store_si128((__m128i *)&mbuf0->packet_type,
				rxdesc_fields);

		_mm_store_si128((__m128i *)&mbuf1->buf_len, template);
		_mm_store_si128((__m128i *)&mbuf1->packet_type,
				rxdesc_fields);
	}

	if (cnt & 1)
		array[cnt] = NULL;
	return 0;
}

#define RTE_MBUF_FROM_BADDR(ba)     (((struct rte_mbuf *)(ba)) - 1)

/* for packets to be processed in the fast path, all packets must:
 * 1. share the same mempool
 * 2. single segment
 * 3. reference counter == 1
 * 4. the data buffer is embedded in the mbuf
 *    (Do not use RTE_MBUF_(IN)DIRECT, since there is a difference
 *     between DPDK 1.8 and 2.0) */
int mbuf_free_bulk(mbuf_array_t array, int cnt)
{
	struct rte_mempool *_pool = array[0]->pool;

	/* broadcast */
	// Offset contains two copies of sizeof(struct rte_mbuf)
	__m128i offset = _mm_set1_epi64x(sizeof(struct rte_mbuf));
	// Mask for byte 1-3 (inlusive)
	__m128i info_mask = _mm_set1_epi64x(0x00ffffff00000000UL);
	// consts for comparison
	__m128i info_simple = _mm_set1_epi64x(0x0001000100000000UL);
	__m128i pool = _mm_set1_epi64x((uint64_t) _pool);

	int i;

	for (i = 0; i < (cnt & ~1); i += 2) {
		struct rte_mbuf *mbuf0 = array[i];
		struct rte_mbuf *mbuf1 = array[i + 1];

		__m128i buf_addrs_derived;
		__m128i buf_addrs_actual;
		__m128i info;
		__m128i pools;
		__m128i vcmp1, vcmp2, vcmp3;

		// Pack two mbuf pointers into one _m128i
		__m128i mbuf_ptrs = gather_m128i(mbuf1, mbuf0);

		// Buffer addresses
		buf_addrs_actual = gather_m128i(&mbuf0->buf_addr, &mbuf1->buf_addr);
		// Do buffers begin right after mbufs (checking if buffers
		// are indirect).
		buf_addrs_derived = _mm_add_epi64(mbuf_ptrs, offset);

		/* refcnt and nb_segs must be 1 */
		info = gather_m128i(&mbuf0->buf_len, &mbuf1->buf_len);
		info = _mm_and_si128(info, info_mask);

		pools = gather_m128i(&mbuf0->pool, &mbuf1->pool);

		vcmp1 = _mm_cmpeq_epi64(buf_addrs_derived, buf_addrs_actual);
		vcmp2 = _mm_cmpeq_epi64(info, info_simple);
		vcmp3 = _mm_cmpeq_epi64(pool, pools);

		vcmp1 = _mm_and_si128(vcmp1, vcmp2);
		vcmp1 = _mm_and_si128(vcmp1, vcmp3);

		if (unlikely(_mm_movemask_epi8(vcmp1) != 0xffff))
			goto slow_path;
	}

	// Odd number of packets
	if (i < cnt) {
		struct rte_mbuf *mbuf = array[i];

		if (unlikely(mbuf->pool != _pool ||
				mbuf->next != NULL ||
				rte_mbuf_refcnt_read(mbuf) != 1 ||
				RTE_MBUF_FROM_BADDR(mbuf->buf_addr) != mbuf))
		{
			goto slow_path;
		}
	}

	/* NOTE: it seems that zeroing the refcnt of mbufs is not necessary.
	 * (allocators will reset them) */
	rte_mempool_put_bulk(_pool, (void **)array, cnt);
	return 0;

slow_path:
	for (i = 0; i < cnt; i++)
		mbuf_free(array[i]);
	return 0;
}

void dump_pkt(struct rte_mbuf* buf) {
	rte_pktmbuf_dump(stdout, buf, 16384);
}
