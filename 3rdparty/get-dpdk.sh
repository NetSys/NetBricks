#!/bin/bash
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
DPDK_VER=2.2.0
DOWNLOAD_PATH="${BASE_DIR}/dpdk.tar.gz"
DPDK_RESULT="${BASE_DIR}/dpdk"
if [ ! -e "$DOWNLOAD_PATH" ]; then
	echo Fetching "http://dpdk.org/browse/dpdk/snapshot/dpdk-${DPDK_VER}.tar.gz"
	curl http://dpdk.org/browse/dpdk/snapshot/dpdk-${DPDK_VER}.tar.gz -o "${DOWNLOAD_PATH}"
fi
if [ ! -d "${DPDK_RESULT}" ]; then
	mkdir "${DPDK_RESULT}"
fi

tar zxvf "${DOWNLOAD_PATH}" -C "${DPDK_RESULT}" --strip-components=1
cp "${BASE_DIR}/common_linuxapp-${DPDK_VER}" "${DPDK_RESULT}/config/common_linuxapp"
export RTE_TARGET=x86_64-native-linuxapp-gcc
make config -C "${DPDK_RESULT}" T=x86_64-native-linuxapp-gcc EXTRA_CFLAGS="-g3 -Wno-error=maybe-uninitialized"
PROCS="$(nproc)"
make -j $PROCS -C "${DPDK_RESULT}"
