#!/bin/bash
TEST_NAME=srv6-inject
PORT_OPTIONS1="dpdk:eth_pcap0,rx_pcap=data/srv6_tcp.pcap,tx_pcap=/tmp/out.pcap"
PORT_OPTIONS2="dpdk:eth_pcap0,rx_pcap=data/ipv6_tcp.pcap,tx_pcap=/tmp/out.pcap"

C='\033[1;34m'
NC='\033[0m'

echo -e "${C}RUNNING: $TEST_NAME${NC}"

../../build.sh run $TEST_NAME -p $PORT_OPTIONS1 -c 1 --dur 1
tcpdump -ter /tmp/out.pcap | tee /dev/tty | diff - data/expect_srv6.out
TEST_SRv6=0

../../build.sh run $TEST_NAME -p $PORT_OPTIONS2 -c 1 --dur 1
tcpdump -ter /tmp/out.pcap | tee /dev/tty | diff - data/expect_ipv6.out
TEST_IPv6toSRv6=$?

echo ----
if [[ $TEST_IPv6toSRv6 != 0 ]] || [[ $TEST_SRv6 != 0 ]]; then
    echo "FAIL: SRv6 Test - $TEST_SRv6 | IPv6toSRv6 Test - $TEST_IPv6toSRv6"
    exit 1
else
    echo "PASS"
fi
