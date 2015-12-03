#!/bin/zsh

echo "Building"
SOFTNIC_DIR=/home/apanda/softnic/softnic

for i in {chain-green-thread,chain-multi-core,chain-same-core}/*.cs
do
	 mcs -unsafe+ -checked- $i VirtualFunctions.cs llring.cs E2D2Iface.cs SoftNic.cs ../lookup-test/iplookup.cs
done
echo "Done building"
PFX=$1
echo "Prefix $PFX"

for cores in {0..2}
do
	for exe in {chain-green-thread,chain-multi-core,chain-same-core}/{NoOpTest,NoOpTestCopy}.exe 
	do
	 out="${exe:r}-$PFX-$cores.txt"
	 echo "Testing $exe writing to $out"
	 sudo SCENARIO=v2s2v $SOFTNIC_DIR/softnic -c 1 -- -l 1 -d 15 | tee "$out" &
	 SOFTNIC_PID=$!
	 sudo LD_PRELOAD=libintel_dpdk.so mono --llvm $exe $cores &
	 echo "Waiting for $SOFTNIC_PID"
	 wait $SOFTNIC_PID
	 sudo pkill mono
	done

	sudo pkill softnic
	sudo pkill mono

	for exe in {chain-green-thread,chain-multi-core,chain-same-core}/{IPLookup,IPLookupCopy}.exe 
	do
	 out="${exe:r}-$PFX-$cores.txt"
	 echo "Testing $exe writing to $out"
	 sudo SCENARIO=v2s2v $SOFTNIC_DIR/softnic -c 1 -- -l 1 -d 15 | tee "$out" &
	 SOFTNIC_PID=$!
	 sudo LD_PRELOAD=libintel_dpdk.so mono --llvm $exe ~/data/mf10krib $cores &
	 wait $SOFTNIC_PID
	 sudo pkill mono
	done
done
