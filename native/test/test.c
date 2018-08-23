// TODO: Fix - segfaults due to include version expectation.

#include <stdio.h>
#include <assert.h>
#include <stdint.h>
#include <dpdk.h>
#include <pmd.h>
#include <rte_config.h>
#include <rte_lcore.h>
#include <pthread.h>
#include <sys/time.h>

struct node {
	int tid;
	int core;
	int queue;
};

int cursec()
{
	struct timeval t;
	gettimeofday(&t, NULL);
	return t.tv_sec;
}

#define PORT_OUT 1
#define PORT_IN 0
void *thr(void* arg)
{
	struct node* n = arg;
	struct rte_mbuf* restrict pkts[32];
	int i;
	int q = n->queue;
	int start_sec = cursec();
	int rcvd = 0;
	int sent = 0;
	init_thread(n->tid, n->core);
	if (q >= 20) {
		printf("Somehow, queue beyond 20\n");
	}
	while(1) {
		/*int recv;*/
		i = mbuf_alloc_bulk(pkts, 60, 32);
		if (i != 0) {
			printf("Error allocating packets %d\n", i);
			break;
		} else {
			int send, recv;

			/* Start setting MAC address */
			for (i = 0; i < 32; i++) {
				struct ether_hdr* hdr =
					rte_pktmbuf_mtod(pkts[i],
						struct ether_hdr*);
				hdr->d_addr.addr_bytes[5] = (10 * q) + 1;
				hdr->s_addr.addr_bytes[5] = (10 * q) + 2;
				hdr->ether_type = rte_cpu_to_be_16(0x0800);
				/*rte_mbuf_sanity_check(pkts[i], 1);*/
			}
			send = send_pkts(PORT_OUT, q, pkts, 32);
			for (i = send; i < 32; i++) {
				mbuf_free(pkts[i]);
			}
			recv = recv_pkts(PORT_IN, q, pkts, 32);
			rcvd += recv;
			sent += send;
			if (cursec() != start_sec) {
				printf("%d %d rx=%d tx=%d\n", n->core,
						(cursec() - start_sec),
						rcvd,
						sent);
				/*printf("recv_pkt\n");*/
				/*rte_pktmbuf_dump(stdout, pkts[0], 16384);*/
				start_sec = cursec();
				rcvd = 0;
				sent = 0;
			}
			for (int i = 0; i < recv; i++) {
				mbuf_free(pkts[i]);
			}
		}
	}
	printf("Socket ID (%d) is %d. DONE\n", n->core, rte_socket_id());
	return NULL;
}

void dump() {
	printf("pkt_len %lu\n", offsetof(struct rte_mbuf, pkt_len));
	printf("sizeof(rte_eth_dev_info) %lu\n", sizeof(struct rte_eth_dev_info));
}

#define THREADS 2
int main (int argc, char* argv[]) {

	/*dump();*/
	pthread_t thread[20];
	struct node n[20];
	int rxq_cores[20];
	int txq_cores[20];
	int ret = init_system(1);

	assert(ret == 0);
	rxq_cores[0] = 10;
	rxq_cores[1] = 11;
	txq_cores[0] = 10;
	txq_cores[1] = 11;

	/*for (int i = 0; i < 20; i++) {*/
		/*rxq_cores[i] = i;*/
		/*txq_cores[i] = i;*/
	/*}*/
	enumerate_pmd_ports();
	ret = init_pmd_port(PORT_OUT, THREADS, THREADS,
			rxq_cores, txq_cores, 256, 256,
			PORT_OUT == PORT_IN, 0, 0);
	assert(ret == 0);
	if (PORT_IN != PORT_OUT) {
		ret = init_pmd_port(PORT_IN, THREADS, THREADS, rxq_cores, txq_cores, 128, 512, 0, 0, 0);
		assert(ret == 0);
	}
	n[0].tid = 10;
	n[0].core = 10;
	n[0].queue = 0;
	n[1].tid = 11;
	n[1].core = 11;
	n[1].queue = 1;
	/*thr(&n[0]);*/
	pthread_create(&thread[0], NULL, &thr, &n[0]);
	pthread_create(&thread[1], NULL, &thr, &n[1]);
	/*for (int i = 0; i < THREADS; i++) {*/
		/*n[i].tid = 64 - i;*/
		/*n[i].core = i;*/
		/*pthread_create(&thread[i],*/
				/*NULL,*/
				/*&thr,*/
				/*&n[i]);*/
	/*}*/

	for (int i = 0; i < THREADS; i++) {
		pthread_join(thread[i], NULL);
	}
	free_pmd_port(PORT_OUT);
	return 0;
}
