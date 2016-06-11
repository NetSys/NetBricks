#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>

#include <rte_config.h>
#include <rte_cycles.h>
#include <rte_timer.h>
#include <rte_ethdev.h>
#include <rte_eal.h>

#include "mempool.h"
#define NUM_PFRAMES (2048 - 1)  // Number of pframes in the mempool
#define CACHE_SIZE 32           // Size of per-core mempool cache

/* Taken from SoftNIC (dpdk.c) */
/* Get NUMA count */
static int get_numa_count() {
    FILE* fp;

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

    fprintf(stderr,
            "Failed to detect # of NUMA nodes from: "
            "/sys/devices/system/node/possible. "
            "Assuming a single-node system...\n");
    return 1;
}

static int init_eal(char* name, int secondary, int core, char* whitelist[], int wl_count, char* vdevs[],
                    int vdev_count) {
    /* As opposed to SoftNIC, this call only initializes the master thread.
     * We cannot rely on threads launched by DPDK within ZCSI, the threads
     * must be launched by the runtime */
    int rte_argc = 0;

    /* FIXME: Make sure number of arguments is not exceeded */
    char* rte_argv[128];

    char opt_master_lcore[1024];
    char opt_lcore_bitmap[1024];
    char opt_socket_mem[1024];

    const char* socket_mem = "1024";

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

    sprintf(opt_socket_mem, "%s", socket_mem);
    for (i = 1; i < numa_count; i++) sprintf(opt_socket_mem + strlen(opt_socket_mem), ",%s", socket_mem);

    rte_argv[rte_argc++] = "lzcsi";
    if (secondary) {
        rte_argv[rte_argc++] = "--proc-type";
        rte_argv[rte_argc++] = "secondary";
    }
    rte_argv[rte_argc++] = "--file-prefix";
    rte_argv[rte_argc++] = name;
    rte_argv[rte_argc++] = "-c";
    rte_argv[rte_argc++] = opt_lcore_bitmap;
    /* Otherwise assume everything is white listed */
    for (int i = 0; i < wl_count; i++) {
        rte_argv[rte_argc++] = "-w";
        rte_argv[rte_argc++] = whitelist[i];
    }
    for (int i = 0; i < vdev_count; i++) {
        rte_argv[rte_argc++] = "--vdev";
        rte_argv[rte_argc++] = vdevs[i];
    }
    rte_argv[rte_argc++] = "-w";
    rte_argv[rte_argc++] = "99:99.0";
    rte_argv[rte_argc++] = "--master-lcore";
    rte_argv[rte_argc++] = opt_master_lcore;
    rte_argv[rte_argc++] = "-n";
    /* number of memory channels (Sandy Bridge) */
    rte_argv[rte_argc++] = "4";  // Number of memory channels on
    // Sandy Bridge.
    rte_argv[rte_argc++] = "--socket-mem";
    rte_argv[rte_argc++] = opt_socket_mem;
    rte_argv[rte_argc] = NULL;

    /* reset getopt() */
    optind = 0;

    /* rte_eal_init: Initializes EAL */
    ret = rte_eal_init(rte_argc, rte_argv);
    if (secondary && rte_eal_process_type() != RTE_PROC_SECONDARY) {
        rte_panic("Not a secondary process");
    }

    /* Change lcore ID */
    RTE_PER_LCORE(_lcore_id) = tid;
    RTE_PER_LCORE(_mempool_core) = core;
    return ret;
}

static void init_timer() { rte_timer_subsystem_init(); }

#define MAX_NAME_LEN 256
int init_secondary(const char* name, int nlen, int core, char* vdevs[], int vdev_count) {
    int ret = 0;
    char clean_name[MAX_NAME_LEN];
    if (name == NULL || nlen >= MAX_NAME_LEN) {
        return -EINVAL;
    }
    strncpy(clean_name, name, nlen);
    clean_name[nlen] = '\0';

    init_timer();
    if ((ret = init_eal(clean_name, 1, core, NULL, 0, vdevs, vdev_count)) < 0) {
        return ret;
    }
    return find_secondary_mempool();
}

int init_system_whitelisted(const char* name, int nlen, int core, char* whitelist[], int wlcount,
                            unsigned int mempool_size, unsigned int mcache_size) {
    int ret = 0;
    if (name == NULL || nlen >= MAX_NAME_LEN) {
        return -EINVAL;
    }
    char clean_name[MAX_NAME_LEN];
    strncpy(clean_name, name, nlen);
    clean_name[nlen] = '\0';

    init_timer();
    if ((ret = init_eal(clean_name, 0, core, whitelist, wlcount, NULL, 0)) < 0) {
        return ret;
    }
    return init_mempool(core, mempool_size, mcache_size);
}

/* Call this from the main thread on ZCSI to initialize things. This initializes
 * the master thread. */
int init_system(char* name, int nlen, int core) {
    return init_system_whitelisted(name, nlen, core, NULL, 0, NUM_PFRAMES, CACHE_SIZE);
}

/* Declared within eal_thread.c, but not exposed */
RTE_DECLARE_PER_LCORE(unsigned, _socket_id);

/* Called by each secondary threads on ZCSI, responsible for affinitization,
 * etc.*/
void init_thread(int tid, int core) {
    /* Among other things this affinitizes the thread */
    rte_cpuset_t cpuset;
    CPU_ZERO(&cpuset);
    CPU_SET(core, &cpuset);
    rte_thread_set_affinity(&cpuset);
    init_mempool_core(core);
    /* Set thread ID correctly */
    RTE_PER_LCORE(_lcore_id) = tid;
    RTE_PER_LCORE(_mempool_core) = core;
}
