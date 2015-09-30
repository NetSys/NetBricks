#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <rte_config.h>
#include <rte_cycles.h>
#include <rte_timer.h>
#include <rte_ethdev.h>
#include <rte_eal.h>

#include "mempool.h"
/* Taken from SoftNIC (dpdk.c) */
/* Generate an lcore bitmap. For now only launch one worker */
static int set_lcore_bitmap(char *buf, int tid, int core)
{
	int off = 0;
	off = sprintf(buf, "%d@%d", tid, core);
	return off;
}

/* Get NUMA count */
static int get_numa_count()
{
	FILE *fp;

	int matched;
	int cnt;

	fp = fopen("/sys/devices/system/node/possible", "r");
	if (!fp)
		goto fail;

	matched = fscanf(fp, "0-%d", &cnt);
	if (matched == 1)
		return cnt + 1;

fail:
	if (fp)
		fclose(fp);

	fprintf(stderr, "Failed to detect # of NUMA nodes from: "
			"/sys/devices/system/node/possible. "
			"Assuming a single-node system...\n");
	return 1;
}

static int init_eal(int tid, int core)
{
	/* As opposed to SoftNIC, this call only initializes the master thread.
	 * We cannot rely on threads launched by DPDK within ZCSI, the threads
	 * must be launched by the runtime */
	int rte_argc = 0;
	char *rte_argv[16];

	char opt_master_lcore[1024];
	char opt_lcore_bitmap[1024];
	char opt_socket_mem[1024];

	const char *socket_mem = "2048";

	int numa_count = get_numa_count();

	int ret;
	int i;

	sprintf(opt_master_lcore, "%d", core);

	/* The actual lcore */
	/*i = set_lcore_bitmap(opt_lcore_bitmap, tid, core);*/
	sprintf(opt_lcore_bitmap, "0x%x", (RTE_MAX_LCORE - 1));
	printf("Core mask : %s\n", opt_lcore_bitmap);

	sprintf(opt_socket_mem, "%s", socket_mem);
	for(i = 1; i < numa_count; i++)
		sprintf(opt_socket_mem + strlen(opt_socket_mem), 
				",%s", socket_mem);

	rte_argv[rte_argc++] = "lzcsi";
	rte_argv[rte_argc++] = "-c";
	rte_argv[rte_argc++] = opt_lcore_bitmap;
	rte_argv[rte_argc++] = "--master-lcore";
	rte_argv[rte_argc++] = opt_master_lcore;
	rte_argv[rte_argc++] = "-n";
	/* number of memory channels (Sandy Bridge) */
	rte_argv[rte_argc++] = "4";	// Number of memory channels on 
					// Sandy Bridge.
	rte_argv[rte_argc++] = "--socket-mem";
	rte_argv[rte_argc++] = opt_socket_mem;
	rte_argv[rte_argc] = NULL;

	/* reset getopt() */
	optind = 0;

	ret = rte_eal_init(rte_argc, rte_argv);

	/* Change lcore ID */
	RTE_PER_LCORE(_lcore_id) = tid;
	return ret;
}

static void init_timer()
{
	rte_timer_subsystem_init();
}

/* Call this from the main thread on ZCSI to initialize things. This initializes 
 * the master thread. */
int init_system(int tid, int core)
{
	init_timer();
	if (init_eal(tid, core) < 0) {
		return 0;
	}
	return init_mempool();
}

/* Declared within eal_thread.c, but not exposed */
void eal_thread_init_master(int);

/* Called by each secondary threads on ZCSI, responsible for affinitization,
 * etc.*/
void init_thread(int tid, int core)
{
	/* Among other things this affinitizes the thread */
	eal_thread_init_master(core);
	/* Set thread ID correct */
	RTE_PER_LCORE(_lcore_id) = tid;
	/* TODO: Set NUMA domain for lcore */
}
