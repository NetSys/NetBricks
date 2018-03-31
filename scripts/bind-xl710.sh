#!/bin/bash
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
DPDK_HOME=$BASE_DIR/../3rdparty/dpdk
modprobe uio
insmod $DPDK_HOME/build/kmod/igb_uio.ko
$DPDK_HOME/usertools/dpdk-devbind.py --status \
			| grep XL710 \
			| awk '{print $1}' \
			| xargs \
			$DPDK_HOME/usertools/dpdk-devbind.py -b igb_uio
