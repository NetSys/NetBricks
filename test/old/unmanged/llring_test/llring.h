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

#ifndef _LLRING_H_
#define _LLRING_H_

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

#define __llring_cache_aligned \
		__attribute__((__aligned__(LLRING_CACHELINE_SIZE)))

#ifdef __KERNEL__

#define llring_likely(x)	likely(x)
#define llring_unlikely(x)	unlikely(x)

static inline void llring_pause(void)	{ cpu_relax(); }

#else

#include <stdint.h>

#define llring_likely(x)      __builtin_expect(!!(x), 1)
#define llring_unlikely(x)    __builtin_expect(!!(x), 0)

#include <emmintrin.h>

static inline void llring_pause(void)	{ _mm_pause(); }

#endif

/* dummy assembly operation to prevent compiler re-ordering of instructions */
#define COMPILER_BARRIER() do { asm volatile("" ::: "memory"); } while(0)

static inline int 
llring_atomic32_cmpset(volatile uint32_t *dst, uint32_t exp, uint32_t src)
{
	return __sync_bool_compare_and_swap(dst, exp, src);
}

enum llring_queue_behavior {
	LLRING_QUEUE_FIXED = 0, /* Enq/Deq a fixed number of items from a ring */
	LLRING_QUEUE_VARIABLE   /* Enq/Deq as many items a possible from ring */
};

#if LLRING_ENABLE_DEBUG
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
struct llring {
	/** Mostly read-only status */
	struct {
		uint32_t slots;          /**< Size of ring. */
		uint32_t mask;           /**< Mask (slots-1) of ring. */
		uint32_t watermark;      /**< Maximum items before LLRING_ERR_QUOT. */
		uint32_t sp_enqueue;     /**< True, if single producer. */
		uint32_t sc_dequeue;     /**< True, if single consumer. */
	} common __llring_cache_aligned;

	/** Ring producer status. */
	struct {
		volatile uint32_t head;  /**< Producer head. */
		volatile uint32_t tail;  /**< Producer tail. */
	} prod __llring_cache_aligned;

	/** Ring consumer status. */
	struct {
		volatile uint32_t head;  /**< Consumer head. */
		volatile uint32_t tail;  /**< Consumer tail. */
	} cons __llring_cache_aligned;

#if LLRING_ENABLE_DEBUG
	struct llring_debug_stats stats[LLRING_MAX_CORES];
#endif

	/* it seems to help */
	char _pad[LLRING_CACHELINE_SIZE];

	void *ring[0] __llring_cache_aligned; /**< Memory space of ring starts here.
	 	 	 	 	 	 * not volatile so need to be careful
	 	 	 	 	 	 * about compiler re-ordering */
} __llring_cache_aligned;

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
#if LLRING_ENABLE_DEBUG
#define __RING_STAT_ADD(r, name, n) do {		\
		unsigned __lcore_id = llring_cpu_id();	\
		r->stats[__lcore_id].name##_objs += n;	\
		r->stats[__lcore_id].name##_bulk += 1;	\
	} while(0)
#else
#define __RING_STAT_ADD(r, name, n) do {} while(0)
#endif

static inline int llring_bytes_with_slots(unsigned int slots)
{
	return sizeof(struct llring) + sizeof(void *) * slots;
}

static inline int llring_bytes(struct llring *r)
{
	return llring_bytes_with_slots(r->common.slots);
}

static inline int 
llring_init(struct llring *r, unsigned int slots, int sp, int sc)
{
	char *p = (char *)r;
	int i;

	/* slots must be a power of 2 */
	if (slots & (slots - 1))
		return -LLRING_ERR_NOPOW2;

	/* poor man's memset */
	for (i = 0; i < llring_bytes_with_slots(slots); i++)
		p[i] = 0;
	
	r->common.slots = slots;
	r->common.mask = slots - 1;
	r->common.watermark = slots;
	r->common.sp_enqueue = !!sp;
	r->common.sc_dequeue = !!sc;

	r->prod.head = r->cons.head = 0;
	r->prod.tail = r->cons.tail = 0;

	return 0;
}

/**
 * Change the high water mark.
 *
 * If *count* is 0, water marking is disabled. Otherwise, it is set to the
 * *count* value. The *count* value must be greater than 0 and less
 * than the ring slots.
 *
 * This function can be called at any time (not necessarily at
 * initialization).
 *
 * @param r
 *   A pointer to the ring structure.
 * @param count
 *   The new water mark value.
 * @return
 *   - 0: Success; water mark changed.
 *   - -LLRING_ERR_INVAL: Invalid water mark value.
 */
