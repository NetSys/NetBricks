#!/bin/zsh

echo "Building"

#echo $$ | sudo tee /mnt/cpuset/sn/tasks

SOFTNIC_DIR=/home/apanda/softnic/softnic

msb Build.proj 
echo "Done building"
#PFX=$1
#echo "Prefix $PFX"


base=5000000000
end=50000000000
inc=1000000000
duration=10

#for chain in {3..4}
#do
chain=2
sizes=(64 128 256 512 1024 1514)
for size in $sizes 
do
bw=40000000000
	for ((chain=4;chain>=1;chain--))
	do
		for ((iter=0;iter<3;iter++))
		do
			out="logs/nip-$bw-$chain-$size-$iter.txt"
			echo "Testing with isolation ${bw} iter ${iter}"
			sudo SCENARIO=s2v2s unbuffer $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l $chain -w 1 -r 4 -t 4 -b $bw -d $duration -s $size -x 60 | tee "$out" &
			SOFTNIC_PID=$!
			sleep 1
			for ((component=0; component<$chain; component++)); do
				vport="vport$component"
				(( core=0 + $component ))
				echo "Starting component $component on core $core vport $vport"
				sudo unbuffer $HOME/softnic/libsn/iso_test -i $vport -o $vport -c $core -l -r "/mnt/tmp/mf10krib2-dedup$comonent" &
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
		for ((iter=0;iter<3;iter++))
		do
			out="logs/nip-u-$chain-$size-$iter.txt"
			echo "Testing with isolation ${bw} iter ${iter}"
			sudo SCENARIO=s2v2s unbuffer $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l $chain -w 1 -r 4 -t 4 -d $duration -s $size -x 60 | tee "$out" &
			SOFTNIC_PID=$!
			sleep 1
			for ((component=0; component<$chain; component++)); do
				vport="vport$component"
				(( core=0 + $component ))
				echo "Starting component $component on core $core vport $vport"
				sudo unbuffer $HOME/softnic/libsn/iso_test -i $vport -o $vport -c $core -l -r "/mnt/tmp/mf10krib2-dedup$comonent" &
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
		for iter in {0..2}
		do
			out="logs/isoip-$bw-$chain-$size-$iter.txt"
			echo "Testing with isolation ${bw} iter ${iter}"
			sudo SCENARIO=s2v2s $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d $duration -r 4 -t 4 -b $bw -s $size -w $chain | tee "$out" &
			SOFTNIC_PID=$!
			sudo LD_PRELOAD=libintel_dpdk.so /home/apanda/mono-bin/bin/mono --gc=sgen --llvm bin/IPSafeT.exe -r 4 -t 4 -- $HOME/data/mf10krib2-dedup $chain $chain&
			echo "Waiting for $SOFTNIC_PID"
			wait $SOFTNIC_PID
			sudo pkill mono
		done
		for iter in {0..2}
		do
			out="logs/noisoip-$bw-$chain-$size-$iter.txt"
			echo "Testing without isolation ${bw} iter ${iter}"
			sudo SCENARIO=s2v2s $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d $duration -r 4 -t 4 -b $bw -s $size -w $chain | tee "$out" &
			SOFTNIC_PID=$!
			sudo LD_PRELOAD=libintel_dpdk.so /home/apanda/mono-bin/bin/mono --gc=sgen --llvm bin/IPLookupT.exe -r 4 -t 4 --  $HOME/data/mf10krib2-dedup $chain $chain&
			echo "Waiting for $SOFTNIC_PID"
			wait $SOFTNIC_PID
			sudo pkill mono
		done
		for iter in {0..2}
		do
			out="logs/isoip-u-$chain-$size-$iter.txt"
			echo "Testing with isolation unlimited iter ${iter}"
			sudo SCENARIO=s2v2s $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d $duration -r 4 -t 4 -s $size -w $chain | tee "$out" &
			SOFTNIC_PID=$!
			sudo LD_PRELOAD=libintel_dpdk.so /home/apanda/mono-bin/bin/mono --gc=sgen --llvm bin/IPSafeT.exe -r 4 -t 4 -- $HOME/data/mf10krib2-dedup $chain $chain&
			echo "Waiting for $SOFTNIC_PID"
			wait $SOFTNIC_PID
			sudo pkill mono
		done
		for iter in {0..2}
		do
			out="logs/noisoip-u-$chain-$size-$iter.txt"
			echo "Testing without isolation unlimited iter ${iter}"
			sudo SCENARIO=s2v2s $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d $duration -r 4 -t 4 -s $size -w $chain | tee "$out" &
			SOFTNIC_PID=$!
			sudo LD_PRELOAD=libintel_dpdk.so /home/apanda/mono-bin/bin/mono --gc=sgen --llvm bin/IPLookupT.exe -r 4 -t 4 --  $HOME/data/mf10krib2-dedup $chain $chain&
			echo "Waiting for $SOFTNIC_PID"
			wait $SOFTNIC_PID
			sudo pkill mono
		done
	done
done
