#ifndef __LLRING_H__
#define __LLRING_H__
#if _MSC_VER
#define LLRING_API __declspec(dllexport)
#else
#define LLRING_API 
#endif 
/*-
 * adopted from DPDK's rte_ring.h
 *
 *   Copyright(c) 2010-2013 Intel Corporation. All rights reserved.
 *   All rights reserved.
 * 
 *   Redistribution and use in source and binary forms, with or without
 *   modification, are permitted provided that the following conditions
 *   are met:
 * 
 *     * Redistributions of source code must retain the above copyright
 *       notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above copyright
 *       notice, this list of conditions and the following disclaimer in
 *       the documentation and/or other materials provided with the
 *       distribution.
 *     * Neither the name of Intel Corporation nor the names of its
 *       contributors may be used to endorse or promote products derived
 *       from this software without specific prior written permission.
 * 
 *   THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 *   "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 *   LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 *   A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 *   OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 *   SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 *   LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 *   DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 *   THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 *   (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 *   OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

/*
 * Derived from FreeBSD's bufring.h
 *
 **************************************************************************
 *
 * Copyright (c) 2007-2009 Kip Macy kmacy@freebsd.org
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 *
 * 2. The name of Kip Macy nor the names of other
 *    contributors may be used to endorse or promote products derived from
 *    this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 *
 ***************************************************************************/

/**
 * Note: the ring implementation is not preemptable. A producer must not
 * be interrupted by another producer that uses the same ring.
 * Same for consumers.
 *
 * (I am not sure if preemption between a producer and a consumer is fine)
 */

#define LLRING_ENABLE_DEBUG 	0

#define LLRING_MAX_CORES	256

#define LLRING_ERR_NOBUF	1
#define LLRING_ERR_QUOT		2	/* successful but exceeded watermark */
#define LLRING_ERR_NOENT	3
#define LLRING_ERR_NOPOW2	4
#define LLRING_ERR_INVAL	5

#define LLRING_CACHELINE_SIZE	64
#if _MSC_VER
#define __llring_cache_aligned \
	    __declspec(align(LLRING_CACHELINE_SIZE))
#else
#define __llring_cache_aligned \
		__attribute__((__aligned__(LLRING_CACHELINE_SIZE)))
#endif
#ifdef __KERNEL__

#define llring_likely(x)	likely(x)
#define llring_unlikely(x)	unlikely(x)

static inline void llring_pause(void)	{ cpu_relax(); }

#else

#include <stdint.h>

#define llring_likely(x)      x
#define llring_unlikely(x)    x

#include <emmintrin.h>
#ifdef _MSC_VER
#include <windows.h>
#define inline __inline
static inline void llring_pause(void)	{ YieldProcessor(); }
#else
static inline void llring_pause(void)	{ _mm_pause(); }
#endif


#endif

/* dummy assembly operation to prevent compiler re-ordering of instructions */
#define COMPILER_BARRIER() do { _ReadWriteBarrier(); } while(0)

static inline int 
llring_atomic32_cmpset(volatile uint32_t *dst, uint32_t exp, uint32_t src)
{
#ifdef _MSC_VER
    return (InterlockedCompareExchange(dst, src, exp) == exp);
#else
	return __sync_bool_compare_and_swap(dst, exp, src);
#endif
}

enum llring_queue_behavior {
	LLRING_QUEUE_FIXED = 0, /* Enq/Deq a fixed number of items from a ring */
	LLRING_QUEUE_VARIABLE   /* Enq/Deq as many items a possible from ring */
};

#if 0 && LLRING_ENABLE_DEBUG
#ifdef __KERNEL__

static inline int llring_cpu_id()	{ return smp_processor_id(); }

#else	/* __KERNEL__ */

#ifdef _RTE_LCORE_H_
static inline int llring_cpu_id() 	{ return rte_lcore_id(); }
#else 	/* _RTE_LCORE_H_ */
#include <sched.h>
static inline int llring_cpu_id() 	{ return sched_getcpu(); }
#endif	/* RTE_LCORE_H_ */

#endif 	/* __KERNEL__ */

/**
 * A structure that stores the ring statistics (per-lcore).
 */
