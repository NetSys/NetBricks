#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <rte_config.h>
#include <rte_eth_ring.h>
#include <rte_ethdev.h>
#include <rte_log.h>
#include <rte_ring.h>

#include "mempool.h"

/**
 * This file provides utility methods to connect to various vSwitches (OVS,
 * Bess) using RTE_RING, this is mostly to allow emulating fast container based
 * connections.
 **/

#define __cacheline_aligned __attribute__((aligned(64)))
#define PORT_NAME_LEN 128
#define MAX_QUEUES_PER_DIR 32
/* This is equivalent to the old bar */
struct rte_ring_bar {
    char name[PORT_NAME_LEN];

    /* The term RX/TX could be very confusing for a virtual switch.
     * Instead, we use the "incoming/outgoing" convention:
     * - incoming: outside -> SoftNIC
     * - outgoing: SoftNIC -> outside */
    int num_inc_q;
    int num_out_q;

    struct rte_ring *inc_qs[MAX_QUEUES_PER_DIR];

    struct rte_ring *out_qs[MAX_QUEUES_PER_DIR];
};

#define PORT_DIR_PREFIX "sn_vports"
#define PORT_FNAME_LEN 128 + 256

static int init_bess_ring(const char *ifname, struct rte_mempool *mempool) {
    struct rte_ring_bar *bar;
    int i;

    FILE *fd;
    char port_file[PORT_FNAME_LEN];
    int port;
    struct rte_eth_conf null_conf;

    memset(&null_conf, 0, sizeof(struct rte_eth_conf));

    snprintf(port_file, PORT_FNAME_LEN, "%s/%s/%s", P_tmpdir, PORT_DIR_PREFIX, ifname);
    fd = fopen(port_file, "r");
    if (!fd) {
        RTE_LOG(WARNING, PMD, "Could not open %s\n", port_file);
        return -EINVAL;
    }
    /* Assuming we need to read one pointer */
    i = fread(&bar, 8, 1, fd);
    fclose(fd);
    if (i != 1) {
        RTE_LOG(WARNING, PMD, "Read %d bytes\n", i);
        return -EINVAL;
    }
    if (bar == NULL) {
        RTE_LOG(WARNING, PMD, "Could not find bar\n");
        return -EINVAL;
    }
    RTE_LOG(INFO, PMD, "Initing vport %p found name %s\n", bar, bar->name);
    RTE_LOG(INFO, PMD, "String length %d\n", PORT_NAME_LEN);
    RTE_LOG(INFO, PMD, "num_inc_q offset %d\n", (int)offsetof(struct rte_ring_bar, num_inc_q));
    RTE_LOG(INFO, PMD, "num_out_q offset %d\n", (int)offsetof(struct rte_ring_bar, num_out_q));
    RTE_LOG(INFO, PMD, "Going to init ring port %s\n", ifname);
    RTE_LOG(INFO, PMD, "Going to create port %s %d %d\n", bar->name, bar->num_out_q, bar->num_inc_q);
    port = rte_eth_from_rings(bar->name, bar->out_qs, bar->num_out_q, bar->inc_qs, bar->num_inc_q, 0);
    for (i = 0; i < bar->num_out_q; i++) {
        rte_eth_rx_queue_setup(port, i, 32, 0, NULL, mempool);
    }
    for (i = 0; i < bar->num_inc_q; i++) {
        rte_eth_tx_queue_setup(port, i, 32, 0, NULL);
    }
    RTE_LOG(INFO, PMD, "Port %d\n", port);
    return port;
    // Do not call rte_eth_dev_configure, else everything breaks
}

/* FIXME: Take name length */
int init_bess_eth_ring(const char *ifname, int core) {
    return init_bess_ring(ifname, get_mempool_for_core(core));
}

static int init_ovs_ring(int iface, struct rte_mempool *mempool) {
    const int Q_NAME               = 64;
    const char *RING_NAME          = "dpdkr%u";
    const char *MP_CLIENT_RXQ_NAME = "dpdkr%u_tx";
    const char *MP_CLIENT_TXQ_NAME = "dpdkr%u_rx";
    char rxq_name[Q_NAME];
    char txq_name[Q_NAME];
    char port_name[Q_NAME];
    struct rte_ring *rxq = NULL;
    struct rte_ring *txq = NULL;
    int port             = 0;
    /* Get queue names */
    snprintf(rxq_name, Q_NAME, MP_CLIENT_RXQ_NAME, iface);
    snprintf(txq_name, Q_NAME, MP_CLIENT_TXQ_NAME, iface);
    snprintf(port_name, Q_NAME, RING_NAME, iface);
    rxq = rte_ring_lookup(rxq_name);
    txq = rte_ring_lookup(txq_name);

    port = rte_eth_from_rings(port_name, &rxq, 1, &txq, 1, 0);
    rte_eth_rx_queue_setup(port, 0, 32, 0, NULL, mempool);
    rte_eth_tx_queue_setup(port, 0, 32, 0, NULL);
    return port;
}

int init_ovs_eth_ring(int iface, int core) {
    return init_ovs_ring(iface, get_mempool_for_core(core));
}
