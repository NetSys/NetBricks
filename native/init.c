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

static int init_eal(int core, char* whitelist[], int wl_count)
{
	/* As opposed to SoftNIC, this call only initializes the master thread.
	 * We cannot rely on threads launched by DPDK within ZCSI, the threads
	 * must be launched by the runtime */
	int rte_argc = 0;
	char *rte_argv[64];

	char opt_master_lcore[1024];
	char opt_lcore_bitmap[1024];
	char opt_socket_mem[1024];

	const char *socket_mem = "1024";
	char prefix[50];

	int numa_count = get_numa_count();

	int ret;
	int i;
	int tid = core;

	if (core > RTE_MAX_LCORE || tid > RTE_MAX_LCORE) {
		return -1;
	}

	sprintf(opt_master_lcore, "%d", tid);

	/* We need to tell rte_eal_init that it should use all possible lcores.
	 * If not, it does an insane thing and 0s out the cpusets for any unused
	 * physical cores and will not work when new threads are allocated. We
	 * could hack around this another way, but this seems more reasonable.*/
	sprintf(opt_lcore_bitmap, "0x%x", (1u << core));
	sprintf(prefix, "hp%d", core);
	printf("Core mask : %s\n", opt_lcore_bitmap);

	sprintf(opt_socket_mem, "%s", socket_mem);
	for(i = 1; i < numa_count; i++)
		sprintf(opt_socket_mem + strlen(opt_socket_mem),
				",%s", socket_mem);

	rte_argv[rte_argc++] = "lzcsi";
	rte_argv[rte_argc++] = "--file-prefix";
	rte_argv[rte_argc++] = prefix;
	rte_argv[rte_argc++] = "-c";
	rte_argv[rte_argc++] = opt_lcore_bitmap;
	/* Otherwise assume everything is white listed */
	if (wl_count > 0) {
		for (int i = 0; i < wl_count; i++) {
			printf("Whitelisting %s\n", whitelist[i]);
			rte_argv[rte_argc++] = "-w";
			rte_argv[rte_argc++] = whitelist[i];
		}
	}
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

	/* rte_eal_init: Initializes EAL */
	ret = rte_eal_init(rte_argc, rte_argv);

	/* Change lcore ID */
	RTE_PER_LCORE(_lcore_id) = tid;
	RTE_PER_LCORE(_mempool_core) = core;
	return ret;
}

static void init_timer()
{
	rte_timer_subsystem_init();
}

/* Call this from the main thread on ZCSI to initialize things. This initializes
 * the master thread. */
int init_system(int core)
{
	int ret = 0;
	init_timer();
	if ((ret = init_eal(core, NULL, 0)) < 0) {
		return ret;

	}
	return init_mempool();
}

int init_system_whitelisted(int core, char *whitelist[], int wlcount) {
	int ret = 0;
	init_timer();
	if ((ret = init_eal(core, whitelist, wlcount)) < 0) {
		return ret;

	}
	return init_mempool();
}

/* Declared within eal_thread.c, but not exposed */
RTE_DECLARE_PER_LCORE(unsigned , _socket_id);

/* Called by each secondary threads on ZCSI, responsible for affinitization,
 * etc.*/
void init_thread(int tid, int core)
{
	/* Among other things this affinitizes the thread */
	rte_cpuset_t cpuset;
	CPU_ZERO(&cpuset);
	CPU_SET(core, &cpuset);
	rte_thread_set_affinity(&cpuset);
	/* Set thread ID correctly */
	RTE_PER_LCORE(_lcore_id) = tid;
	RTE_PER_LCORE(_mempool_core) = core;
}
