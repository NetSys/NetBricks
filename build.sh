#!/bin/bash
set -e
3rdparty/get-dpdk.sh
proc="$(nproc)"
make -j $proc -C native
pushd framework
cargo build --release
popd
