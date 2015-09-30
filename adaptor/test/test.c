#include <stdio.h>
#include <assert.h>
#include <stdint.h>
#include <dpdk.h>
#include <rte_config.h>
#include <rte_lcore.h>
#include <pthread.h>

struct node {
	int tid;
	int core;
};

uint64_t fib (uint64_t l)
{
	uint64_t a = 0, b = 1;
	while (b < l) {
		int temp = a;
		a = b;
		b = a + temp;
	}
	return b;
}

void *thr(void* arg)
{
	struct node* n = arg;
	printf("Started new thread %d %d\n", n->tid, n->core);
	init_thread(n->tid, n->core);
	printf("Socket ID is %d\n", rte_socket_id());
	return NULL;
}

int main (int argc, char* argv[]) {

	/*uint64_t f = fib(1ul << 48);*/
	/*pthread_t thread[20];*/
	/*struct node n[20];*/
	int ret = init_system(32, 5);

	assert(ret >= 0);
	/*printf("%lu\n", f);*/
	printf("%u\n", rte_socket_id());
	init_thread(64, 8);
	printf("Socket ID is %u\n", rte_socket_id());
	/*n[0].tid = 1;*/
	/*n[0].core = 1;*/
	/*pthread_create(&thread[0],*/
			/*NULL,*/
			/*&thr,*/
			/*&n[0]);*/
	/*for (int i = 0; i < 20; i++) {*/
		/*printf("For lcore %d cpuset %d\n",*/
				/*i,*/
				/*lcore_config[i].core_id);*/
		/*for (int j = 1; j < 10; j++) {*/
			/*printf("\tis_set (%d): %d\n", j,*/
					/*CPU_ISSET(j, &lcore_config[i].cpuset));*/
		/*}*/
		/*n[i].tid = i;*/
		/*n[i].core = i;*/
		/*pthread_create(&thread[i],*/
				/*NULL,*/
				/*&thr,*/
				/*&n[i]);*/
	/*}*/

	/*for (int i = 0; i < 20; i++) {*/
		/*pthread_join(thread[0], NULL);*/
	/*}*/
	return 1;
}
