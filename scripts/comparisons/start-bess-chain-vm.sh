#!/bin/bash
set -o errexit
# args
# 1: Master core (4)
# 2: Number of rings (1)

BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
BESS_HOME="$BASE_DIR/../bess"
DPDK_LIB="$BASE_DIR/../dpdk/build/lib"
export LD_LIBRARY_PATH="${DPDK_LIB}:${LD_LIBRARY_PATH}"

INP_LCORE=${1-"4"}
MASTER_LCORE=$((INP_LCORE - 1))
CORE0=$((MASTER_LCORE + 1))
CORE1=$((MASTER_LCORE + 2))
PHYNICS=1
RINGS=${2-1}
IFACE=${BENCH_IFACE-"07:00.0"}
BESS_CORE0=${CORE0} BESS_CORE1=${CORE1} BESS_IFACE=${IFACE} BESS_CHAIN=${RINGS} ${BESS_HOME}/bin/bessctl daemon start \
                                                -- run file ${BASE_DIR}/vhchain.bess

start_vm () {
    MCORE=$1
    LCORE=$2
    VDEV=$3
    PORT=$4
    MONITOR_PORT=$5
    DISPLAY=$6
    echo "Using VDEV ${VDEV}"

    CORE_MASK=$(printf "0x%x" $((2**(MCORE) + 2**(LCORE))))
    OUT_FILE=$(mktemp)
    echo "Core Mask" ${CORE_MASK}
    sudo taskset ${CORE_MASK} qemu-system-x86_64 --enable-kvm --cpu host,migratable=off --smp 2,cores=2,threads=1,sockets=1 -snapshot -hda \
    ${BASE_DIR}/../debian/debian-nb.img -m 2048M -object memory-backend-file,id=mem,size=2048M,mem-path=/dev/hugepages,share=on -numa \
    node,memdev=mem -mem-prealloc -monitor telnet:127.0.0.1:${MONITOR_PORT},server,nowait -device e1000,netdev=user.0 -netdev \
    user,id=user.0,hostfwd=tcp::${PORT}-:22 -vga std  -serial file:${OUT_FILE} -daemonize -vnc :${DISPLAY},password \
    -chardev socket,id=char0,path=/tmp/v${VDEV} -netdev type=vhost-user,id=v${VDEV},chardev=char0,vhostforce \
    -device virtio-net-pci,mac=00:16:3d:22:33:57,netdev=v${VDEV}
    echo "Out file is ${OUT_FILE}"
    until [ -e ${OUT_FILE} ]; do
        sleep 0.1
    done
    until cat $OUT_FILE | grep "login:"; do
        sleep 0.1
    done
    echo "Booted"
}

start_app () {
    PORT=$1
    extra_args=$2
    ssh -p${PORT} -i ${BASE_DIR}/../debian/vm_key -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no root@localhost \
        /opt/netbricks/scripts/bind-virtio.sh
    devs=$( ssh -p${PORT} -i ${BASE_DIR}/../debian/vm_key -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no root@localhost \
            /opt/netbricks/3rdparty/dpdk/tools/dpdk-devbind.py --status \
            | grep 'Virtio' \
            | awk '{print $1}' )
    echo "Active ports are $devs"
    ports_string=""
    for dev in $devs; do
        ports_string+="-p ${dev} -c 1"
    done
    ssh -p${PORT} -i ${BASE_DIR}/../debian/vm_key -o UserKnownHostsFile=/dev/null \
        -o StrictHostKeyChecking=no root@localhost \
    "nohup /opt/netbricks/build.sh run zcsi-chain -l 1 -m 0 ${ports_string} ${extra_args} &> /var/log/nb < /dev/null &"
}
BASE_CORE=$(( CORE1 + 1 ))
BASE_VDEV=0
BASE_PORT=5555
BASE_MONITOR=1234
BASE_DISPLAY=1
start_vm ${BASE_CORE} $(( BASE_CORE + 1)) ${BASE_VDEV} ${BASE_PORT} ${BASE_MONITOR} ${BASE_DISPLAY}
if ((odd == 1)); then
    start_app ${BASE_PORT} ""
else
    start_app ${BASE_PORT} "-j 1"
fi

for (( vm = 1; vm < ${RINGS}; vm++ )); do
    PORT=$(( BASE_PORT + vm ))
    CORE=$(( BASE_CORE + 2*vm ))
    start_vm ${CORE} $(( CORE + 1 )) $(( BASE_VDEV + vm )) $PORT $(( BASE_MONITOR + vm )) \
        $(( BASE_DISPLAY + vm ))
    start_app ${PORT}
done