static inline int llring_set_water_mark(struct llring *r, unsigned count)
{
	if (count >= r->common.slots)
		return -LLRING_ERR_INVAL;

	/* if count is 0, disable the watermarking */
	if (count == 0)
		count = r->common.slots;

	r->common.watermark = count;
	return 0;
}


/* the actual enqueue of pointers on the ring. 
 * Placed here since identical code needed in both
 * single and multi producer enqueue functions */
#define LLRING_ENQUEUE_PTRS() do { \
	const uint32_t slots = r->common.slots; \
	uint32_t idx = prod_head & mask; \
	if (llring_likely(idx + n < slots)) { \
		for (i = 0; i < (n & ((~(unsigned)0x3))); i+=4, idx+=4) { \
			r->ring[idx] = obj_table[i]; \
			r->ring[idx+1] = obj_table[i+1]; \
			r->ring[idx+2] = obj_table[i+2]; \
			r->ring[idx+3] = obj_table[i+3]; \
		} \
		switch (n & 0x3) { \
			case 3: r->ring[idx++] = obj_table[i++]; \
			case 2: r->ring[idx++] = obj_table[i++]; \
			case 1: r->ring[idx++] = obj_table[i++]; \
		} \
	} else { \
		for (i = 0; idx < slots; i++, idx++)\
			r->ring[idx] = obj_table[i]; \
		for (idx = 0; i < n; i++, idx++) \
			r->ring[idx] = obj_table[i]; \
	} \
} while(0)

/* the actual copy of pointers on the ring to obj_table. 
 * Placed here since identical code needed in both
 * single and multi consumer dequeue functions */
#define LLRING_DEQUEUE_PTRS() do { \
	uint32_t idx = cons_head & mask; \
	const uint32_t slots = r->common.slots; \
	if (llring_likely(idx + n < slots)) { \
		for (i = 0; i < (n & (~(unsigned)0x3)); i+=4, idx+=4) {\
			obj_table[i] = r->ring[idx]; \
			obj_table[i+1] = r->ring[idx+1]; \
			obj_table[i+2] = r->ring[idx+2]; \
			obj_table[i+3] = r->ring[idx+3]; \
		} \
		switch (n & 0x3) { \
			case 3: obj_table[i++] = r->ring[idx++]; \
			case 2: obj_table[i++] = r->ring[idx++]; \
			case 1: obj_table[i++] = r->ring[idx++]; \
		} \
	} else { \
		for (i = 0; idx < slots; i++, idx++) \
			obj_table[i] = r->ring[idx]; \
		for (idx = 0; i < n; i++, idx++) \
			obj_table[i] = r->ring[idx]; \
	} \
} while (0)

/**
 * @internal Enqueue several objects on the ring (multi-producers safe).
 *
 * This function uses a "compare and set" instruction to move the
 * producer index atomically.
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects).
 * @param n
 *   The number of objects to add in the ring from the obj_table.
 * @param behavior
 *   LLRING_QUEUE_FIXED:    Enqueue a fixed number of items from a ring
 *   LLRING_QUEUE_VARIABLE: Enqueue as many items a possible from ring
 * @return
 *   Depend on the behavior value
 *   if behavior = LLRING_QUEUE_FIXED
 *   - 0: Success; objects enqueue.
 *   - -LLRING_ERR_QUOT: Quota exceeded. The objects have been enqueued, but the
 *     high water mark is exceeded.
 *   - -LLRING_ERR_NOBUF: Not enough room in the ring to enqueue, no object is enqueued.
 *   if behavior = LLRING_QUEUE_VARIABLE
 *   - n: Actual number of objects enqueued.
 */