struct llring_debug_stats {
	uint64_t enq_success_bulk; /**< Successful enqueues number. */
	uint64_t enq_success_objs; /**< Objects successfully enqueued. */
	uint64_t enq_quota_bulk;   /**< Successful enqueues above watermark. */
	uint64_t enq_quota_objs;   /**< Objects enqueued above watermark. */
	uint64_t enq_fail_bulk;    /**< Failed enqueues number. */
	uint64_t enq_fail_objs;    /**< Objects that failed to be enqueued. */
	uint64_t deq_success_bulk; /**< Successful dequeues number. */
	uint64_t deq_success_objs; /**< Objects successfully dequeued. */
	uint64_t deq_fail_bulk;    /**< Failed dequeues number. */
	uint64_t deq_fail_objs;    /**< Objects that failed to be dequeued. */
} __llring_cache_aligned;
#endif	/* LLRING_ENABLE_DEBUG */

/**
 * The producer and the consumer have a head and a tail index. The particularity
 * of these index is that they are not between 0 and slots(ring). These indexes
 * are between 0 and 2^32, and we mask their value when we access the ring[]
 * field. Thanks to this assumption, we can do subtractions between 2 index
 * values in a modulo-32bit base: that's why the overflow of the indexes is not
 * a problem.
 */
__llring_cache_aligned struct llring{
	/** Mostly read-only status */
	__llring_cache_aligned struct{
		uint32_t slots;          /**< Size of ring. */
		uint32_t mask;           /**< Mask (slots-1) of ring. */
		uint32_t watermark;      /**< Maximum items before LLRING_ERR_QUOT. */
		uint32_t sp_enqueue;     /**< True, if single producer. */
		uint32_t sc_dequeue;     /**< True, if single consumer. */
	} common;

	/** Ring producer status. */
	__llring_cache_aligned struct{
		volatile uint32_t head;  /**< Producer head. */
		volatile uint32_t tail;  /**< Producer tail. */
	} prod;

	/** Ring consumer status. */
	__llring_cache_aligned struct{
		volatile uint32_t head;  /**< Consumer head. */
		volatile uint32_t tail;  /**< Consumer tail. */
	} cons;

#if 0 && LLRING_ENABLE_DEBUG
	struct llring_debug_stats stats[LLRING_MAX_CORES];
#endif

	/* it seems to help */
	char _pad[LLRING_CACHELINE_SIZE];

	__llring_cache_aligned void *ring[0]; /**< Memory space of ring starts here.
	 	 	 	 	 	 * not volatile so need to be careful
	 	 	 	 	 	 * about compiler re-ordering */
};

#define RING_QUOT_EXCEED (1 << 31)  /**< Quota exceed for burst ops */
#define RING_SZ_MASK  (unsigned)(0x0fffffff) /**< Ring slots mask */

/**
 * @internal When debug is enabled, store ring statistics.
 * @param r
 *   A pointer to the ring.
 * @param name
 *   The name of the statistics field to increment in the ring.
 * @param n
 *   The number to add to the object-oriented statistics.
 */
#if 0 && LLRING_ENABLE_DEBUG
#define __RING_STAT_ADD(r, name, n) do {		\
		unsigned __lcore_id = llring_cpu_id();	\
		r->stats[__lcore_id].name##_objs += n;	\
		r->stats[__lcore_id].name##_bulk += 1;	\
	} while(0)
#else
#define __RING_STAT_ADD(r, name, n) do {} while(0)
#endif

LLRING_API int llring_bytes_with_slots(unsigned int slots);
LLRING_API int llring_bytes(struct llring *r);
LLRING_API int llring_init(struct llring *r, unsigned int slots, int sp, int sc);
LLRING_API int llring_set_water_mark(struct llring *r, unsigned count);
LLRING_API int llring_enqueue_bulk(struct llring *r, void * const *obj_table, unsigned n);
LLRING_API int llring_enqueue(struct llring *r, void *obj);
LLRING_API int llring_dequeue_burst(struct llring *r, void **obj_table, unsigned n);
LLRING_API struct llring* llring_alloc_and_init(unsigned int slots, int sp, int sc);
LLRING_API int free_ring(struct llring* ring);
#endif
