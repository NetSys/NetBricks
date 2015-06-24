#!/bin/zsh

echo "Building"

echo $$ | sudo tee /mnt/cpuset/sn/tasks

SOFTNIC_DIR=/home/apanda/softnic/softnic

msb Build.proj 
echo "Done building"
#PFX=$1
#echo "Prefix $PFX"


base=5000000000
end=20000000000
inc=1000000000
duration=10
mems=(1M 2M 4M 8M 16M 32M 64M 128M 256M 512M 1024M 2048M)
mem=8M
bw=10000000000
allocs=(32 64 128 256 512 1024 2048 4096 8192 16384 32768 65536)

for alloc in $allocs
do
		for iter in {0..0}
		do
			out="logs/fgc-$bw-$iter-$alloc.txt"
			echo "Testing fixed ${bw} ${alloc} Iter $iter"
			sudo SCENARIO=v2s2v $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d $duration -r 4 -t 4 -b $bw | tee "$out" &
			SOFTNIC_PID=$!
			sudo LD_PRELOAD=libintel_dpdk.so MONO_GC_PARAMS="nursery-size=$mem" /home/apanda/mono-bin/bin/mono --gc=sgen --llvm bin/FixedGC.exe -r 4 -t 4 -- $alloc&
			echo "Waiting for $SOFTNIC_PID"
			wait $SOFTNIC_PID
			sudo pkill mono
		done
		for iter in {0..0}
		do
			out="logs/dgc-$bw-$iter-$alloc.txt"
			echo "Testing dynamic ${bw} ${alloc} Iter $iter mem $mem"
			sudo SCENARIO=v2s2v $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d $duration -r 4 -t 4 -b $bw | tee "$out" &
			SOFTNIC_PID=$!
			sudo LD_PRELOAD=libintel_dpdk.so MONO_GC_PARAMS="nursery-size=$mem" /home/apanda/mono-bin/bin/mono --gc=sgen --llvm bin/DynamicGC.exe -r 4 -t 4 -- $alloc&
			echo "Waiting for $SOFTNIC_PID"
			wait $SOFTNIC_PID
			sudo pkill mono
		done
done
