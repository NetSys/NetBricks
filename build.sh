#!/bin/bash
# Stop on any errors
set -e
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
BUILD_SCRIPT=$( basename "$0" )

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
DPDK_HOME="${BASE_DIR}/3rdparty/dpdk"
DPDK="${DPDK_HOME}/build/lib/libdpdk.a"

CARGO="${TOOLS_BASE}/bin/cargo"
CARGO_HOME="${TOOLS_BASE}/cargo"

MUSL_DOWNLOAD_PATH="${DOWNLOAD_DIR}/musl.tar.gz"
MUSL_RESULT="${EXT_BASE}/musl"
MUSL_TEST="${TOOLS_BASE}/lib/libc.a"

RUST_TEST="${TOOLS_BASE}/bin/rustc"
RUST_DOWNLOAD_PATH="${EXT_BASE}/rust"

LLVM_DOWNLOAD_PATH="${DOWNLOAD_DIR}/llvm.tar.gz"
LLVM_RESULT="${EXT_BASE}/llvm"
UNWIND_RESULT="${TOOLS_BASE}/lib/libunwind.a"

rust_build_static() {
    if [ ! -d ${RUST_DOWNLOAD_PATH} ]; then
        git clone https://github.com/rust-lang/rust.git \
            ${RUST_DOWNLOAD_PATH}
    else
        pushd ${RUST_DOWNLOAD_PATH}
        git pull
        popd
    fi
    pushd ${RUST_DOWNLOAD_PATH}
    ./configure --target=x86_64-unknown-linux-musl \
        --musl-root=${TOOLS_BASE} --prefix=${TOOLS_BASE} \
        --enable-optimize --disable-valgrind \
        --disable-docs
    popd
    make -j -C ${RUST_DOWNLOAD_PATH}
    make -j -C ${RUST_DOWNLOAD_PATH} install
}

rust_static() {
    echo "Running rust_static"
    if [ ! -e ${MUSL_TEST} ] || [ ! -z ${_BUILD_UPDATE_} ]; then
        musl
    else
        echo "Musl found, not building"
    fi

    if [ ! -e ${UNWIND_RESULT} ] || [ ! -z ${_BUILD_UPDATE_} ]; then
        libunwind
    else
        echo "libunwind found, not building"
    fi

    if [ ! -e ${RUST_TEST} ] || [ ! -z ${_BUILD_UPDATE_} ]; then
        rust_build_static
    else
        echo "Rust found not building"
    fi
    export RUSTC="${TOOLS_BASE}/bin/rustc"
}

rust () {
    echo "Building rust"
    if [ ! -z ${RUST_STATIC} ]; then
        rust_static
    fi
    if [ ! -d ${BIN_DIR} ]; then
        mkdir -p ${BIN_DIR}
    fi
    cp ${SCRIPTS_DIR}/rust*.sh ${BIN_DIR}/
}

toggle_symbols () {
    if [ ! -z ${NETBRICKS_SYMBOLS} ]; then
        find ${BASE_DIR}/test -name Cargo.toml -exec sed -i 's/debug = false/debug = true/g' {} \;
    else
        find ${BASE_DIR}/test -name Cargo.toml -exec sed -i 's/debug = true/debug = false/g' {} \;
    fi
}

find_sctp () {
    set +o errexit
    gcc -lsctp 2>&1 | grep "cannot find" >/dev/null
    export SCTP_PRESENT=$?
    set -o errexit
    if [ ${SCTP_PRESENT} -eq 1 ]; then
        echo "SCTP library found"
    else
        echo "No SCTP library found, install libsctp ('sudo apt-get install libsctp-dev' on debian)"
    fi
}


examples=(
        test/framework-test
        test/delay-test
        test/chain-test
        test/lpm
        test/nat
        test/maglev
        test/tcp_check
        test/sctp-test
        test/config-test
        test/reset-parse
)

print_examples () {
    echo "The following examples are available:"
    for eg in ${examples[@]}; do
        if [ -e ${BASE_DIR}/${eg}/Cargo.toml ]; then
            target=$( ${CARGO} read-manifest --manifest-path ${BASE_DIR}/${eg}/Cargo.toml | ${BASE_DIR}/scripts/read-target.py - )
            printf "\t %s\n" ${target}
        fi
    done
    exit 0
}

clean () {
    pushd $BASE_DIR/framework
    ${CARGO} clean || true
    popd

    pushd $BASE_DIR/test/framework-test
    ${CARGO} clean || true
    popd

    for example in ${examples[@]}; do
        pushd ${BASE_DIR}/$example
        ${CARGO} clean || true
        popd
    done
    make clean -C ${BASE_DIR}/native
}

UNWIND_BUILD="${TOOLS_BASE}"/libunwind

deps () {
    # Build DPDK
    if [ ! -e $DPDK ]; then
        dpdk
    else
        echo "DPDK found not building"
    fi

    rust

    if [ ! -e $CARGO ]; then
        cargo
    else
        echo "Cargo found, not building"
    fi
}

