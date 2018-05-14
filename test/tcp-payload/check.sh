#!/bin/bash
TEST_NAME=tcp-payload
PORT_OPTIONS="dpdk:eth_pcap0,rx_pcap=data/http_lemmy.pcap,tx_pcap=/tmp/out.pcap"
../../build.sh run $TEST_NAME -p $PORT_OPTIONS -c 1\
    |& tee /dev/tty | sed -n '1,/BEGIN TEST OUTPUT/!p' | diff - data/expect.out

result=$?
echo ----
if [[ $result != 0 ]]; then
  echo FAIL
  exit $result
else
  echo PASS
fi
