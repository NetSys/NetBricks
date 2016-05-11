#!/bin/bash
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
DPDK_VER=16.04
MODE=download # or git
DOWNLOAD_PATH="${BASE_DIR}/dpdk.tar.gz"
DPDK_RESULT="${BASE_DIR}/dpdk"

if [ "$MODE" = "download" ]; then
	if [ ! -e "$DOWNLOAD_PATH" ]; then
		echo Fetching "http://dpdk.org/browse/dpdk/snapshot/dpdk-${DPDK_VER}.tar.gz"
		curl http://dpdk.org/browse/dpdk/snapshot/dpdk-${DPDK_VER}.tar.gz -o "${DOWNLOAD_PATH}"
	fi
	if [ ! -d "${DPDK_RESULT}" ]; then
		mkdir -p ${DPDK_RESULT}
	fi
	tar zxvf "${DOWNLOAD_PATH}" -C "${DPDK_RESULT}" --strip-components=1
else
	DPDK_REV="2e14846d15addd349a909176473e936f0cf79075"
	if [ ! -d "${DPDK_RESULT}" ]; then
		git clone git://dpdk.org/dpdk ${DPDK_RESULT}
		pushd ${DPDK_RESULT}
		git checkout $DPDK_REV
		popd
	fi
fi

cp "${BASE_DIR}/common_linuxapp-${DPDK_VER}" "${DPDK_RESULT}/config/common_linuxapp"
export RTE_TARGET=x86_64-native-linuxapp-gcc
make config -C "${DPDK_RESULT}" T=x86_64-native-linuxapp-gcc EXTRA_CFLAGS="-g3 -Wno-error=maybe-uninitialized"
PROCS="$(nproc)"
make -j $PROCS -C "${DPDK_RESULT}"