static inline int __attribute__((always_inline))
__llring_mp_do_enqueue(struct llring *r, void * const *obj_table,
			 unsigned n, enum llring_queue_behavior behavior)
{
	uint32_t prod_head, prod_next;
	uint32_t cons_tail, free_entries;
	const unsigned max = n;
	int success;
	unsigned i;
	uint32_t mask = r->common.mask;
	int ret;

	/* move prod.head atomically */
	do {
		/* Reset n to the initial burst count */
		n = max;

		prod_head = r->prod.head;
		cons_tail = r->cons.tail;
		/* The subtraction is done between two unsigned 32bits value
		 * (the result is always modulo 32 bits even if we have
		 * prod_head > cons_tail). So 'free_entries' is always between 0
		 * and slots(ring)-1. */
		free_entries = (mask + cons_tail - prod_head);

		/* check that we have enough room in ring */
		if (llring_unlikely(n > free_entries)) {
			if (behavior == LLRING_QUEUE_FIXED) {
				__RING_STAT_ADD(r, enq_fail, n);
				return -LLRING_ERR_NOBUF;
			}
			else {
				/* No free entry available */
				if (free_entries == 0) {
					__RING_STAT_ADD(r, enq_fail, n);
					return 0;
				}

				n = free_entries;
			}
		}

		prod_next = prod_head + n;
		success = llring_atomic32_cmpset(&r->prod.head, prod_head, 
				prod_next);
	} while (llring_unlikely(success == 0));

	/* write entries in ring */
	LLRING_ENQUEUE_PTRS();
	COMPILER_BARRIER();

	/* if we exceed the watermark */
	if (llring_unlikely(((mask + 1) - free_entries + n) > r->common.watermark)) {
		ret = (behavior == LLRING_QUEUE_FIXED) ? -LLRING_ERR_QUOT :
				(int)(n | RING_QUOT_EXCEED);
		__RING_STAT_ADD(r, enq_quota, n);
	}
	else {
		ret = (behavior == LLRING_QUEUE_FIXED) ? 0 : n;
		__RING_STAT_ADD(r, enq_success, n);
	}

	/*
	 * If there are other enqueues in progress that preceeded us,
	 * we need to wait for them to complete
	 */
	while (llring_unlikely(r->prod.tail != prod_head))
		llring_pause();

	r->prod.tail = prod_next;
	return ret;
}

/**
 * @internal Enqueue several objects on a ring (NOT multi-producers safe).
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects).
 * @param n
 *   The number of objects to add in the ring from the obj_table.
 * @param behavior
 *   LLRING_QUEUE_FIXED:    Enqueue a fixed number of items from a ring
 *   LLRING_QUEUE_VARIABLE: Enqueue as many items a possible from ring
 * @return
 *   Depend on the behavior value
 *   if behavior = LLRING_QUEUE_FIXED
 *   - 0: Success; objects enqueue.
 *   - -LLRING_ERR_QUOT: Quota exceeded. The objects have been enqueued, but the
 *     high water mark is exceeded.
 *   - -LLRING_ERR_NOBUF: Not enough room in the ring to enqueue, no object is enqueued.
 *   if behavior = LLRING_QUEUE_VARIABLE
 *   - n: Actual number of objects enqueued.
 */
static inline int __attribute__((always_inline))
__llring_sp_do_enqueue(struct llring *r, void * const *obj_table,
			 unsigned n, enum llring_queue_behavior behavior)
{
	uint32_t prod_head, cons_tail;
	uint32_t prod_next, free_entries;
	unsigned i;
	uint32_t mask = r->common.mask;
	int ret;

	prod_head = r->prod.head;
	cons_tail = r->cons.tail;
	/* The subtraction is done between two unsigned 32bits value
	 * (the result is always modulo 32 bits even if we have
	 * prod_head > cons_tail). So 'free_entries' is always between 0
	 * and slots(ring)-1. */
	free_entries = mask + cons_tail - prod_head;

	/* check that we have enough room in ring */
	if (llring_unlikely(n > free_entries)) {
		if (behavior == LLRING_QUEUE_FIXED) {
			__RING_STAT_ADD(r, enq_fail, n);
			return -LLRING_ERR_NOBUF;
		}
		else {
			/* No free entry available */
			if (free_entries == 0) {
				__RING_STAT_ADD(r, enq_fail, n);
				return 0;
			}

			n = free_entries;
		}
	}

	prod_next = prod_head + n;
	r->prod.head = prod_next;

	/* write entries in ring */
	LLRING_ENQUEUE_PTRS();
	COMPILER_BARRIER();

	/* if we exceed the watermark */
	if (llring_unlikely(((mask + 1) - free_entries + n) > r->common.watermark)) {
		ret = (behavior == LLRING_QUEUE_FIXED) ? -LLRING_ERR_QUOT :
			(int)(n | RING_QUOT_EXCEED);
		__RING_STAT_ADD(r, enq_quota, n);
	}
	else {
		ret = (behavior == LLRING_QUEUE_FIXED) ? 0 : n;
		__RING_STAT_ADD(r, enq_success, n);
	}

	r->prod.tail = prod_next;
	return ret;
}

