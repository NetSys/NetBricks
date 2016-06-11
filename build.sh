#!/bin/bash
# Stop on any errors
set -e
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"

EXT_BASE="$BASE_DIR/3rdparty"
TOOLS_BASE="$BASE_DIR/3rdparty/tools"
DOWNLOAD_DIR="${BASE_DIR}/3rdparty/downloads"
SCRIPTS_DIR="${EXT_BASE}/scripts"
BIN_DIR="${TOOLS_BASE}/bin"
if [ ! -e $DOWNLOAD_DIR ]; then
	mkdir -p ${DOWNLOAD_DIR}
fi
if [ ! -e ${TOOLS_BASE} ]; then
	mkdir -p ${TOOLS_BASE}
fi

DPDK="$BASE_DIR/3rdparty/dpdk/build/lib/libdpdk.a"

CARGO="${TOOLS_BASE}/bin/cargo"
CARGO_HOME="${TOOLS_BASE}/cargo"

MUSL_DOWNLOAD_PATH="${DOWNLOAD_DIR}/musl.tar.gz"
MUSL_RESULT="${EXT_BASE}/musl"
MUSL_TEST="${TOOLS_BASE}/lib/libc.a"

RUST_TEST="${TOOLS_BASE}/bin/rustc.sh"
RUST_DOWNLOAD_PATH="${EXT_BASE}/rust"

LLVM_DOWNLOAD_PATH="${DOWNLOAD_DIR}/llvm.tar.gz"
LLVM_RESULT="${EXT_BASE}/llvm"
UNWIND_RESULT="${TOOLS_BASE}/lib/libunwind.a"
UNWIND_BUILD="${TOOLS_BASE}"/libunwind

deps () {
	# Build DPDK
	if [ ! -e $DPDK ]; then
		dpdk
	else
		echo "DPDK found not building"
	fi

	if [ ! -e ${MUSL_TEST} ]; then
		musl
	else
		echo "Musl found, not building"
	fi

	if [ ! -e ${UNWIND_RESULT} ]; then
		libunwind
	else
		echo "libunwind found, not building"
	fi
	
	if [ ! -e ${RUST_TEST} ]; then
		rust
	else
		echo "Rust found not building"
	fi

	if [ ! -e $CARGO ]; then
		cargo
	else
		echo "Cargo found, not building"
	fi
}

clean_deps() {
	echo "Cleaning dependencies"
	echo "Clean Cargo"
	make -C ${CARGO_HOME} clean
	echo "Clean native"
	make -C $BASE_DIR/native clean || true

	echo "Remove Rust"
	make -C ${RUST_DOWNLOAD_PATH} uninstall
	make -C ${RUST_DOWNLOAD_PATH} clean

	echo "Remove libunwind"
	rm ${TOOLS_BASE}/lib/libunwind.a
	rm -rf ${UNWIND_BUILD}

	echo "Remove DPDK"
	rm $BASE_DIR/3rdparty/dpdk.tar.gz || true
	rm -rf $BASE_DIR/3rdparty/dpdk || true

	echo "Cleaned deps"
}

dpdk () {
	$BASE_DIR/3rdparty/get-dpdk.sh ${DOWNLOAD_DIR}
	proc="$(nproc)"
}

cargo () {
	if [ ! -e $CARGO_HOME ]; then
		git clone https://github.com/apanda/cargo $CARGO_HOME
	else
		pushd $CARGO_HOME
		git pull
		popd
	fi
	# Build cargo
	if [ ! -e $CARGO_HOME/src/rust-installer/gen-installer.sh ]; then
		git clone https://github.com/rust-lang/rust-installer.git \
			$CARGO_HOME/src/rust-installer
	fi
	pushd $CARGO_HOME
	./configure --prefix=${TOOLS_BASE} \
		--local-rust-root=${TOOLS_BASE}
	make -j
	make install
	popd
}

