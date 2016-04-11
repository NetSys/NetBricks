#!/bin/bash
set -x
set -o errexit
OVS_HOME=/opt/e2d2/ovs
VM_HOME=/opt/e2d2/vm
HUGEPAGES_HOME=/opt/e2d2/libhugetlbfs
pushd $OVS_HOME
export LD_LIBRARY_PATH=/opt/e2d2/dpdk/build/lib
$( $OVS_HOME/utilities/ovs-dev.py env )
$OVS_HOME/utilities/ovs-dev.py kill
LD_PRELOAD="$HUGEPAGES_HOME/obj64/libhugetlbfs.so" $OVS_HOME/utilities/ovs-dev.py reset run --dpdk \
		-c 0x1 -n 4 -r 1 --socket-mem 1024,0 -w 07:00.0 -w 07:00.1 -w 07:00.2 -w 07:00.3
ovs-vsctl set Open . other_config:n-dpdk-rxqs=1
ovs-vsctl add-br b -- set bridge b datapath_type=netdev
ovs-vsctl set Open . other_config:pmd-cpu-mask=0x30
ovs-vsctl set Open . other_config:n-handler-threads=1
ovs-vsctl set Open . other_config:n-revalidator-threads=1
ovs-vsctl set Open . other_config:max-idle=10000
ovs-vsctl add-port b dpdk0 -- set Interface dpdk0 type=dpdk
ovs-ofctl del-flows b

ovs-ofctl add-flow b in_port=1,actions=in_port
popd
