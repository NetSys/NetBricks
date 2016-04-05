#!/bin/bash
# Stop on any errors
set -e
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"

deps () {
	# Build DPDK
	$BASE_DIR/3rdparty/get-dpdk.sh
    export RTE_TARGET=x86_64-native-linuxapp-gcc
	proc="$(nproc)"
	make -j $proc -C native

	# Build cargo
	pushd $BASE_DIR/cargo
	cargo build --release
	popd
}

if [ $# -ge 1 ]; then
	TASK=$1
else
	TASK=build
fi

case $TASK in
	help)
		echo "./build.sh <Command>
		Where command is one of
		build: Build the project
		doc: Run rustdoc and produce documentation
		fmt: Run rustfmt to format text prettily.
		lint: Run clippy to lint the project
		"
		;;
	build)
		deps
		pushd $BASE_DIR/framework
		$BASE_DIR/cargo/target/release/cargo build --release
		popd

		pushd $BASE_DIR/test/framework-test
		$BASE_DIR/cargo/target/release/cargo build --release
		popd
		;;
	fmt)
		deps
		pushd $BASE_DIR/framework
		$BASE_DIR/cargo/target/release/cargo fmt
		popd

		pushd $BASE_DIR/test/framework-test
		$BASE_DIR/cargo/target/release/cargo fmt
		popd
		;;
	doc)
		deps
		pushd $BASE_DIR/framework
		$BASE_DIR/cargo/target/release/cargo rustdoc -- \
			--no-defaults --passes "collapse-docs" --passes \
				"unindent-comments" 
		popd
		;;
	lint)
		deps
		pushd $BASE_DIR/framework
		$BASE_DIR/cargo/target/release/cargo clean; $BASE_DIR/cargo/target/release/cargo build --features dev
		popd
		;;
esac
