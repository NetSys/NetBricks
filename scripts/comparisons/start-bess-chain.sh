#!/bin/bash
set -o errexit
# args
# 1: Master core (4)
# 2: Number of rings (1)

BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
BESS_HOME="$BASE_DIR/../bess"
DPDK_LIB="$BASE_DIR/../dpdk/build/lib"
export LD_LIBRARY_PATH="${DPDK_LIB}:${LD_LIBRARY_PATH}"
#HUGEPAGES_HOME=/opt/e2d2/libhugetlbfs

INP_LCORE=${1-"4"}
MASTER_LCORE=$((INP_LCORE - 1))
CORE0=$((MASTER_LCORE + 1))
CORE1=$((MASTER_LCORE + 2))
PHYNICS=1
RINGS=${2-1}
IFACE=${BENCH_IFACE-"07:00.0"}

INP_LCORE=${1-"4"}
MASTER_LCORE=$((INP_LCORE - 1))
CORE0=$((MASTER_LCORE + 1))
CORE1=$((MASTER_LCORE + 2))
PHYNICS=1
RINGS=${2-1}
IFACE=${BENCH_IFACE-"07:00.0"}
${BESS_HOME}/bin/bessd -a -k
BESS_PID=$(cat /var/run/bessd.pid)
echo "bessd pid is ${BESS_PID}"
sleep 5
BESS_CORE0=${CORE0} BESS_CORE1=${CORE1} BESS_IFACE=${IFACE} BESS_CHAIN=${RINGS} ${BESS_HOME}/bin/bessctl \
                                                run file ${BASE_DIR}/vpchain.bess


odd=$((RINGS%2))
echo "PMD mask=${PMD_MASK}"
CONTAINER_CORE=$((CORE1 + 1)) # Cores here are 0 numbered
if ((odd==1)); then
    sudo docker run -d --privileged --cpuset-cpus="${MASTER_LCORE},${CONTAINER_CORE}" -v /sys/bus/pci/drivers:/sys/bus/pci/drivers -v \
    /sys/kernel/mm/hugepages:/sys/kernel/mm/hugepages -v /mnt/huge:/mnt/huge -v /dev:/dev -v \
    /sys/devices/system/node:/sys/devices/system/node -v /var/run:/var/run -v /tmp/sn_vports:/tmp/sn_vports \
    netbricks:vswitch /opt/netbricks/build.sh run zcsi-chain \
    --secondary -n "rte${BESS_PID}" -l 1 -m ${MASTER_LCORE} -c ${CONTAINER_CORE} -p bess:rte_ring0
else
    sudo docker run -d --privileged --cpuset-cpus="${MASTER_LCORE},${CONTAINER_CORE}" -v /sys/bus/pci/drivers:/sys/bus/pci/drivers -v \
    /sys/kernel/mm/hugepages:/sys/kernel/mm/hugepages -v /mnt/huge:/mnt/huge -v /dev:/dev -v \
    /sys/devices/system/node:/sys/devices/system/node -v /var/run:/var/run -v /tmp/sn_vports:/tmp/sn_vports \
    netbricks:vswitch /opt/netbricks/build.sh run zcsi-chain \
    --secondary -n "rte${BESS_PID}" -l 1 -m ${MASTER_LCORE} -c ${CONTAINER_CORE} -p bess:rte_ring0 -j 1
fi

for (( ctr=1; ctr<$RINGS; ctr++ )); do
    CORE=$((CONTAINER_CORE + ctr))
    sudo docker run -d --privileged --cpuset-cpus="${MASTER_LCORE},${CORE}" -v /sys/bus/pci/drivers:/sys/bus/pci/drivers -v \
    /sys/kernel/mm/hugepages:/sys/kernel/mm/hugepages -v /mnt/huge:/mnt/huge -v /dev:/dev -v \
    /sys/devices/system/node:/sys/devices/system/node -v /var/run:/var/run -v /tmp/sn_vports:/tmp/sn_vports \
    netbricks:vswitch /opt/netbricks/build.sh run zcsi-chain \
    --secondary -n "rte${BESS_PID}" -l 1 -m ${MASTER_LCORE} -c ${CORE} -p bess:rte_ring${ctr}
done
