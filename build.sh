#!/bin/bash
# Stop on any errors
set -e
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"

# Build DPDK
$BASE_DIR/3rdparty/get-dpdk.sh
proc="$(nproc)"
make -j $proc -C native

# Build cargo
pushd $BASE_DIR/cargo
cargo build --release
popd

pushd $BASE_DIR/framework
$BASE_DIR/cargo/target/release/cargo build --release
popd

pushd $BASE_DIR/test/framework-test
$BASE_DIR/cargo/target/release/cargo build --release
popd
