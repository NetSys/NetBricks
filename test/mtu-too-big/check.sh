#!/bin/bash
TEST_NAME=mtu-too-big

C='\033[1;34m'
NC='\033[0m'

echo -e "${C}RUNNING: $TEST_NAME${NC}"

PORT_OPTIONS1="dpdk:eth_pcap0,rx_pcap=data/in_bounds.pcap,tx_pcap=/tmp/out.pcap"
PORT_OPTIONS2="dpdk:eth_pcap0,rx_pcap=data/over_bounds.pcap,tx_pcap=/tmp/out.pcap"

../../build.sh run $TEST_NAME -p $PORT_OPTIONS1 -c 1 --dur 1
tcpdump -ter /tmp/out.pcap | tee /dev/tty | diff - data/expect_ipv6.out
IN_BOUNDS_IPv6=$?

../../build.sh run $TEST_NAME -p $PORT_OPTIONS2 -c 1 --dur 1
tcpdump -ter /tmp/out.pcap | tee /dev/tty | diff - data/expect_icmpv6_toobig.out
OVER_BOUNDS_IPv6=$?

echo ----
if [[ $IN_BOUNDS_IPv6 != 0 ]] || [[ $OVER_BOUNDS_IPv6 != 0 ]]; then
    echo "FAIL: V6 IN-BOUNDS TEST - $IN_BOUNDS_IPv6 | V6 OVER-BOUNDS TEST - $OVER_BOUNDS_IPv6"
    exit 1
else
    echo "PASS"
fi
