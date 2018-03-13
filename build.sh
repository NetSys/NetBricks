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
DPDK_VER=17.08
DPDK_HOME="${BASE_DIR}/3rdparty/dpdk"
DPDK_LD_PATH="${DPDK_HOME}/build/lib"
DPDK_CONFIG_FILE=${DPDK_CONFIG_FILE-"${EXT_BASE}/dpdk-confs/common_linuxapp-${DPDK_VER}"}
if grep "CONFIG_RTE_BUILD_SHARED_LIB=y" ${DPDK_CONFIG_FILE}; then
    DPDK="${DPDK_HOME}/build/lib/libdpdk.so"
else
    DPDK="${DPDK_HOME}/build/lib/libdpdk.a"
fi

CARGO_LOC=`which cargo || true`
export CARGO=${CARGO_PATH-"${CARGO_LOC}"}
#if [ -z ${CARGO} ] || [ ! -e ${CARGO} ]; then
    #echo "Could not find a preinstalled Cargo in PATH. Set CARGO_PATH if necessary."
    #exit 1
#fi
#echo "Using Cargo from ${CARGO}"

MUSL_DOWNLOAD_PATH="${DOWNLOAD_DIR}/musl.tar.gz"
MUSL_RESULT="${EXT_BASE}/musl"
MUSL_TEST="${TOOLS_BASE}/lib/libc.a"

RUST_TEST="${TOOLS_BASE}/bin/rustc"
RUST_DOWNLOAD_PATH="${EXT_BASE}/rust"

LLVM_DOWNLOAD_PATH="${DOWNLOAD_DIR}/llvm.tar.gz"
LLVM_RESULT="${EXT_BASE}/llvm"
UNWIND_RESULT="${TOOLS_BASE}/lib/libunwind.a"

NATIVE_LIB_PATH="${BASE_DIR}/native"
export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

source ${BASE_DIR}/examples.sh
REQUIRE_RUSTFMT=0
export RUSTFLAGS="-C target-cpu=native"

if [[ "$OSTYPE" == "darwin"* ]]; then
    proc=`sysctl -n hw.physicalcpu`
else
    proc=`nproc`
fi

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

native () {
    make -j $proc -C $BASE_DIR/native
    make -C $BASE_DIR/native install
}


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
    rm -rf ${BASE_DIR}/target
}

UNWIND_BUILD="${TOOLS_BASE}"/libunwind

