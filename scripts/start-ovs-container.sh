#!/bin/bash
set -o errexit
# args
# 1: Master core (1)
# 2: PMD mask (0x30)
# 3: Number of physical NICs (1)
# 4: Number of rings (1)

OVS_HOME=/opt/e2d2/ovs
VM_HOME=/opt/e2d2/vm
HUGEPAGES_HOME=/opt/e2d2/libhugetlbfs

MASTER_LCORE=${1-"0x1"}
PMD_MASK=${2-"0x30"}
PHYNICS=${3-1}
RINGS=${4-1}

pushd $OVS_HOME
export LD_LIBRARY_PATH=/opt/e2d2/dpdk/build/lib
$( $OVS_HOME/utilities/ovs-dev.py env )
$OVS_HOME/utilities/ovs-dev.py kill
LD_PRELOAD="$HUGEPAGES_HOME/obj64/libhugetlbfs.so"
$OVS_HOME/utilities/ovs-dev.py \
	reset run --dpdk -c $MASTER_LCORE -n 4 -r 1 --socket-mem 1024,0 \
	--file-prefix "rte"
ovs-vsctl set Open . other_config:n-dpdk-rxqs=1
ovs-vsctl add-br b -- set bridge b datapath_type=netdev
ovs-vsctl set Open . other_config:pmd-cpu-mask="$PMD_MASK"
ovs-vsctl set Open . other_config:n-handler-threads=1
ovs-vsctl set Open . other_config:n-revalidator-threads=1
ovs-vsctl set Open . other_config:max-idle=10000

for (( pinterface=0; pinterface<$PHYNICS; pinterface++ )); do
	iface="dpdk${pinterface}"
	echo "Setting up physical interface ${iface}"
	ovs-vsctl add-port b ${iface} -- set Interface ${iface} type=dpdk
done
#ovs-vsctl add-port b dpdk1 -- set Interface dpdk1 type=dpdk
for (( rinterface=0; rinterface<$RINGS; rinterface++ )); do
	iface="dpdkr${rinterface}"
	echo "Setting up physical interface ${iface}"
	ovs-vsctl add-port b ${iface} -- set Interface ${iface} type=dpdkr
done
#ovs-vsctl add-port b v1 -- set Interface v1 type=dpdkvhostuser
ovs-ofctl del-flows b

ovs-ofctl add-flow b in_port=1,actions=output:2
ovs-ofctl add-flow b in_port=2,actions=output:1
#ovs-ofctl add-flow b in_port=3,actions=output:1
#ovs-ofctl add-flow b in_port=4,actions=output:2
popd