clean_deps() {
    echo "Cleaning dependencies"
    rm -rf ${BIN_DIR} || true
    rm -rf ${DOWNLOAD_DIR} || true
    rm -rf ${TOOLS_BASE} || true
    rm -rf ${LLVM_RESULT} || true
    rm -rf ${MUSL_RESULT} || true
    rm -rf ${DPDK_HOME} || true
    echo "Cleaned DEPS"
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
    if [ ! -z ${RUST_STATIC} ]; then
        echo "Rust static is ${RUST_STATIC}, building with that"
        ./configure --prefix=${TOOLS_BASE} \
            --local-rust-root=${TOOLS_BASE}
    else
        ./configure --prefix=${TOOLS_BASE}
    fi
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

if [ $# -ge 1 ]; then
    TASK=$1
else
    TASK=build
fi

case $TASK in
    deps)
        deps
        ;;
    enable_symbols)
        export NETBRICKS_SYMBOLS=1
        toggle_symbols
        ;;
    disable_symbols)
        unset NETBRICKS_SYMBOLS || true
        toggle_symbols
        ;;
    sctp)
        find_sctp
        ;;
    build_test)
        shift
        if [ $# -lt 1 ]; then
            echo Can build one of the following tests:
            for example in ${examples[@]}; do
                base_eg=$( basename ${example} )
                printf "\t %s\n" ${base_eg}
            done
            exit 1
        fi
        build_dir=$1
        if [ ! -e ${BASE_DIR}/test/${build_dir}/Cargo.toml ]; then
            echo "No Cargo.toml, not valid"
        fi
        pushd ${BASE_DIR}/test/${build_dir}
            ${CARGO} build --release
        popd
        ;;
    build)
        deps

        make -j $proc -C $BASE_DIR/native

        find_sctp

        pushd $BASE_DIR/framework
        if [ ${SCTP_PRESENT} -eq 1 ]; then
            ${CARGO} build --release --features "sctp"
        else
            ${CARGO} build --release
        fi
        popd

        for example in ${examples[@]}; do
            if [[ ${example} == *sctp* ]]; then
                if [ ${SCTP_PRESENT} -eq 1 ]; then
                    pushd ${BASE_DIR}/${example}
                    ${CARGO} build --release
                    popd
                fi
            else
                pushd ${BASE_DIR}/${example}
                ${CARGO} build --release
                popd
            fi
        done
        ;;
    build_container)
        clean
        clean_deps
        sudo docker build -f container/dynamic/Dockerfile -t netbricks:latest ${BASE_DIR}
        echo "Done building container as netbricks:latest"
        ;;
    test)
        deps
        pushd $BASE_DIR/framework
        ${CARGO} test --release
        popd
        ;;
    run)
        shift
        if [ $# -le 1 ]; then
            print_examples
        fi
        cmd=$1
        shift
        executable=${BASE_DIR}/target/release/$cmd
        if [ ! -e ${executable} ]; then
            echo "${executable} not found, building"
            ${BASE_DIR}/${BUILD_SCRIPT} build
        fi
        export PATH="${BIN_DIR}:${PATH}"
        export LD_LIBRARY_PATH="${TOOLS_BASE}:${LD_LIBRARY_PATH}"
        sudo env PATH="$PATH" LD_LIBRARY_PATH="$LD_LIBRARY_PATH" LD_PRELOAD="$LD_PRELOAD" \
            ${BASE_DIR}/target/release/$cmd "$@"
        ;;
    debug)
        shift
        if [ $# -le 1 ]; then
            print_examples
        fi
        cmd=$1
        shift
        executable=${BASE_DIR}/target/release/$cmd
        if [ ! -e ${executable} ]; then
            echo "${executable} not found, building"
            ${BASE_DIR}/${BUILD_SCRIPT} build
        fi
        export PATH="${BIN_DIR}:${PATH}"
        export LD_LIBRARY_PATH="${TOOLS_BASE}:${LD_LIBRARY_PATH}"
        sudo env PATH="$PATH" LD_LIBRARY_PATH="$LD_LIBRARY_PATH" LD_PRELOAD="$LD_PRELOAD" \
            rust-gdb --args ${BASE_DIR}/target/release/$cmd "$@"
        ;;
    update_rust)
        _BUILD_UPDATE_=1
        rust
        cargo
        ;;
    fmt)
        deps
        pushd $BASE_DIR/framework
        ${CARGO} fmt
        popd

        for example in ${examples[@]}; do
            pushd ${BASE_DIR}/${example}
            ${CARGO} fmt
            popd
        done
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
        clean
        clean_deps
        ;;
    clean)
        clean
        ;;
    env)
        echo "export PATH=\"${BIN_DIR}:${PATH}\""
        echo "export LD_LIBRARY_PATH=\"${TOOLS_BASE}:${LD_LIBRARY_PATH}\""
        ;;
    *)
        echo "./build.sh <Command>
        Where command is one of
        deps: Build dependencies
        build: Build the project
        build_test: Build a particular test.
        doc: Run rustdoc and produce documentation
        fmt: Run rustfmt to format text prettily.
        lint: Run clippy to lint the project
        clean: Remove all built files
        dist_clean: Remove all support files
        env: Environment variables, run as eval `./build.sh env`.
        run: Run one of the examples.
        debug: Debug one of the examples.
        "
        ;;
esac
