#ifndef __PMD_H__
#define __PMD_H__
void enumerate_pmd_ports();
int init_pmd_port(int port, int rxqs, int txqs, int rxq_core[], int txq_core[],
		int nrxd, int ntxd, int loopback, int tso, int csumoffload);
int free_pmd_port(int port);
int recv_pkts(int port, int qid, mbuf_array_t pkts, int len);
int send_pkts(int port, int qid, mbuf_array_t pkts, int len);
#endif
