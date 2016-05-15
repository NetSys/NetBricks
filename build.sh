#!/bin/bash
# Stop on any errors
set -e
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
EXT_BASE="$BASE_DIR/3rdparty"
TOOLS_BASE="$BASE_DIR/3rdparty/tools"
CARGO="${TOOLS_BASE}/cargo/target/release/cargo"
DPDK="$BASE_DIR/3rdparty/dpdk/build/lib/libdpdk.a"
MUSL_TEST="${TOOLS_BASE}/lib/libc.a"
RUST_TEST="${TOOLS_BASE}/bin/rustc"
mkdir -p TOOLS_BASE

deps () {
	# Build DPDK
	if [ ! -e $DPDK ]; then
		dpdk
	else
		echo "DPDK found not building"
	fi
	if [ ! -e $CARGO ]; then
		cargo
	else
		echo "Cargo found, not building"
	fi

	if [ ! -e MUSL_TEST ]; then
		musl
	else
		echo "Musl found, not building"
	fi
}

clean_deps() {
	rm $BASE_DIR/3rdparty/dpdk.tar.gz || true
	rm -rf $BASE_DIR/3rdparty/dpdk || true
	make -C $BASE_DIR/native clean || true
}

dpdk () {
	$BASE_DIR/3rdparty/get-dpdk.sh
	proc="$(nproc)"
	make -j $proc -C $BASE_DIR/native
}

cargo () {
	# Build cargo
	pushd $BASE_DIR/cargo
	cargo build --release
	popd
}

musl () {
	MUSL_DOWNLOAD_PATH="${EXT_BASE}/musl.tar.gz"
	MUSL_RESULT="${EXT_BASE}/musl"
	curl http://www.musl-libc.org/releases/musl-1.1.10.tar.gz \
		-o "${MUSL_DOWNLOAD_PATH}"
	mkdir -p ${MUSL_RESULT}
	tar zxvf "${MUSL_DOWNLOAD_PATH}" \
		-C "${MUSL_RESULT}" --strip-components=1
	pushd ${MUSL_RESULT}
	./configure --disable-shared --prefix="$TOOLS_BASE"
	make -j
	make install
	popd
}

if [ $# -ge 1 ]; then
	TASK=$1
else
	TASK=build
fi

case $TASK in
	deps)
		deps
		;;
	build)
		deps
		pushd $BASE_DIR/framework
		$BASE_DIR/cargo/target/release/cargo build --release
		popd

		pushd $BASE_DIR/test/framework-test
		$BASE_DIR/cargo/target/release/cargo build --release 
		popd
		
		pushd $BASE_DIR/test/delay-test
                $BASE_DIR/cargo/target/release/cargo build --release
                popd

		pushd $BASE_DIR/test/chain-test
                $BASE_DIR/cargo/target/release/cargo build --release
                popd

		pushd $BASE_DIR/test/lpm
                $BASE_DIR/cargo/target/release/cargo build --release
                popd

		pushd $BASE_DIR/test/nat
                $BASE_DIR/cargo/target/release/cargo build --release
                popd

		pushd $BASE_DIR/test/maglev
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

		pushd $BASE_DIR/test/delay-test
		$BASE_DIR/cargo/target/release/cargo fmt
		popd

		pushd $BASE_DIR/test/chain-test
                $BASE_DIR/cargo/target/release/cargo fmt
                popd

		pushd $BASE_DIR/test/lpm
                $BASE_DIR/cargo/target/release/cargo fmt
                popd

		pushd $BASE_DIR/test/nat
                $BASE_DIR/cargo/target/release/cargo fmt
                popd

		pushd $BASE_DIR/test/maglev
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
	dist_clean) 
		clean_deps
		;;
	clean)
		pushd $BASE_DIR/framework
		$BASE_DIR/cargo/target/release/cargo clean || true
		popd

		pushd $BASE_DIR/test/framework-test
		$BASE_DIR/cargo/target/release/cargo clean || true
		popd
		
		pushd $BASE_DIR/test/delay-test
                $BASE_DIR/cargo/target/release/cargo clean || true
                popd

		pushd $BASE_DIR/test/chain-test
                $BASE_DIR/cargo/target/release/cargo clean || true
                popd

		pushd $BASE_DIR/test/lpm
                $BASE_DIR/cargo/target/release/cargo clean || true
                popd

		pushd $BASE_DIR/test/nat
                $BASE_DIR/cargo/target/release/cargo clean || true
                popd

		pushd $BASE_DIR/test/maglev
                $BASE_DIR/cargo/target/release/cargo clean || true
                popd
		;;
	*)
		echo "./build.sh <Command>
		Where command is one of
		build: Build the project
		doc: Run rustdoc and produce documentation
		fmt: Run rustfmt to format text prettily.
		lint: Run clippy to lint the project
		"
		;;
esac
