#!/bin/zsh

sudo pkill -9 iso_test
sudo pkill -9 fastforward 
sudo pkill softnic
sleep 1

echo "Building"

#echo $$ | sudo tee /mnt/cpuset/sn/tasks

SOFTNIC_DIR=/home/apanda/softnic/softnic

base=5000000000
end=50000000000
inc=1000000000
duration=10

sizes=(1514 64 128 256 512 1024)
for size in $sizes 
do
bw=40000000000
#bw=700000000
	for ((chain=4;chain>=1;chain--))
	do
		for ((iter=0;iter<3;iter++))
		do
			out="logs/coreshare-$bw-$chain-$size-$iter.txt"
			echo "Testing with isolation ${bw} iter ${iter}"
			sudo SCENARIO=s2v2s unbuffer $SOFTNIC_DIR/softnic -c 0,1,2,3 -- -l $chain -w $chain -r 4 -t 4 -b $bw -s $size -d $duration -x 240 | tee "$out" &
			SOFTNIC_PID=$!
			sleep 1
			flc=4
			base_core=4
            pipeline=0
			for ((pipeline=0; pipeline<$chain; pipeline++)); do
                (( base=$pipeline*100 ))
				(( pcore=$base_core + $pipeline ))
				for ((component=0; component<$chain; component++)); do
					(( vpc=$base + $component ))
					vport="vport$vpc"
					(( cycles=100 ))
					echo "Starting component $component on core $pipeline vport $vport"
					sudo unbuffer $HOME/softnic/libsn/iso_test -e -i $vport -o $vport -c $pcore -v $flc -l -r "/mnt/tmp/mf10krib2-dedup$component" | tee logs/coreshare-$bw-$chain-$size-$iter-$vport &
					(( flc += 1 ))
					sleep 2
				done
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
