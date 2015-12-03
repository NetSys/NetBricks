#!/bin/zsh

echo "Building"

echo $$ | sudo tee /mnt/cpuset/sn/tasks

SOFTNIC_DIR=/home/apanda/softnic/softnic

#PFX=$1
#echo "Prefix $PFX"


base=5000000000
end=30000000000
inc=1000000000
duration=10
rib=$HOME/data/mf10krib2-dedup

for ((chain=4;chain>=4;chain-=1)) 
do
for ((bw=$base;bw<=$end;bw+=inc))
do
	for ((iter=0;iter<3;iter++))
	do
		out="logs/ntouch-$bw-$chain-$iter.txt"
		echo "Testing with isolation ${bw} iter ${iter}"
		sudo SCENARIO=s2v2s unbuffer $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l $chain -w 1 -r 4 -t 4 -b $bw -d $duration | tee "$out" &
		SOFTNIC_PID=$!
		sleep 1
		for ((component=0; component<$chain; component++)); do
			ddlog="logs/iso-$bw-$chain-$iter-$component.txt"
			vport="vport$component"
			(( core=1 + $component ))
			echo "Starting component $component on core $core vport $vport"
			sudo unbuffer $HOME/softnic/libsn/iso_test -i $vport -o $vport -c $core -l -r "/mnt/tmp/mf10krib2-dedup$comonent" | tee "$ddlog" &
			sleep 2
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
