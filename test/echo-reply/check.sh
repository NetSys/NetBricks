#!/bin/bash
TEST_NAME=echo-reply

C='\033[1;34m'
NC='\033[0m'

echo -e "${C}RUNNING: $TEST_NAME${NC}"

PORT_OPTIONS1="dpdk:eth_pcap0,rx_pcap=data/echo_request.pcap,tx_pcap=/tmp/out.pcap"

../../build.sh run $TEST_NAME -p $PORT_OPTIONS1 -c 1 --dur 1
tcpdump -tner /tmp/out.pcap | tee /dev/tty | diff - data/echo_reply.out

result=$?
echo ----
if [[ $result != 0 ]]; then
  echo FAIL
  exit $result
else
  echo PASS
fi
