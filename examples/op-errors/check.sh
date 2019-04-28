#!/bin/bash
TEST_NAME=op-errors
PORT_OPTIONS1="dpdk:eth_pcap0,rx_pcap=data/ipv6_tcp.pcap,tx_pcap=/tmp/out.pcap"

C='\033[1;34m'
NC='\033[0m'

echo -e "${C}RUNNING: $TEST_NAME${NC}"

../../build.sh run $TEST_NAME -p $PORT_OPTIONS1 -c 1 -d 1
tcpdump -tner /tmp/out.pcap | tee /dev/tty | diff - data/expect_v6.out
TEST_OUTPUT=$?

cat test.log | tee /dev/tty | diff - data/op_errors.out
TEST_LOG=$?

result=$?
echo ----
if [[ $TEST_OUTPUT != 0 ]] || [[ $TEST_LOG != 0 ]]; then
    echo "FAIL: Test Output - $TEST_OUTPUT | Test Log - $TEST_LOG"
    exit 1
else
    echo "PASS"
fi