/**
 * @internal Dequeue several objects from a ring (multi-consumers safe). When
 * the request objects are more than the available objects, only dequeue the
 * actual number of objects
 *
 * This function uses a "compare and set" instruction to move the
 * consumer index atomically.
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects) that will be filled.
 * @param n
 *   The number of objects to dequeue from the ring to the obj_table.
 * @param behavior
 *   LLRING_QUEUE_FIXED:    Dequeue a fixed number of items from a ring
 *   LLRING_QUEUE_VARIABLE: Dequeue as many items a possible from ring
 * @return
 *   Depend on the behavior value
 *   if behavior = LLRING_QUEUE_FIXED
 *   - 0: Success; objects dequeued.
 *   - -LLRING_ERR_NOENT: Not enough entries in the ring to dequeue; no object is
 *     dequeued.
 *   if behavior = LLRING_QUEUE_VARIABLE
 *   - n: Actual number of objects dequeued.
 */

static inline int __attribute__((always_inline))
__llring_mc_do_dequeue(struct llring *r, void **obj_table,
		 unsigned n, enum llring_queue_behavior behavior)
{
	uint32_t cons_head, prod_tail;
	uint32_t cons_next, entries;
	const unsigned max = n;
	int success;
	unsigned i;
	uint32_t mask = r->common.mask;

	/* move cons.head atomically */
	do {
		/* Restore n as it may change every loop */
		n = max;

		cons_head = r->cons.head;
		prod_tail = r->prod.tail;
		/* The subtraction is done between two unsigned 32bits value
		 * (the result is always modulo 32 bits even if we have
		 * cons_head > prod_tail). So 'entries' is always between 0
		 * and slots(ring)-1. */
		entries = (prod_tail - cons_head);

		/* Set the actual entries for dequeue */
		if (n > entries) {
			if (behavior == LLRING_QUEUE_FIXED) {
				__RING_STAT_ADD(r, deq_fail, n);
				return -LLRING_ERR_NOENT;
			}
			else {
				if (entries == 0) {
					__RING_STAT_ADD(r, deq_fail, n);
					return 0;
				}

				n = entries;
			}
		}

		cons_next = cons_head + n;
		success = llring_atomic32_cmpset(&r->cons.head, cons_head,
				cons_next);
	} while (llring_unlikely(success == 0));

	/* copy in table */
	LLRING_DEQUEUE_PTRS();
	COMPILER_BARRIER();

	/*
	 * If there are other dequeues in progress that preceded us,
	 * we need to wait for them to complete
	 */
	while (llring_unlikely(r->cons.tail != cons_head))
		llring_pause();

	__RING_STAT_ADD(r, deq_success, n);
	r->cons.tail = cons_next;

	return behavior == LLRING_QUEUE_FIXED ? 0 : n;
}

/**
 * @internal Dequeue several objects from a ring (NOT multi-consumers safe).
 * When the request objects are more than the available objects, only dequeue
 * the actual number of objects
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects) that will be filled.
 * @param n
 *   The number of objects to dequeue from the ring to the obj_table.
 * @param behavior
 *   LLRING_QUEUE_FIXED:    Dequeue a fixed number of items from a ring
 *   LLRING_QUEUE_VARIABLE: Dequeue as many items a possible from ring
 * @return
 *   Depend on the behavior value
 *   if behavior = LLRING_QUEUE_FIXED
 *   - 0: Success; objects dequeued.
 *   - -LLRING_ERR_NOENT: Not enough entries in the ring to dequeue; no object is
 *     dequeued.
 *   if behavior = LLRING_QUEUE_VARIABLE
 *   - n: Actual number of objects dequeued.
 */
