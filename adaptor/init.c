#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <rte_config.h>
#include <rte_cycles.h>
#include <rte_timer.h>
#include <rte_ethdev.h>
#include <rte_eal.h>

/* Need to set this to be able to read time from DPDK */
extern uint64_t tsc_hz;

/* Taken from SoftNIC (dpdk.c) */
/* Generate an lcore bitmap. For now only launch one worker */
static int set_lcore_bitmap(char *buf, int tid, int core)
{
	int off = 0;
	sprintf(buf, "%d@%d,", tid, core);
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

static void init_eal(int tid, int core)
{
	int rte_argc = 0;
	char *rte_argv[16];

	char opt_master_lcore[1024];
	char opt_lcore_bitmap[1024];
	char opt_socket_mem[1024];

	const char *socket_mem = "2048";

	int numa_count = get_numa_count();

	int ret;
	int i;

	sprintf(opt_master_lcore, "%d", RTE_MAX_LCORE - 1);

	/* The actual lcore */
	i = set_lcore_bitmap(opt_lcore_bitmap, tid, core);
	/* The master lcore */
	i = set_lcore_bitmap(opt_lcore_bitmap + i,
			RTE_MAX_LCORE - 1,
			core);

	sprintf(opt_socket_mem, "%s", socket_mem);
	for(i = 1; i < numa_count; i++)
		sprintf(opt_socket_mem + strlen(opt_socket_mem), ",%s", socket_mem);

	rte_argv[rte_argc++] = "lzcsi";
	rte_argv[rte_argc++] = "--master-lcore";
	rte_argv[rte_argc++] = opt_master_lcore;
	rte_argv[rte_argc++] = "--lcore";
	rte_argv[rte_argc++] = opt_lcore_bitmap;
	rte_argv[rte_argc++] = "-n";
	rte_argv[rte_argc++] = "4";	/* number of memory channels (Sandy Bridge) */
#if 1
	rte_argv[rte_argc++] = "--socket-mem";
	rte_argv[rte_argc++] = opt_socket_mem;
#else
	rte_argv[rte_argc++] = "--no-huge";
#endif
	rte_argv[rte_argc] = NULL;

	/* reset getopt() */
	optind = 0;

	ret = rte_eal_init(rte_argc, rte_argv);
	assert(ret >= 0);
}

static void init_timer()
{
	rte_timer_subsystem_init();
}

/* Reenable this after copying stuff from SoftNIC */
/*#if DPDK < DPDK_VER(2, 0, 0)*/
  /*#error DPDK 2.0.0 or higher is required*/
/*#endif*/

/* Call this from the main thread on ZCSI to initialize things */
void init_system(int tid, int core)
{
	init_eal(tid, core);

	tsc_hz = rte_get_tsc_hz();
}

/* Declared within eal_thread.c, but not exposed */
void eal_thread_init_master(int);

/* Called from all secondary threads on ZCSI */
void init_thread(int tid, int core)
{
	/* Among other things this affinitizes the thread */
	eal_thread_init_master(core);
	/* Set thread ID correct */
	RTE_PER_LCORE(_lcore_id) = tid;
	/* TODO: Set NUMA domain for lcore */
}
