#!/bin/bash
TEST_NAME=signals

C='\033[1;34m'
NC='\033[0m'

echo -e "${C}RUNNING: $TEST_NAME${NC}"

PORT_OPTIONS="dpdk:eth_pcap0,rx_pcap=data/srv6_tcp.pcap,tx_pcap=/tmp/out.pcap"

nohup ../../build.sh run $TEST_NAME -p $PORT_OPTIONS -c 1 &
# Extra time to load the signaler
sleep 3
PID=`pidof signals`
kill -HUP "$PID"
sleep 1
kill -TERM "$PID"
sleep 1

echo ----

pidof signals
if [[ $? == 0 ]]; then
    kill -9 "$PID"
    echo "FAIL: process still running"
    exit 1
fi

cat test.log | tee /dev/tty | diff - data/expect.out
if [[ $? != 0 ]]; then
    echo "FAIL: wrong output"
    exit 1
fi

echo "PASS"
