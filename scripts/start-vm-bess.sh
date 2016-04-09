#!/bin/bash
set -x
set -o errexit
OVS_HOME=/opt/e2d2/ovs
BESS_HOME=/opt/e2d2/bess
VM_HOME=/opt/e2d2/vm
VHOST_HOME=/tmp
HUGEPAGES_HOME=/opt/e2d2/libhugetlbfs

LD_PRELOAD="$HUGEPAGES_HOME/obj64/libhugetlbfs.so" $BESS_HOME/bin/bessd -k
$BESS_HOME/bin/bessctl run file $BESS_HOME/bessctl/conf/samples/fwd.bess
sleep 10
taskset 0x1c0 qemu-system-x86_64 --enable-kvm -cpu host -smp 2,cores=2,threads=1,sockets=1 -hda $VM_HOME/linux-bess.img -vnc :2,password -m 8192M \
		-object memory-backend-file,id=mem,size=8192M,mem-path=/dev/hugepages,share=on -numa node,memdev=mem \
		-mem-prealloc -netdev user,id=user.0,hostfwd=tcp::8000-:22 -device e1000,netdev=user.0 -vga std \
		-monitor telnet:127.0.0.1:1234,server,nowait -chardev socket,id=char0,path=/tmp/sn_vhost_v0 \
		-netdev type=vhost-user,id=v0,chardev=char0,vhostforce -device virtio-net-pci,mac=00:16:3d:22:33:56,netdev=v0
$BESS_HOME/bin/bessctl daemon stop
rm -f /tmp/sn_vhost_v*