static inline int __attribute__((always_inline))
__llring_sc_do_dequeue(struct llring *r, void **obj_table,
		 unsigned n, enum llring_queue_behavior behavior)
{
	uint32_t cons_head, prod_tail;
	uint32_t cons_next, entries;
	unsigned i;
	uint32_t mask = r->common.mask;

	cons_head = r->cons.head;
	prod_tail = r->prod.tail;
	/* The subtraction is done between two unsigned 32bits value
	 * (the result is always modulo 32 bits even if we have
	 * cons_head > prod_tail). So 'entries' is always between 0
	 * and slots(ring)-1. */
	entries = prod_tail - cons_head;

	if (n > entries) {
		if (behavior == LLRING_QUEUE_FIXED) {
			__RING_STAT_ADD(r, deq_fail, n);
			return -LLRING_ERR_NOENT;
		}
		else {
			if (entries == 0) {
				__RING_STAT_ADD(r, deq_fail, n);
				return 0;
			}

			n = entries;
		}
	}

	cons_next = cons_head + n;
	r->cons.head = cons_next;

	/* copy in table */
	LLRING_DEQUEUE_PTRS();
	COMPILER_BARRIER();

	__RING_STAT_ADD(r, deq_success, n);
	r->cons.tail = cons_next;
	return behavior == LLRING_QUEUE_FIXED ? 0 : n;
}

/**
 * Enqueue several objects on the ring (multi-producers safe).
 *
 * This function uses a "compare and set" instruction to move the
 * producer index atomically.
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects).
 * @param n
 *   The number of objects to add in the ring from the obj_table.
 * @return
 *   - 0: Success; objects enqueue.
 *   - -LLRING_ERR_QUOT: Quota exceeded. The objects have been enqueued, but the
 *     high water mark is exceeded.
 *   - -LLRING_ERR_NOBUF: Not enough room in the ring to enqueue, no object is enqueued.
 */
static inline int __attribute__((always_inline))
llring_mp_enqueue_bulk(struct llring *r, void * const *obj_table,
			 unsigned n)
{
	return __llring_mp_do_enqueue(r, obj_table, n, LLRING_QUEUE_FIXED);
}

static inline int __attribute__((always_inline))
llring_sp_enqueue_bulk(struct llring *r, void * const *obj_table,
			 unsigned n)
{
	return __llring_sp_do_enqueue(r, obj_table, n, LLRING_QUEUE_FIXED);
}

static inline int __attribute__((always_inline))
llring_enqueue_bulk(struct llring *r, void * const *obj_table,
		      unsigned n)
{
	if (r->common.sp_enqueue)
		return llring_sp_enqueue_bulk(r, obj_table, n);
	else
		return llring_mp_enqueue_bulk(r, obj_table, n);
}

static inline int __attribute__((always_inline))
llring_mp_enqueue(struct llring *r, void *obj)
{
	return llring_mp_enqueue_bulk(r, &obj, 1);
}

/**
 * Enqueue one object on a ring (NOT multi-producers safe).
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj
 *   A pointer to the object to be added.
 * @return
 *   - 0: Success; objects enqueued.
 *   - -LLRING_ERR_QUOT: Quota exceeded. The objects have been enqueued, but the
 *     high water mark is exceeded.
 *   - -LLRING_ERR_NOBUF: Not enough room in the ring to enqueue; no object is enqueued.
 */
static inline int __attribute__((always_inline))
llring_sp_enqueue(struct llring *r, void *obj)
{
	return llring_sp_enqueue_bulk(r, &obj, 1);
}

/**
 * Enqueue one object on a ring.
 *
 * This function calls the multi-producer or the single-producer
 * version, depending on the default behaviour that was specified at
 * ring creation time (see flags).
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj
 *   A pointer to the object to be added.
 * @return
 *   - 0: Success; objects enqueued.
 *   - -LLRING_ERR_QUOT: Quota exceeded. The objects have been enqueued, but the
 *     high water mark is exceeded.
 *   - -LLRING_ERR_NOBUF: Not enough room in the ring to enqueue; no object is enqueued.
 */
static inline int __attribute__((always_inline))
llring_enqueue(struct llring *r, void *obj)
{
	if (r->common.sp_enqueue)
		return llring_sp_enqueue(r, obj);
	else
		return llring_mp_enqueue(r, obj);
}

/**
 * Dequeue several objects from a ring (multi-consumers safe).
 *
 * This function uses a "compare and set" instruction to move the
 * consumer index atomically.
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects) that will be filled.
 * @param n
 *   The number of objects to dequeue from the ring to the obj_table.
 * @return
 *   - 0: Success; objects dequeued.
 *   - -LLRING_ERR_NOENT: Not enough entries in the ring to dequeue; no object is
 *     dequeued.
 */
static inline int __attribute__((always_inline))
llring_mc_dequeue_bulk(struct llring *r, void **obj_table, unsigned n)
{
	return __llring_mc_do_dequeue(r, obj_table, n, LLRING_QUEUE_FIXED);
}

