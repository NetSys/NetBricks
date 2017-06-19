#include <assert.h>
#include <numa.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <rte_config.h>
#include <rte_cycles.h>
#include <rte_eal.h>
#include <rte_ethdev.h>
#include <rte_timer.h>

#include "mempool.h"
#define NUM_PFRAMES (2048 - 1)  // Number of pframes in the mempool
#define MEMPOOL_SIZE 1024       // Default mempool size
#define CACHE_SIZE 32           // Size of per-core mempool cache

#define MAX_ARGS 128

static inline void bind_to_domain(int socket_id) {
    struct bitmask* numa_bitmask = numa_bitmask_setbit(
        numa_bitmask_clearall(numa_bitmask_alloc(numa_num_possible_nodes())), socket_id);
    numa_bind(numa_bitmask);
}

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

static void add_arg(int* rte_argc, char** rte_argv, char* s) {
    if (*rte_argc >= MAX_ARGS) {
        fprintf(stderr, "init_eal exceeded max number of args!");
        return;
    }
    rte_argv[(*rte_argc)++] = s;
}

static int init_eal(char* name, int secondary, int core, int mempool_size, char* whitelist[],
                    int wl_count, char* vdevs[], int vdev_count) {
    /* As opposed to SoftNIC, this call only initializes the master thread.
     * We cannot rely on threads launched by DPDK within ZCSI, the threads
     * must be launched by the runtime */
    int rte_argc = 0;

    char* rte_argv[MAX_ARGS];

    char opt_master_lcore[1024];
    char opt_lcore_bitmap[1024];
    char opt_socket_mem[1024];

    int numa_count = get_numa_count();
    int socket_id  = 0;

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

    sprintf(opt_socket_mem, "%d", mempool_size);
    for (i = 1; i < numa_count; i++)
        sprintf(opt_socket_mem + strlen(opt_socket_mem), ",%d", mempool_size);

    add_arg(&rte_argc, rte_argv, "lzcsi");
    if (secondary) {
        add_arg(&rte_argc, rte_argv, "--proc-type");
        add_arg(&rte_argc, rte_argv, "secondary");
    }
    add_arg(&rte_argc, rte_argv, "--file-prefix");
    add_arg(&rte_argc, rte_argv, name);
    add_arg(&rte_argc, rte_argv, "-c");
    add_arg(&rte_argc, rte_argv, opt_lcore_bitmap);

    for (int i = 0; i < wl_count; i++) {
        add_arg(&rte_argc, rte_argv, "-w");
        add_arg(&rte_argc, rte_argv, whitelist[i]);
    }
    for (int i = 0; i < vdev_count; i++) {
        add_arg(&rte_argc, rte_argv, "--vdev");
        add_arg(&rte_argc, rte_argv, vdevs[i]);
    }

    /* This just makes sure that by default everything is blacklisted */
    add_arg(&rte_argc, rte_argv, "-w");
    add_arg(&rte_argc, rte_argv, "99:99.0");

    add_arg(&rte_argc, rte_argv, "--master-lcore");
    add_arg(&rte_argc, rte_argv, opt_master_lcore);

    add_arg(&rte_argc, rte_argv, "-n");
    /* number of memory channels (Sandy Bridge) */
    add_arg(&rte_argc, rte_argv, "4");  // Number of memory channels on
    // Sandy Bridge.
    add_arg(&rte_argc, rte_argv, "--socket-mem");
    add_arg(&rte_argc, rte_argv, opt_socket_mem);
    add_arg(&rte_argc, rte_argv, NULL);

    /* reset getopt() */
    optind = 0;

    /* rte_eal_init: Initializes EAL */
    ret = rte_eal_init(rte_argc, rte_argv);
    if (secondary && rte_eal_process_type() != RTE_PROC_SECONDARY) {
        rte_panic("Not a secondary process");
    }

    /* Change lcore ID */
    RTE_PER_LCORE(_lcore_id)     = tid;
    RTE_PER_LCORE(_mempool_core) = core;
    socket_id                    = rte_lcore_to_socket_id(core);
    if (numa_available() != -1) {
        bind_to_domain(socket_id);
    }

    return ret;
}

static void init_timer() {
    rte_timer_subsystem_init();
}

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
    if ((ret = init_eal(clean_name, 1, core, MEMPOOL_SIZE, NULL, 0, vdevs, vdev_count)) < 0) {
        return ret;
    }
    return find_secondary_mempool();
}

int init_system_whitelisted(const char* name, int nlen, int core, char* whitelist[], int wlcount,
                            unsigned int mempool_size, unsigned int mcache_size, int slots) {
    int ret = 0;
    if (name == NULL || nlen >= MAX_NAME_LEN) {
        return -EINVAL;
    }
    char clean_name[MAX_NAME_LEN];
    strncpy(clean_name, name, nlen);
    clean_name[nlen] = '\0';

    init_timer();
    if ((ret = init_eal(clean_name, 0, core, mempool_size, whitelist, wlcount, NULL, 0)) < 0) {
        return ret;
    }
    return init_mempool(core, mempool_size, mcache_size, slots);
}

/* Call this from the main thread on ZCSI to initialize things. This initializes
 * the master thread. */
int init_system(char* name, int nlen, int core, int slots) {
    return init_system_whitelisted(name, nlen, core, NULL, 0, NUM_PFRAMES, CACHE_SIZE, slots);
}

/* Declared within eal_thread.c, but not exposed */
RTE_DECLARE_PER_LCORE(unsigned, _socket_id);

/* Called by each secondary threads on ZCSI, responsible for affinitization,
 * etc.*/
int init_thread(int tid, int core) {
    /* Among other things this affinitizes the thread */
    rte_cpuset_t cpuset;
    int socket_id   = rte_lcore_to_socket_id(core);
    int numa_active = numa_available();
    CPU_ZERO(&cpuset);
    CPU_SET(core, &cpuset);
    rte_thread_set_affinity(&cpuset);
    if (numa_active != -1) {
        bind_to_domain(socket_id);
    }
    init_mempool_core(core);

    /* Set thread ID correctly */
    RTE_PER_LCORE(_lcore_id)     = tid;
    RTE_PER_LCORE(_mempool_core) = core;
    return numa_active == -1 ? numa_active : socket_id;
}
