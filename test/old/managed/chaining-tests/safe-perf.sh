#!/bin/zsh

echo "Building"

echo $$ | sudo tee /mnt/cpuset/sn/tasks

SOFTNIC_DIR=/home/apanda/softnic/softnic

msb Build.proj 
echo "Done building"
#PFX=$1
#echo "Prefix $PFX"


base=5000000000
end=30000000000
inc=1000000000
duration=10
rib=$HOME/data/mf10krib

for chain in {1..3}
do
for bw in `seq $base $inc $end`
do
	for iter in {0..2}
	do
		out="logs/isolate-$bw-$chain-$iter.txt"
		echo "Testing with isolation ${bw} iter ${iter}"
		sudo SCENARIO=v2s2v $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d $duration -r 4 -t 4 -b $bw | tee "$out" &
		SOFTNIC_PID=$!
		sudo LD_PRELOAD=libintel_dpdk.so /home/apanda/mono-perf-bin/bin/mono --gc=sgen --llvm bin/IPLookupSafe.exe -r 4 -t 4 -- $rib $chain&
		echo "Waiting for $SOFTNIC_PID"
		wait $SOFTNIC_PID
		sudo pkill mono
	done
	for iter in {0..2}
	do
		out="logs/noisolate-$bw-$chain-$iter.txt"
		echo "Testing without isolation ${bw} iter ${iter}"
		sudo SCENARIO=v2s2v $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d $duration -r 4 -t 4 -b $bw | tee "$out" &
		SOFTNIC_PID=$!
		sudo LD_PRELOAD=libintel_dpdk.so /home/apanda/mono-perf-bin/bin/mono --gc=sgen --llvm bin/IPLookup.exe -r 4 -t 4 -- $rib $chain&
		echo "Waiting for $SOFTNIC_PID"
		wait $SOFTNIC_PID
		sudo pkill mono
	done
done
done