/**
 * Dequeue several objects from a ring (NOT multi-consumers safe).
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects) that will be filled.
 * @param n
 *   The number of objects to dequeue from the ring to the obj_table,
 *   must be strictly positive.
 * @return
 *   - 0: Success; objects dequeued.
 *   - -LLRING_ERR_NOENT: Not enough entries in the ring to dequeue; no object is
 *     dequeued.
 */
static inline int __attribute__((always_inline))
llring_sc_dequeue_bulk(struct llring *r, void **obj_table, unsigned n)
{
	return __llring_sc_do_dequeue(r, obj_table, n, LLRING_QUEUE_FIXED);
}

/**
 * Dequeue several objects from a ring.
 *
 * This function calls the multi-consumers or the single-consumer
 * version, depending on the default behaviour that was specified at
 * ring creation time (see flags).
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects) that will be filled.
 * @param n
 *   The number of objects to dequeue from the ring to the obj_table.
 * @return
 *   - 0: Success; objects dequeued.
 *   - -LLRING_ERR_NOENT: Not enough entries in the ring to dequeue, no object is
 *     dequeued.
 */
static inline int __attribute__((always_inline))
llring_dequeue_bulk(struct llring *r, void **obj_table, unsigned n)
{
	if (r->common.sc_dequeue)
		return llring_sc_dequeue_bulk(r, obj_table, n);
	else
		return llring_mc_dequeue_bulk(r, obj_table, n);
}

/**
 * Dequeue one object from a ring (multi-consumers safe).
 *
 * This function uses a "compare and set" instruction to move the
 * consumer index atomically.
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_p
 *   A pointer to a void * pointer (object) that will be filled.
 * @return
 *   - 0: Success; objects dequeued.
 *   - -LLRING_ERR_NOENT: Not enough entries in the ring to dequeue; no object is
 *     dequeued.
 */
static inline int __attribute__((always_inline))
llring_mc_dequeue(struct llring *r, void **obj_p)
{
	return llring_mc_dequeue_bulk(r, obj_p, 1);
}

/**
 * Dequeue one object from a ring (NOT multi-consumers safe).
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_p
 *   A pointer to a void * pointer (object) that will be filled.
 * @return
 *   - 0: Success; objects dequeued.
 *   - -LLRING_ERR_NOENT: Not enough entries in the ring to dequeue, no object is
 *     dequeued.
 */
static inline int __attribute__((always_inline))
llring_sc_dequeue(struct llring *r, void **obj_p)
{
	return llring_sc_dequeue_bulk(r, obj_p, 1);
}

/**
 * Dequeue one object from a ring.
 *
 * This function calls the multi-consumers or the single-consumer
 * version depending on the default behaviour that was specified at
 * ring creation time (see flags).
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_p
 *   A pointer to a void * pointer (object) that will be filled.
 * @return
 *   - 0: Success, objects dequeued.
 *   - -LLRING_ERR_NOENT: Not enough entries in the ring to dequeue, no object is
 *     dequeued.
 */
static inline int __attribute__((always_inline))
llring_dequeue(struct llring *r, void **obj_p)
{
	if (r->common.sc_dequeue)
		return llring_sc_dequeue(r, obj_p);
	else
		return llring_mc_dequeue(r, obj_p);
}

/**
 * Test if a ring is full.
 *
 * @param r
 *   A pointer to the ring structure.
 * @return
 *   - 1: The ring is full.
 *   - 0: The ring is not full.
 */
static inline int
llring_full(const struct llring *r)
{
	uint32_t prod_tail = r->prod.tail;
	uint32_t cons_tail = r->cons.tail;
	return (((cons_tail - prod_tail - 1) & r->common.mask) == 0);
}

/**
 * Test if a ring is empty.
 *
 * @param r
 *   A pointer to the ring structure.
 * @return
 *   - 1: The ring is empty.
 *   - 0: The ring is not empty.
 */
static inline int
llring_empty(const struct llring *r)
{
	uint32_t prod_tail = r->prod.tail;
	uint32_t cons_tail = r->cons.tail;
	return !!(cons_tail == prod_tail);
}

/**
 * Return the number of entries in a ring.
 *
 * @param r
 *   A pointer to the ring structure.
 * @return
 *   The number of entries in the ring.
 */