deps () {
    # Build DPDK
    export DPDK_CONFIG_FILE=${DPDK_CONFIG_FILE}
    export DPDK_VER=${DPDK_VER}
    if [ ! -e $DPDK ]; then
        dpdk
    else
        echo "DPDK found not building"
    fi

    rust

    if [ ${REQUIRE_RUSTFMT} -ne 0 ]; then
        rust_fmt
    fi
    echo "Done with deps"
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
    curl -L http://llvm.org/releases/3.7.0/llvm-3.7.0.src.tar.xz \
        -o "${LLVM_DOWNLOAD_PATH}"
    mkdir -p ${LLVM_RESULT}
    tar xf "${LLVM_DOWNLOAD_PATH}" \
        -C "${LLVM_RESULT}" --strip-components=1
    UNWIND_DOWNLOAD="${DOWNLOAD_DIR}"/unwind.tar.gz
    UNWIND_DIR="${LLVM_RESULT}/projects/libunwind"
    mkdir -p ${UNWIND_DIR}
    curl -L http://llvm.org/releases/3.7.0/libunwind-3.7.0.src.tar.xz \
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

rust_fmt () {
    RUSTFMT=${BIN_DIR}/cargo-fmt
    echo "Checking if ${RUSTFMT} exists (${REQUIRE_RUSTFMT})"
    if [ ! -e "${RUSTFMT}" ]; then
        ${CARGO} install --root ${TOOLS_BASE} rustfmt-nightly
        export RUSTFMT=${RUSTFMT}
    else
        export RUSTFMT=${RUSTFMT}
    fi
}

if [ $# -ge 1 ]; then
    TASK=$1
else
    TASK=build
fi

case $TASK in
    deps)
        REQUIRE_RUSTFMT=1
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
    build_fmwk)
        deps
        native
        find_sctp
        pushd $BASE_DIR/framework
        if [ ${SCTP_PRESENT} -eq 1 ]; then
            ${CARGO} build --release --features "sctp"
        else
            ${CARGO} build --release
        fi
        popd
        ;;
    build)
        deps

        native

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
    create_container)
        clean
        clean_deps
        docker build -f container/Dockerfile -t netbricks:vswitch --build-arg dpdk_file="common_linuxapp-${DPDK_VER}.container" ${BASE_DIR}
        echo "Done building container as netbricks:vswitch"
        ;;
    ctr_dpdk)
        shift
        if [ $# -lt 1 ]; then
            echo "build.sh ctr_dpdk dir"
            exit 1
        fi
        result="$( readlink -f $1 )"
        ctr="$( docker create netbricks:vswitch )"
        docker cp ${ctr}:/opt/netbricks/3rdparty/dpdk $result
        docker rm ${ctr}
        ;;
    _build_container)
        curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
        export DPDK_CONFIG_FILE="${EXT_BASE}/dpdk-confs/common_linuxapp-${DPDK_VER}.container"
        PATH="$HOME/.cargo/bin:$PATH" ${BASE_DIR}/build.sh build
        ;;
    build_container)
        docker pull apanda/netbricks-build:latest
        docker run -t -v /lib/modules:/lib/modules \
            -v ${BASE_DIR}:/opt/netbricks \
            apanda/netbricks-build:latest /opt/netbricks/build.sh _build_container
        ;;
    update_container)
        docker build --no-cache -f ${BASE_DIR}/build-container/Dockerfile -t apanda/netbricks-build:latest \
            ${BASE_DIR}/build-container
        docker push apanda/netbricks-build:latest
        ;;
    ctr_test)
        docker pull apanda/netbricks-build:latest
        docker run -t -v /lib/modules:/lib/modules \
            -v /lib/modules/`uname -r`/build:/lib/modules/`uname -r`/build -v ${BASE_DIR}:/opt/netbricks \
            -v /mnt/huge:/mnt/huge apanda/netbricks-build:latest /opt/netbricks/build.sh test
        ;;
    test)
        pushd $BASE_DIR/framework
        export LD_LIBRARY_PATH="${NATIVE_LIB_PATH}:${DPDK_LD_PATH}:${TOOLS_BASE}:${LD_LIBRARY_PATH}"
        RUST_BACKTRACE=1 ${CARGO} test --release
        popd

        for testname in tcp_payload macswap; do
          pushd $BASE_DIR/test/$testname
          ./check.sh
          popd
        done
        ;;
    run)
        shift
        if [ $# -le 0 ]; then
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
        export LD_LIBRARY_PATH="${NATIVE_LIB_PATH}:${DPDK_LD_PATH}:${TOOLS_BASE}:${LD_LIBRARY_PATH}"
        sudo env PATH="$PATH" LD_LIBRARY_PATH="$LD_LIBRARY_PATH" LD_PRELOAD="$LD_PRELOAD" \
            $executable "$@"
        ;;
    debug)
        shift
        if [ $# -le 0 ]; then
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
        export LD_LIBRARY_PATH="${NATIVE_LIB_PATH}:${DPDK_LD_PATH}:${TOOLS_BASE}:${LD_LIBRARY_PATH}"
        sudo env PATH="$PATH" LD_LIBRARY_PATH="$LD_LIBRARY_PATH" LD_PRELOAD="$LD_PRELOAD" \
            rust-gdb --args $executable "$@"
        ;;
    update_rust)
        _BUILD_UPDATE_=1
        rust
        cargo_clone
        cargo_build
        ;;
    fmt)
        REQUIRE_RUSTFMT=1
        deps
        pushd $BASE_DIR/framework
        ${RUSTFMT} fmt -- --config-path ${BASE_DIR}/.travis || true
        popd

        for example in ${examples[@]}; do
            pushd ${BASE_DIR}/${example}
            ${RUSTFMT} fmt -- --config-path ${BASE_DIR}/.travis || true
            popd
        done
        ;;
    _fmt_travis)
        echo "Running _fmt_travis"
        pushd $BASE_DIR/framework
        ${CARGO} fmt -- --config-path ${BASE_DIR}/.travis --write-mode=diff
        popd
        for example in ${examples[@]}; do
            pushd ${BASE_DIR}/${example}
            ${CARGO} fmt -- --config-path ${BASE_DIR}/.travis --write-mode=diff
            popd
        done
        ;;
    fmt_travis)
        docker pull apanda/netbricks-build:latest
        docker run -t  -v /lib/modules:/lib/modules \
            -v /lib/modules/`uname -r`/build:/lib/modules/`uname -r`/build -v ${BASE_DIR}:/opt/netbricks \
             apanda/netbricks-build:latest /opt/netbricks/build.sh _fmt_travis
        ;;
    check_manifest)
        deps
        pushd ${BASE_DIR}
        ${CARGO} verify-project --verbose
        popd

        pushd ${BASE_DIR}/framework
        ${CARGO} verify-project | grep true
        popd

        for example in ${examples[@]}; do
            pushd ${BASE_DIR}/${example}
            ${CARGO} verify-project | grep true
            popd
        done
        ;;
    check_examples)
        python3 scripts/check-examples.py "${examples[@]}"
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
        ${CARGO} update # Clippy breaks with new compilers
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
        echo "export LD_LIBRARY_PATH=\"${NATIVE_LIB_PATH}:${TOOLS_BASE}:${LD_LIBRARY_PATH}\""
        ;;
    *)
        cat <<endhelp
./build.sh <Command>
      Where command is one of
          deps: Build dependencies
          sctp: Check if sctp library is present.
          build: Build the project (this includes framework and all tests).
          build_fmwk: Just build framework.
          build_test: Build a particular test.
          create_container: Build the NetBricks container.
          ctr_dpdk: Copy DPDK from container
          build_container: Build NetBricks within a container.
          test: Run unit tests.
          run: Run one of the examples (Must specify example name and arguments).
          debug: Debug one of the examples (Must specify example name and examples).
          doc: Run rustdoc and produce documentation
          update_rust: Pull and update Cargo.
          update_container: Update and push container used for build.
          fmt: Run rustfmt to format code.
          fmt_travis: Run rustfmt to detect code formatting violations.
          lint: Run clippy to lint the project
          clean: Remove all built files
          dist_clean: Remove all support files
          env: Environment variables, run as eval \`./build.sh env\`.
endhelp
        ;;
esac
