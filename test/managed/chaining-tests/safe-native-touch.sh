#!/bin/zsh

echo "Building"

echo $$ | sudo tee /mnt/cpuset/sn/tasks

SOFTNIC_DIR=/home/apanda/softnic/softnic

#PFX=$1
#echo "Prefix $PFX"


base=5000000000
end=50000000000
inc=1000000000
duration=10
rib=$HOME/data/mf10krib2-dedup
chain=2
sizes=(64 128 256 512 1024 1514)
for size in $sizes 
do
for ((bw=$base;bw<=$end;bw+=inc))
do
	for ((iter=0;iter<3;iter++))
	do
		out="logs/ntouch-$bw-$size-$iter.txt"
		echo "Testing with isolation ${bw} iter ${iter}"
		sudo SCENARIO=s2v2s unbuffer $SOFTNIC_DIR/softnic -c 5,6,7 -- -l $chain -w 1 -r 3 -t 3 -b $bw -d $duration -s $size | tee "$out" &
		SOFTNIC_PID=$!
		sleep 1
		for ((component=0; component<$chain; component++)); do
			ddlog="logs/iso-$bw-$chain-$iter-$component.txt"
			vport="vport$component"
			(( core=1 + $component ))
			echo "Starting component $component on core $core vport $vport"
			sudo unbuffer $HOME/softnic/libsn/iso_test -i $vport -o $vport -c $core -t | tee "$ddlog" &
			sleep 1
		done
		echo "Waiting for $SOFTNIC_PID"
		wait $SOFTNIC_PID
		sudo pkill -9 iso_test
		sudo pkill softnic
		while [[ `pidof iso_test` != "" ]]; do
			echo "Waiting for iso_test to die"
		done
	done
done
done
