#!/bin/bash
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
DPDK_HOME=$BASE_DIR/../3rdparty/dpdk
export LD_LIBRARY_PATH=$DPDK_HOME/build/lib
modprobe uio
insmod $DPDK_HOME/build/kmod/igb_uio.ko
$DPDK_HOME/tools/dpdk_nic_bind.py --status \
			| grep XL710 \
			| awk '{print $1}' \
			| xargs \
			$DPDK_HOME/tools/dpdk_nic_bind.py -b igb_uio
