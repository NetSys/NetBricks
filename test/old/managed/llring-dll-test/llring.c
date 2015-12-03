#include "llring.h"
#include <stdio.h>
#ifdef _MSC_VER
#define INLINE_ATTRIBUTE __forceinline
#else
#define INLINE_ATTRIBUTE __attribute__((always_inline))
#endif
int llring_bytes_with_slots(unsigned int slots)
{
	return sizeof(struct llring) + sizeof(void *) * slots;
}

int llring_bytes(struct llring *r)
{
	return llring_bytes_with_slots(r->common.slots);
}

int 
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
int llring_set_water_mark(struct llring *r, unsigned count)
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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

static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
llring_mp_enqueue_bulk(struct llring *r, void * const *obj_table,
			 unsigned n)
{
	return __llring_mp_do_enqueue(r, obj_table, n, LLRING_QUEUE_FIXED);
}

static  int INLINE_ATTRIBUTE
llring_sp_enqueue_bulk(struct llring *r, void * const *obj_table,
			 unsigned n)
{
	return __llring_sp_do_enqueue(r, obj_table, n, LLRING_QUEUE_FIXED);
}

int llring_enqueue_bulk(struct llring *r, void * const *obj_table,
		      unsigned n)
{
	if (r->common.sp_enqueue)
		return llring_sp_enqueue_bulk(r, obj_table, n);
	else
		return llring_mp_enqueue_bulk(r, obj_table, n);
}

static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
int llring_enqueue(struct llring *r, void *obj)
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
static  int INLINE_ATTRIBUTE
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
int llring_dequeue_burst(struct llring *r, void **obj_table, unsigned n)
{
	if (r->common.sc_dequeue)
		return llring_sc_dequeue_burst(r, obj_table, n);
	else
		return llring_mc_dequeue_burst(r, obj_table, n);
}

struct llring* llring_alloc_and_init(unsigned int slots, int sp, int sc) {
	struct llring* r = (struct llring*)malloc(llring_bytes_with_slots(slots));
	llring_init(r, slots, sp, sc);
	return r;
}

int free_ring(struct llring* ring) {
	free(ring);
	return 1;
}
