#define _GNU_SOURCE

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#if _MSC_VER
#include <Windows.h>
#else
#include <sys/time.h>
#endif

#include <pthread.h>

#include "llring.h"

static int PROD_CORE_ID = 1;
static int CONS_CORE_ID = 3;
static int SLOTS_PER_LLRING = 256;
static int BATCH_SIZE = 32;

static int WARMUP_TIME = 2;
static int MEASURE_TIME = 5;

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
	void *buffers = malloc(SLOTS_PER_LLRING * sizeof(void*));
	uint64_t count = 0;
	uint64_t prev_time = get_sec();
	struct llring *ring = (struct llring*)arg;

	uint64_t measure_count = 0;
	uint64_t measure_sec = 0;

	while (1) {
		int ret;
		ret = llring_dequeue_burst(ring, buffers, BATCH_SIZE);
		count += ret;

		uint64_t cur_time = get_sec();
		if (prev_time != cur_time) {
			//printf("%lu %lu\n", cur_time - prev_time, count / (cur_time - prev_time));
			measure_sec++;
			if (measure_sec > WARMUP_TIME)
				measure_count += count;

			count = 0;
			prev_time = cur_time;

			if (measure_sec > WARMUP_TIME + MEASURE_TIME) {
				printf("%d %d %lu\n", SLOTS_PER_LLRING, BATCH_SIZE, 
				       measure_count / MEASURE_TIME);
				exit(0);
			}
		}
	}
}

static void *producer(void*arg)
{
	void *buffers = malloc(SLOTS_PER_LLRING * sizeof(void*));
	struct llring *ring = (struct llring*)arg;
	memset(buffers, 1, sizeof(buffers));
	
	while (1) {
		llring_enqueue_bulk(ring, buffers, BATCH_SIZE); 
	}
}

void parse_arg(int argc, char* argv[])
{
	int opt;
	while ((opt = getopt(argc, argv, "p:c:s:b:")) != -1) {
		switch(opt) {
		case 'p':
			PROD_CORE_ID = atoi(optarg);
			break;
		case 'c':
			CONS_CORE_ID = atoi(optarg);
			break;
		case 's':
			SLOTS_PER_LLRING = atoi(optarg);
			break;
		case 'b':
			BATCH_SIZE = atoi(optarg);
			break;
		default:
			fprintf(stderr, "wrong argument\n");
			exit(-1);
			break;
		}
	}
}

int main(int argc, char* argv[])
{

	struct llring *ring;
	pthread_attr_t attr;
	pthread_t prod, cons;
	int s;
	void *res;
	cpu_set_t prod_cpu_set, cons_cpu_set;

	parse_arg(argc, argv);

	ring = malloc(llring_bytes_with_slots(SLOTS_PER_LLRING));
	s = pthread_attr_init(&attr);

	llring_init(ring, SLOTS_PER_LLRING,  0, 0);

	pthread_create(&cons, &attr, &consumer, ring);
	pthread_create(&prod, &attr, &producer, ring);

	CPU_ZERO(&cons_cpu_set);
	CPU_SET(PROD_CORE_ID, &cons_cpu_set);
	pthread_setaffinity_np(cons, sizeof(cons_cpu_set), &cons_cpu_set); 

	CPU_ZERO(&prod_cpu_set);
	CPU_SET(CONS_CORE_ID, &prod_cpu_set);
	pthread_setaffinity_np(prod, sizeof(prod_cpu_set), &prod_cpu_set); 

	pthread_join(prod, &res);
	pthread_join(cons, &res);

	return 0;
}
