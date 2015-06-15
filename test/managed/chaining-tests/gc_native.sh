#!/bin/zsh

echo "Building"

echo $$ | sudo tee /mnt/cpuset/sn/tasks

SOFTNIC_DIR=/home/apanda/softnic/softnic

#msb Build.proj 
#echo "Done building"
#PFX=$1
#echo "Prefix $PFX"


base=500000000
end=20000000000
inc=500000000

for bw in `seq $base $inc $end`
do
	for iter in {0..2}
	do
		out="logs/basealloc-$bw-$iter.txt"
		echo "Testing baseline ${bw} Iter $iter"
		sudo SCENARIO=v2s2v $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d 15 -r 4 -t 4 -b $bw | tee "$out" &
		SOFTNIC_PID=$!
		sleep 1
		sudo $SOFTNIC_DIR/../libsn/alloc_test -i vport0 -o vport1 -c 2 &
		echo "Waiting for $SOFTNIC_PID"
		wait $SOFTNIC_PID
		sudo pkill alloc_test
	done
	for iter in {0..2}
	do
		out="logs/falloc-$bw-$iter.txt"
		echo "Testing fixed ${bw} Iter $iter"
		sudo SCENARIO=v2s2v $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d 15 -r 4 -t 4 -b $bw | tee "$out" &
		SOFTNIC_PID=$!
		sleep 1
		sudo $SOFTNIC_DIR/../libsn/alloc_test -i vport0 -o vport1 -c 2 -f &
		echo "Waiting for $SOFTNIC_PID"
		wait $SOFTNIC_PID
		sudo pkill alloc_test
	done
	for iter in {0..2}
	do
		out="logs/dalloc-$bw-$iter.txt"
		echo "Testing dynamic ${bw} Iter $iter"
		sudo SCENARIO=v2s2v $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d 15 -r 4 -t 4 -b $bw | tee "$out" &
		SOFTNIC_PID=$!
		sleep 1
		sudo $SOFTNIC_DIR/../libsn/alloc_test -i vport0 -o vport1 -c 2 -d &
		echo "Waiting for $SOFTNIC_PID"
		wait $SOFTNIC_PID
		sudo pkill alloc_test
	done
done

