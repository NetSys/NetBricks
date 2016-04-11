#!/bin/bash
ZCSI_ROOT=/opt/e2d2
echo "Delaying for " $DELAY
echo "Using inteface" $IFACE
echo "Master core" $MCORE
echo "Receiving core" $RCORE
export LD_LIBRARY_PATH=$ZCSI_ROOT/native:$ZCSI_ROOT/3rdparty/dpdk/build/lib
DELAY_TEST_ROOT=$ZCSI_ROOT/test/delay-test/target/release
$DELAY_TEST_ROOT/zcsi-delay -m $MCORE -c $RCORE -v $IFACE --secondary -n rte -d $DELAY
