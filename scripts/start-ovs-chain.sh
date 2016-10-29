#!/bin/bash
set -o errexit
# args
# 1: Master core (5)
# 2: PMD mask (0x60)
# 4: Number of rings (1)

BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
OVS_HOME="$BASE_DIR/../ovs"
DPDK_LIB="$BASE_DIR/../dpdk/build/lib"
export LD_LIBRARY_PATH="${DPDK_LIB}:${LD_LIBRARY_PATH}"
echo $OVS_HOME
#HUGEPAGES_HOME=/opt/e2d2/libhugetlbfs

INP_LCORE=${1-"4"}
MASTER_LCORE=$((INP_LCORE - 1))
CORE0=$((MASTER_LCORE + 1))
CORE1=$((MASTER_LCORE + 2))
PMD_MASK=$(printf "0x%x" $((2**(CORE0 - 1) + 2**(CORE1 - 1))))
#PMD_MASK=${2-"0x60"}
PHYNICS=1
RINGS=${2-1}
${BASE_DIR}/kill-ovs-chain.py

pushd $OVS_HOME
$( $OVS_HOME/utilities/ovs-dev.py env )
#$OVS_HOME/utilities/ovs-dev.py kill
$OVS_HOME/utilities/ovs-dev.py \
    reset run --dpdk -c ${MASTER_LCORE} -n 4 -r 1 --socket-mem 1024,0 \
	--file-prefix "ovs" -w 07:00.0 -w 07:00.1 -w 07:00.2 -w 07:00.3
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

for (( rinterface=0; rinterface<$RINGS; rinterface++ )); do
	iface="dpdkr${rinterface}"
	echo "Setting up DPDK ring interface ${iface}"
	ovs-vsctl add-port b ${iface} -- set Interface ${iface} type=dpdkr
done
ovs-ofctl del-flows b

ports=$((PHYNICS+RINGS))
for (( port=0; port<$((ports - 1)); port++ )); do
    src_port=$((port+1))
    dst_port=$((port+2))
    echo ovs-ofctl add-flow b in_port=${src_port},actions=output:${dst_port}
    ovs-ofctl add-flow b in_port=${src_port},actions=output:${dst_port}
done
ovs-ofctl add-flow b in_port=${ports},actions=output:1
odd=$((RINGS%2))
echo "PMD mask=${PMD_MASK}"
CONTAINER_CORE=$((CORE1 + 0)) # Cores here are 0 numbered
if ((odd==1)); then
    sudo docker run -d --privileged --cpuset-cpus="${MASTER_LCORE},${CONTAINER_CORE}" -v /sys/bus/pci/drivers:/sys/bus/pci/drivers -v \
    /sys/kernel/mm/hugepages:/sys/kernel/mm/hugepages -v /mnt/huge:/mnt/huge -v /dev:/dev -v \
    /sys/devices/system/node:/sys/devices/system/node -v /var/run:/var/run netbricks:vswitch /opt/netbricks/build.sh run zcsi-chain \
    --secondary -n ovs -l 1 -m ${MASTER_LCORE} -c ${CONTAINER_CORE} -p ovs:0
else
    sudo docker run -d --privileged --cpuset-cpus="${MASTER_LCORE},${CONTAINER_CORE}" -v /sys/bus/pci/drivers:/sys/bus/pci/drivers -v \
    /sys/kernel/mm/hugepages:/sys/kernel/mm/hugepages -v /mnt/huge:/mnt/huge -v /dev:/dev -v \
    /sys/devices/system/node:/sys/devices/system/node -v /var/run:/var/run netbricks:vswitch /opt/netbricks/build.sh run zcsi-chain \
    --secondary -n ovs -l 1 -m ${MASTER_LCORE} -c ${CONTAINER_CORE} -p ovs:0 -j 1
fi

for (( ctr=1; ctr<$RINGS; ctr++ )); do
    CORE=$((CONTAINER_CORE + ctr))
    sudo docker run -d --privileged --cpuset-cpus="${MASTER_LCORE},${CORE}" -v /sys/bus/pci/drivers:/sys/bus/pci/drivers -v \
    /sys/kernel/mm/hugepages:/sys/kernel/mm/hugepages -v /mnt/huge:/mnt/huge -v /dev:/dev -v \
    /sys/devices/system/node:/sys/devices/system/node -v /var/run:/var/run netbricks:vswitch /opt/netbricks/build.sh run zcsi-chain \
    --secondary -n ovs -l 1 -m ${MASTER_LCORE} -c ${CORE} -p ovs:${ctr}
done
popd
