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
sizes=(64 128 256 512 1024 1514)
for size in $sizes 
do
bw=40000000000
	for ((chain=64;chain>=4;chain/=2))
	do
		for ((iter=0;iter<3;iter++))
		do
			out="logs/isoip-$bw-$chain-$size-$iter.txt"
			echo "Testing with isolation ${bw} iter ${iter}"
			sudo SCENARIO=s2v2s $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d $duration -r 4 -t 4 -b $bw -s $size -l 1 -w 1| tee "$out" &
			SOFTNIC_PID=$!
			sudo LD_PRELOAD=libintel_dpdk.so /home/apanda/mono-bin/bin/mono --gc=sgen --llvm bin/IPLookupSafe.exe -r 4 -t 4 -- $HOME/data/mf10krib2-dedup $chain&
			echo "Waiting for $SOFTNIC_PID"
			wait $SOFTNIC_PID
			sudo pkill mono
			while [[ `pidof mono` != "" ]]; do
				echo "Waiting for mono to die"
			done

			out="logs/nativecsip-$bw-$chain-$size-$iter.txt"
			echo "Testing with isolation ${bw} iter ${iter}"
			(( delay=($chain*10) + 60 ))
			sudo SCENARIO=s2v2s unbuffer $SOFTNIC_DIR/softnic -c 0,1,2,3 -- -l $chain -w 1 -r 4 -t 4 -b $bw -s $size -d $duration -x $delay | tee "$out" &
			SOFTNIC_PID=$!
			sleep 1
			flc=4
			base_core=4
            pipeline=0
            (( base=$pipeline*100 ))
			(( pcore=$base_core + $pipeline ))
			for ((component=0; component<$chain; component++)); do
				(( vpc=$base + $component ))
				vport="vport$vpc"
				echo "Starting component $component on core $pipeline vport $vport"
				sudo unbuffer $HOME/softnic/libsn/iso_test -e -i $vport -o $vport -c $pcore -v $flc -l -r "/mnt/tmp/mf10krib2-dedup$component" | tee logs/coreshare-$bw-$chain-$size-$iter-$vport &
				(( flc += 1 ))
				sleep 2
			done
			echo "Waiting for $SOFTNIC_PID"
			wait $SOFTNIC_PID
			sudo pkill -9 iso_test
			sudo pkill softnic
			while [[ `pidof iso_test` != "" ]]; do
				echo "Waiting for iso_test to die"
			done

			out="logs/noisoip-$bw-$chain-$size-$iter.txt"
			echo "Testing without isolation ${bw} iter ${iter}"
			sudo SCENARIO=s2v2s $SOFTNIC_DIR/softnic -c 4,5,6,7 -- -l 1 -d $duration -r 4 -t 4 -b $bw -s $size -w 1 | tee "$out" &
			SOFTNIC_PID=$!
			sudo LD_PRELOAD=libintel_dpdk.so /home/apanda/mono-bin/bin/mono --gc=sgen --llvm bin/IPLookup.exe -r 4 -t 4 --  $HOME/data/mf10krib2-dedup $chain&
			echo "Waiting for $SOFTNIC_PID"
			wait $SOFTNIC_PID
			sudo pkill mono
			while [[ `pidof mono` != "" ]]; do
				echo "Waiting for mono to die"
			done
		done
	done
done
