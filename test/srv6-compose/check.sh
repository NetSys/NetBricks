#!/bin/bash
TEST_NAME=srv6-compose
PORT_OPTIONS1="dpdk:eth_pcap0,rx_pcap=data/ipv4_tcp.pcap,tx_pcap=/tmp/out.pcap"
PORT_OPTIONS2="dpdk:eth_pcap0,rx_pcap=data/ipv6_tcp.pcap,tx_pcap=/tmp/out.pcap"
PORT_OPTIONS3="dpdk:eth_pcap0,rx_pcap=data/srv6_tcp.pcap,tx_pcap=/tmp/out.pcap"

C='\033[1;34m'
NC='\033[0m'

echo -e "${C}RUNNING: $TEST_NAME${NC}"

../../build.sh run $TEST_NAME -p $PORT_OPTIONS1 -c 1 --dur 1
tcpdump -ter /tmp/out.pcap | tee /dev/tty | diff - data/expect_ipv4.out
TEST_IPv4=$?

../../build.sh run $TEST_NAME -p $PORT_OPTIONS2 -c 1 --dur 1
tcpdump -ter /tmp/out.pcap | tee /dev/tty | diff - data/expect_ipv6.out
TEST_IPv6=$?

../../build.sh run $TEST_NAME -p $PORT_OPTIONS3 -c 1 --dur 1
tcpdump -ter /tmp/out.pcap | tee /dev/tty | diff - data/expect_srv6.out
TEST_SRv6=$?

echo ----
if [[ $TEST_IPv4 != 0 ]] || [[ $TEST_IPv6 != 0 ]] || [[ $TEST_SRv6 != 0 ]]; then
    echo "FAIL: IPv4 Test - $TEST_IPv4 | IPv6 Test - $TEST_IPv6 | SRv6 Test - $TEST_SRv6"
    exit 1
else
    echo "PASS"
fi
