#!/bin/bash
set -o errexit
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
OVS_HOME="$BASE_DIR/../ovs"
DPDK_LIB="$BASE_DIR/../dpdk/build/lib"
export LD_LIBRARY_PATH="${DPDK_LIB}:${LD_LIBRARY_PATH}"
running=$(docker ps -q)
if [ ! -z "$running" ]; then 
    echo "Killing and removing container"
    docker kill ${running}
    docker rm ${running}
fi
pushd $OVS_HOME
$( $OVS_HOME/utilities/ovs-dev.py env )
$OVS_HOME/utilities/ovs-dev.py kill
popd
