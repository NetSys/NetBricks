#!/bin/bash
TEST_NAME=ndp-neighbor-solicitation

C='\033[1;34m'
NC='\033[0m'

echo -e "${C}RUNNING: $TEST_NAME${NC}"

PORT_OPTIONS="dpdk:eth_pcap0,rx_pcap=data/ndp_neighbor_solicitation.pcap,tx_pcap=/tmp/out.pcap"

../../build.sh run $TEST_NAME -p $PORT_OPTIONS -c 1 --dur 1
tcpdump -tner /tmp/out.pcap | tee /dev/tty | diff - data/expect_ndp_neighbor_solicitation.out

TEST_NEIGHBOR_SOLICITATION=$?

echo ----
if [[ $TEST_NEIGHBOR_SOLICITATION != 0 ]]; then
    echo "FAIL: Neighbor Solicitation Test - $TEST_NEIGHBOR_SOLICITATION"
    exit 1
else
    echo "PASS"
fi