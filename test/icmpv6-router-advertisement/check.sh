#!/bin/bash
TEST_NAME=icmpv6-router-advertisement

C='\033[1;34m'
NC='\033[0m'

echo -e "${C}RUNNING: $TEST_NAME${NC}"

PORT_OPTIONS2="dpdk:eth_pcap0,rx_pcap=data/icmpv6-router-advertisement.pcap,tx_pcap=data/out.pcap"

../../build.sh run $TEST_NAME -p $PORT_OPTIONS2 -c 1 --dur 1
