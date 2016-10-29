#!/bin/bash
set -o errexit
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
OVS_HOME=${BASE_DIR}/../ovs
OUT_FILE=$(mktemp)
CORE_MASK=$(printf "0x%x" $((2**(6-1) + 2**(7-1))))
sudo taskset ${CORE_MASK} qemu-system-x86_64 --enable-kvm --cpu host,migratable=off --smp 2,cores=2,threads=1,sockets=1 -hda \
${BASE_DIR}/../debian/debian-nb.img -m 8192M -object memory-backend-file,id=mem,size=8192M,mem-path=/dev/hugepages,share=on -numa \
node,memdev=mem -mem-prealloc -monitor telnet:127.0.0.1:1234,server,nowait -device e1000,netdev=user.0 -netdev \
user,id=user.0,hostfwd=tcp::5555-:22 -vga std  -serial file:${OUT_FILE} -daemonize -vnc :2,password \
-chardev socket,id=char0,path=${OVS_HOME}/_run/run/v0 -netdev type=vhost-user,id=v0,chardev=char0,vhostforce \
-device virtio-net-pci,mac=00:16:3d:22:33:57,netdev=v0
echo "Out file is ${OUT_FILE}"
until [ -e ${OUT_FILE} ]; do
    sleep 0.1
done
until cat $OUT_FILE | grep "login:"; do
    sleep 0.1
done
echo "Booted"
echo "ssh -p5555 -i ${BASE_DIR}/../debian/vm_key apanda@localhost"
ssh -p5555 -i ${BASE_DIR}/../debian/vm_key root@localhost
