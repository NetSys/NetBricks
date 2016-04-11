#!/bin/bash
# Arguments
# 1 start or stop
# 2 name
# 3 delay
# 4 master lcore
# 5 receive core
# 6 interface

cmd=$1

case $cmd in
	start)
		if [ "$#" -ne 6 ]; then
			echo "Incorrect arguments $#"
			echo "$0 start name delay mcore rcore iface"
			exit 1
		fi
		name=$2
		delay=$3
		mcore=$4
		rcore=$5
		iface=$6
		docker run -d --privileged --cidfile="${name}.cid" \
			--name=${name} \
			--cpuset-cpus="${mcore},${rcore}" \
			-e DELAY=$delay \
			-e MCORE=$mcore \
			-e RCORE=$rcore \
			-e IFACE=$iface \
			-v /sys/bus/pci/drivers:/sys/bus/pci/drivers \
			-v /sys/kernel/mm/hugepages:/sys/kernel/mm/hugepages \
			-v /mnt/huge/:/mnt/huge/ \
			-v /dev:/dev \
			-v /sys/devices/system/node:/sys/devices/system/node \
			-v /var/run:/var/run e2d2/zcsi:0.2
	;;
	stop)
		if [ "$#" -ne 2 ]; then
			echo "Incorrect arguments"
			exit 1
		fi
		name=$2
		if [ ! -e "${name}.cid" ]; then
			echo "Could not find container ${name}"
			exit 1
		fi
		docker kill `cat "${name}.cid"`
		rm ${name}.cid
		docker rm ${name}
	;;
esac