musl () {
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

libunwind () {
	curl http://llvm.org/releases/3.7.0/llvm-3.7.0.src.tar.xz \
		-o "${LLVM_DOWNLOAD_PATH}"
	mkdir -p ${LLVM_RESULT}
	tar xf "${LLVM_DOWNLOAD_PATH}" \
		-C "${LLVM_RESULT}" --strip-components=1
	UNWIND_DOWNLOAD="${DOWNLOAD_DIR}"/unwind.tar.gz
	UNWIND_DIR="${LLVM_RESULT}/projects/libunwind"
	mkdir -p ${UNWIND_DIR}
	curl http://llvm.org/releases/3.7.0/libunwind-3.7.0.src.tar.xz \
		-o "${UNWIND_DOWNLOAD}"
	tar xf "${UNWIND_DOWNLOAD}" -C "${UNWIND_DIR}" --strip-components=1
	mkdir -p "${UNWIND_BUILD}"
	pushd ${UNWIND_BUILD}
	cmake -DLLVM_PATH="${LLVM_RESULT}" -DLIBUNWIND_ENABLE_SHARED=0 \
		"${UNWIND_DIR}"
	make -j
	mkdir -p ${TOOLS_BASE}/lib
	cp lib/libunwind.a ${TOOLS_BASE}/lib
	popd
}

rust () {
	if [ ! -d ${RUST_DOWNLOAD_PATH} ]; then
		git clone https://github.com/rust-lang/rust.git \
			${RUST_DOWNLOAD_PATH}
	fi
	pushd ${RUST_DOWNLOAD_PATH}
	./configure --target=x86_64-unknown-linux-musl \
		--musl-root=${TOOLS_BASE} --prefix=${TOOLS_BASE} \
		--enable-optimize --disable-valgrind \
		--disable-docs
	popd
	make -j -C ${RUST_DOWNLOAD_PATH}
	make -j -C ${RUST_DOWNLOAD_PATH} install
	cp ${SCRIPTS_DIR}/rust*.sh ${BIN_DIR}/
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

		make -j $proc -C $BASE_DIR/native

		pushd $BASE_DIR/framework
		${CARGO} build --release
		popd

		pushd $BASE_DIR/test/framework-test
		${CARGO} build --release 
		popd
		
		pushd $BASE_DIR/test/delay-test
                ${CARGO} build --release
                popd

		pushd $BASE_DIR/test/chain-test
                ${CARGO} build --release
                popd

		pushd $BASE_DIR/test/lpm
                ${CARGO} build --release
                popd

		pushd $BASE_DIR/test/nat
                ${CARGO} build --release
                popd

		pushd $BASE_DIR/test/maglev
                ${CARGO} build --release
                popd
		;;
	fmt)
		deps
		pushd $BASE_DIR/framework
		${CARGO} fmt
		popd

		pushd $BASE_DIR/test/framework-test
		${CARGO} fmt
		popd

		pushd $BASE_DIR/test/delay-test
		${CARGO} fmt
		popd

		pushd $BASE_DIR/test/chain-test
                ${CARGO} fmt
                popd

		pushd $BASE_DIR/test/lpm
                ${CARGO} fmt
                popd

		pushd $BASE_DIR/test/nat
                ${CARGO} fmt
                popd

		pushd $BASE_DIR/test/maglev
                ${CARGO} fmt
                popd
		;;
	doc)
		deps
		pushd $BASE_DIR/framework
		${CARGO} rustdoc -- \
			--no-defaults --passes "collapse-docs" --passes \
				"unindent-comments" 
		popd
		;;
	lint)
		deps
		pushd $BASE_DIR/framework
		${CARGO} clean
		${CARGO} build --features dev
		popd
		;;
	dist_clean) 
		clean_deps
		;&
	clean)
		pushd $BASE_DIR/framework
		${CARGO} clean || true
		popd

		pushd $BASE_DIR/test/framework-test
		${CARGO} clean || true
		popd
		
		pushd $BASE_DIR/test/delay-test
                ${CARGO} clean || true
                popd

		pushd $BASE_DIR/test/chain-test
                ${CARGO} clean || true
                popd

		pushd $BASE_DIR/test/lpm
                ${CARGO} clean || true
                popd

		pushd $BASE_DIR/test/nat
                ${CARGO} clean || true
                popd

		pushd $BASE_DIR/test/maglev
                ${CARGO} clean || true
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
