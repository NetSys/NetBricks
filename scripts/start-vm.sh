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
#ovs-vsctl add-port b dpdk1 -- set Interface dpdk1 type=dpdk
ovs-vsctl add-port b v0 -- set Interface v0 type=dpdkvhostuser
#ovs-vsctl add-port b v1 -- set Interface v1 type=dpdkvhostuser
ovs-ofctl del-flows b

ovs-ofctl add-flow b in_port=1,actions=output:2
ovs-ofctl add-flow b in_port=2,actions=output:1
#ovs-ofctl add-flow b in_port=3,actions=output:1
#ovs-ofctl add-flow b in_port=4,actions=output:2
popd

taskset 0x1c0 qemu-system-x86_64 --enable-kvm -cpu host -smp 2,cores=2,threads=1,sockets=1 -hda $VM_HOME/linux-ovs.img -vnc :2,password -m 8192M \
		-object memory-backend-file,id=mem,size=8192M,mem-path=/dev/hugepages,share=on -numa node,memdev=mem \
		-mem-prealloc -netdev user,id=user.0,hostfwd=tcp::8000-:22 -device e1000,netdev=user.0 -vga std \
		-monitor telnet:127.0.0.1:1234,server,nowait -chardev socket,id=char0,path=$OVS_HOME/_run/run/v0 \
		-netdev type=vhost-user,id=v0,chardev=char0,vhostforce -device virtio-net-pci,mac=00:16:3d:22:33:56,netdev=v0
		#-chardev socket,id=char1,path=$OVS_HOME/_run/run/v1 -netdev type=vhost-user,id=v1,chardev=char1,vhostforce \
		#-device virtio-net-pci,mac=00:16:3d:22:33:57,netdev=v1
