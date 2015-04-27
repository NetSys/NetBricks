#define _GNU_SOURCE

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#if _MSC_VER
#include <Windows.h>
#else
#include <sys/time.h>
#endif

#include <pthread.h>

#include "llring.h"

#define SLOTS_PER_LLRING 256
#define BATCH_SIZE 32

static uint64_t get_sec(void)
{
#if _MSC_VER
	return timeGetTime() / 1000;
#else
	struct timeval time;
	gettimeofday(&time, NULL);
	//long millis = (time.tv_sec * 1000) + (time.tv_usec / 1000);
	return (uint64_t)time.tv_sec;
#endif
}


static void *consumer(void * arg)
{
	void *buffers[SLOTS_PER_LLRING];
	uint64_t count = 0;
	uint64_t prev_time = get_sec();
	struct llring *ring = (struct llring*)arg;

	while (1) {
		int ret;
		ret = llring_dequeue_burst(ring, buffers, BATCH_SIZE);
		count += ret;

		uint64_t cur_time = get_sec();
		if (prev_time != cur_time) {
			printf("%lu %lu\n", cur_time - prev_time, count / (cur_time - prev_time));
			count = 0;
			prev_time = cur_time;
		}
	}
}

static void *producer(void*arg)
{
	void *buffers[SLOTS_PER_LLRING];
	struct llring *ring = (struct llring*)arg;
	memset(buffers, 1, sizeof(buffers));
	
	while (1) {
		llring_enqueue_bulk(ring, buffers, BATCH_SIZE); 
	}
}

int main(int argc, char* argv[])
{

	struct llring *ring = malloc(llring_bytes_with_slots(SLOTS_PER_LLRING));
	pthread_attr_t attr;
	pthread_t prod, cons;
	int s;
	void *res;
	cpu_set_t prod_cpu_set, cons_cpu_set;

	s = pthread_attr_init(&attr);

	llring_init(ring, SLOTS_PER_LLRING,  0, 0);

	pthread_create(&cons, &attr, &consumer, ring);
	pthread_create(&prod, &attr, &producer, ring);

	CPU_ZERO(&cons_cpu_set);
	CPU_SET(1, &cons_cpu_set);
	pthread_setaffinity_np(cons, sizeof(cons_cpu_set), &cons_cpu_set); 

	CPU_ZERO(&prod_cpu_set);
	CPU_SET(3, &prod_cpu_set);
	pthread_setaffinity_np(prod, sizeof(prod_cpu_set), &prod_cpu_set); 

	pthread_join(prod, &res);
	pthread_join(cons, &res);
}
