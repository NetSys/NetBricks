#!/bin/bash
TEST_NAME=ipv4or6
PORT_OPTIONS1="dpdk:eth_pcap0,rx_pcap=data/ipv4_tcp.pcap,tx_pcap=/tmp/out.pcap"

PORT_OPTIONS2="dpdk:eth_pcap0,rx_pcap=data/ipv6_tcp.pcap,tx_pcap=/tmp/out.pcap"

../../build.sh run $TEST_NAME -p $PORT_OPTIONS1 -c 1 --dur 1
tcpdump -ter /tmp/out.pcap | tee /dev/tty | diff - data/expect_ipv4.out
TEST_IPv4=$?

../../build.sh run $TEST_NAME -p $PORT_OPTIONS2 -c 1 --dur 1
tcpdump -ter /tmp/out.pcap | tee /dev/tty | diff - data/expect_ipv6.out
TEST_IPv6=$?

echo ----
if [[ $TEST_IPv4 != 0 ]] | [[ $TEST_IPv6 != 0 ]]; then
    echo FAIL: IPv4 Test - $TEST_IPv4 | IPv6 Test - $TEST_IPv6
else
    echo PASS
fi
