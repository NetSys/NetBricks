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
	int q = n->core;
	int start_sec = cursec();
	int rcvd = 0;
	int sent = 0;
	int print = 1;
	init_thread(n->tid, n->core);
	if (q >= 20) {
		printf("Somehow, queue beyond 20\n");
	}
	for (int j = 0; j < 1000000;) {
		/*int recv;*/
		i = mbuf_alloc_bulk(pkts, 64, 32);
		if (i != 0) {
			printf("Error allocating packets %d\n", i);
			break;
		} else {
			int send, recv;
			if (print) {
				print = 0;
				/*printf("send_pkt\n");*/
				/*rte_pktmbuf_dump(stdout, pkts[0], 16384);*/
			}
			for (i = 0; i < 32; i++) {
				/* Start setting MAC address */
				struct ether_hdr* hdr =
					rte_pktmbuf_mtod(pkts[i],
						struct ether_hdr*);
				hdr->d_addr.addr_bytes[5] = (10 * q) + 1;
				hdr->s_addr.addr_bytes[5] = (10 * q) + 2;
				hdr->ether_type = rte_cpu_to_be_16(0x1105);
				rte_mbuf_sanity_check(pkts[i], 1);
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
				print = 1;
			}
			for (int i = 0; i < recv; i++) {
				mbuf_free(pkts[i]);
			}
		}
	}
	printf("Socket ID (%d) is %d. DONE\n", n->core, rte_socket_id());
	return NULL;
}
#define THREADS 20 
int main (int argc, char* argv[]) {

	pthread_t thread[20];
	struct node n[20];
	int ret = init_system(0);
	int rxq_cores[20];
	int txq_cores[20];

	assert(ret == 0);

	for (int i = 0; i < 20; i++) {
		rxq_cores[i] = i;
		txq_cores[i] = i;
	}
	enumerate_pmd_ports();
	ret = init_pmd_port(PORT_OUT, THREADS, THREADS, rxq_cores, txq_cores, 128, 512, 0, 0, 0);
	assert(ret == 0);
	ret = init_pmd_port(PORT_IN, THREADS, THREADS, rxq_cores, txq_cores, 128, 512, 0, 0, 0);
	assert(ret == 0);
	for (int i = 0; i < THREADS; i++) {
		n[i].tid = 64 - i;
		n[i].core = i;
		pthread_create(&thread[i],
				NULL,
				&thr,
				&n[i]);
	}

	for (int i = 0; i < THREADS; i++) {
		pthread_join(thread[i], NULL);
	}
	free_pmd_port(PORT_OUT);
	/*free_pmd_port(PORT_IN);*/
	return 0;
}
