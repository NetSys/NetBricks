#!/bin/bash
TEST_NAME=srv6-sighup-flow
PORT_OPTIONS1="dpdk:eth_pcap0,rx_pcap=data/srv6_tcp.pcap,tx_pcap=/tmp/out.pcap"

C='\033[1;34m'
NC='\033[0m'

echo -e "${C}RUNNING: $TEST_NAME${NC}"

nohup ../../build.sh run $TEST_NAME -p $PORT_OPTIONS1 -c 1 --dur 5 &
sleep 1
PID=`pidof srv6-sighup-flow`
kill -HUP "$PID"
sleep 1
cat test.log | tee /dev/tty | diff - data/expect_srv6.out
TEST_ON_OFF=$?

echo ----
if [[ $TEST_ON_OFF != 0 ]]; then
    echo "FAIL: TEST_SIGHUP - $TEST_ON_OFF"
    exit 1
else
    echo "PASS"
fi