static inline unsigned
llring_count(const struct llring *r)
{
	uint32_t prod_tail = r->prod.tail;
	uint32_t cons_tail = r->cons.tail;
	return ((prod_tail - cons_tail) & r->common.mask);
}

/**
 * Return the number of free entries in a ring.
 *
 * @param r
 *   A pointer to the ring structure.
 * @return
 *   The number of free entries in the ring.
 */
static inline unsigned
llring_free_count(const struct llring *r)
{
	uint32_t prod_tail = r->prod.tail;
	uint32_t cons_tail = r->cons.tail;
	return ((cons_tail - prod_tail - 1) & r->common.mask);
}

/**
 * Enqueue several objects on the ring (multi-producers safe).
 *
 * This function uses a "compare and set" instruction to move the
 * producer index atomically.
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects).
 * @param n
 *   The number of objects to add in the ring from the obj_table.
 * @return
 *   - n: Actual number of objects enqueued.
 */
static inline int __attribute__((always_inline))
llring_mp_enqueue_burst(struct llring *r, void * const *obj_table,
			 unsigned n)
{
	return __llring_mp_do_enqueue(r, obj_table, n, LLRING_QUEUE_VARIABLE);
}

/**
 * Enqueue several objects on a ring (NOT multi-producers safe).
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects).
 * @param n
 *   The number of objects to add in the ring from the obj_table.
 * @return
 *   - n: Actual number of objects enqueued.
 */
static inline int __attribute__((always_inline))
llring_sp_enqueue_burst(struct llring *r, void * const *obj_table,
			 unsigned n)
{
	return __llring_sp_do_enqueue(r, obj_table, n, LLRING_QUEUE_VARIABLE);
}

/**
 * Enqueue several objects on a ring.
 *
 * This function calls the multi-producer or the single-producer
 * version depending on the default behavior that was specified at
 * ring creation time (see flags).
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects).
 * @param n
 *   The number of objects to add in the ring from the obj_table.
 * @return
 *   - n: Actual number of objects enqueued.
 */
static inline int __attribute__((always_inline))
llring_enqueue_burst(struct llring *r, void * const *obj_table,
		      unsigned n)
{
	if (r->common.sp_enqueue)
		return 	llring_sp_enqueue_burst(r, obj_table, n);
	else
		return 	llring_mp_enqueue_burst(r, obj_table, n);
}

/**
 * Dequeue several objects from a ring (multi-consumers safe). When the request
 * objects are more than the available objects, only dequeue the actual number
 * of objects
 *
 * This function uses a "compare and set" instruction to move the
 * consumer index atomically.
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects) that will be filled.
 * @param n
 *   The number of objects to dequeue from the ring to the obj_table.
 * @return
 *   - n: Actual number of objects dequeued, 0 if ring is empty
 */
static inline int __attribute__((always_inline))
llring_mc_dequeue_burst(struct llring *r, void **obj_table, unsigned n)
{
	return __llring_mc_do_dequeue(r, obj_table, n, LLRING_QUEUE_VARIABLE);
}

/**
 * Dequeue several objects from a ring (NOT multi-consumers safe).When the
 * request objects are more than the available objects, only dequeue the
 * actual number of objects
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects) that will be filled.
 * @param n
 *   The number of objects to dequeue from the ring to the obj_table.
 * @return
 *   - n: Actual number of objects dequeued, 0 if ring is empty
 */
static inline int __attribute__((always_inline))
llring_sc_dequeue_burst(struct llring *r, void **obj_table, unsigned n)
{
	return __llring_sc_do_dequeue(r, obj_table, n, LLRING_QUEUE_VARIABLE);
}

/**
 * Dequeue multiple objects from a ring up to a maximum number.
 *
 * This function calls the multi-consumers or the single-consumer
 * version, depending on the default behaviour that was specified at
 * ring creation time (see flags).
 *
 * @param r
 *   A pointer to the ring structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects) that will be filled.
 * @param n
 *   The number of objects to dequeue from the ring to the obj_table.
 * @return
 *   - Number of objects dequeued, or a negative error code on error
 */
static inline int __attribute__((always_inline))
llring_dequeue_burst(struct llring *r, void **obj_table, unsigned n)
{
	if (r->common.sc_dequeue)
		return llring_sc_dequeue_burst(r, obj_table, n);
	else
		return llring_mc_dequeue_burst(r, obj_table, n);
}

#endif /* _LLRING_H_ */
