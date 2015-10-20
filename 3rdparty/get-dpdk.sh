#!/bin/bash
DPDK_VER=2.1.0
DOWNLOAD_PATH="dpdk.tar.gz"
DPDK_RESULT=dpdk
if [ ! -e "$DOWNLOAD_PATH" ]; then
	curl http://dpdk.org/browse/dpdk/snapshot/dpdk-${DPDK_VER}.tar.gz -o dpdk.tar.gz
fi
if [ ! -d "$DPDK_RESULT" ]; then
	mkdir $DPDK_RESULT
fi

tar zxvf dpdk.tar.gz -C dpdk --strip-components=1
cp common_linuxapp-${DPDK_VER} dpdk/config/common_linuxapp
