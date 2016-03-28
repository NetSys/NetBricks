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

if [ $# -ge 1 ]; then
	TASK=$1
else
	TASK=build
fi

case $TASK in
	build)
		pushd $BASE_DIR/framework
		$BASE_DIR/cargo/target/release/cargo build --release
		popd

		pushd $BASE_DIR/test/framework-test
		$BASE_DIR/cargo/target/release/cargo build --release
		popd
		;;
	fmt)
		pushd $BASE_DIR/framework
		$BASE_DIR/cargo/target/release/cargo fmt
		popd

		pushd $BASE_DIR/test/framework-test
		$BASE_DIR/cargo/target/release/cargo fmt
		popd
		;;
	doc)
		pushd $BASE_DIR/framework
		$BASE_DIR/cargo/target/release/cargo rustdoc -- \
			--no-defaults --passes "collapse-docs" --passes \
				"unindent-comments" 
		popd
		;;
	lint)
		pushd $BASE_DIR/framework
		$BASE_DIR/cargo/target/release/cargo clean; $BASE_DIR/cargo/target/release/cargo build --features dev
		popd
		;;
esac
