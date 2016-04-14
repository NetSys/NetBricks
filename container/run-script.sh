#!/bin/bash
ZCSI_ROOT=/opt/e2d2
echo "Delaying for " $DELAY
echo "Using intefaces" ${IFACE[@]}
echo "Master core" $MCORE
echo "Receiving core" $RCORE
IF=( "${IFACE[@]/#/-v }" )
CORES=( )
for i in "${!IFACE[@]}"; do
	CORES[$i]="-c $RCORE"
done
$DELAY_TEST_ROOT/zcsi-delay -m $MCORE ${IF[@]} ${CORES[@]}  --secondary -n rte -d $DELAY
