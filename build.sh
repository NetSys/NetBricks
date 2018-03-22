#!/bin/bash
# Stop on any errors
set -e
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
BUILD_SCRIPT=$( basename "$0" )

CARGO_LOC=`which cargo || true`
export CARGO=${CARGO_PATH-"${CARGO_LOC}"}

DPDK_VER=17.08
DPDK_HOME="/opt/dpdk/dpdk-stable-${DPDK_VER}"
DPDK_LD_PATH="${DPDK_HOME}/build/lib"
DPDK_CONFIG_FILE=${DPDK_CONFIG_FILE-"${DPDK_HOME}/config/common_linuxapp"}

NATIVE_LIB_PATH="${BASE_DIR}/native"
export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

source ${BASE_DIR}/examples.sh

if [[ "$OSTYPE" == "darwin"* ]]; then
    proc=`sysctl -n hw.physicalcpu`
else
    proc=`nproc`
fi

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

if [ $# -ge 1 ]; then
    TASK=$1
else
    TASK=build
fi

case $TASK in
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
            echo "Can build one of the following tests:"
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
        ${CARGO} build
        popd
        ;;
    build_test_rel)
        shift
        if [ $# -lt 1 ]; then
            echo "Can build a release for one of the following tests:"
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
        find_sctp
        native

        pushd $BASE_DIR/framework
        ${CARGO} build
        popd
        ;;
    build)
        find_sctp
        native

        pushd $BASE_DIR/framework
        ${CARGO} build
        popd

        for example in ${examples[@]}; do
            pushd ${BASE_DIR}/${example}
            ${CARGO} build
            popd
        done
        ;;
    build_rel)
        find_sctp
        native

        pushd $BASE_DIR/framework
        ${CARGO} build --release
        popd

        for example in ${examples[@]}; do
            pushd ${BASE_DIR}/${example}
            ${CARGO} build --release
            popd
        done
        ;;
    test)
        pushd $BASE_DIR/framework
        export LD_LIBRARY_PATH="${NATIVE_LIB_PATH}:${DPDK_LD_PATH}:${LD_LIBRARY_PATH}"
        RUST_BACKTRACE=1 ${CARGO} test
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
        executable=${BASE_DIR}/target/debug/$cmd
        if [ ! -e ${executable} ]; then
            echo "${executable} not found, building"
            ${BASE_DIR}/${BUILD_SCRIPT} build
        fi
        export PATH="${BIN_DIR}:${PATH}"
        export LD_LIBRARY_PATH="${NATIVE_LIB_PATH}:${DPDK_LD_PATH}:${LD_LIBRARY_PATH}"
        sudo env PATH="$PATH" LD_LIBRARY_PATH="$LD_LIBRARY_PATH" LD_PRELOAD="$LD_PRELOAD" \
            $executable "$@"
        ;;
    run_rel)
        shift
        if [ $# -le 0 ]; then
            print_examples
        fi
        cmd=$1
        shift
        executable=${BASE_DIR}/target/release/$cmd
        if [ ! -e ${executable} ]; then
            echo "${executable} not found, building"
            ${BASE_DIR}/${BUILD_SCRIPT} build_rel
        fi
        export PATH="${BIN_DIR}:${PATH}"
        export LD_LIBRARY_PATH="${NATIVE_LIB_PATH}:${DPDK_LD_PATH}:${LD_LIBRARY_PATH}"
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
        executable=${BASE_DIR}/target/debug/$cmd
        if [ ! -e ${executable} ]; then
            echo "${executable} not found, building"
            ${BASE_DIR}/${BUILD_SCRIPT} build
        fi
        export PATH="${BIN_DIR}:${PATH}"
        export LD_LIBRARY_PATH="${NATIVE_LIB_PATH}:${DPDK_LD_PATH}:${LD_LIBRARY_PATH}"
        sudo env PATH="$PATH" LD_LIBRARY_PATH="$LD_LIBRARY_PATH" LD_PRELOAD="$LD_PRELOAD" \
            rust-gdb --args $executable "$@"
        ;;
    check_manifest)
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
        pushd $BASE_DIR/framework
        ${CARGO} rustdoc -- \
            --no-defaults --passes "collapse-docs" --passes \
                "unindent-comments"
        popd
        ;;
    lint)
        pushd $BASE_DIR/framework
        ${CARGO} clean
        ${CARGO} update # Clippy breaks with new compilers
        ${CARGO} build --features dev
        popd
        ;;
    clean)
        clean
        ;;
    fmt)
        pushd $BASE_DIR/framework
        cargo fmt
        popd

        for example in ${examples[@]}; do
            pushd ${BASE_DIR}/${example}
            cargo fmt
            popd
        done
        ;;
    env)
        echo "export PATH=\"${BIN_DIR}:${PATH}\""
        echo "export LD_LIBRARY_PATH=\"${NATIVE_LIB_PATH}:${TOOLS_BASE}:${LD_LIBRARY_PATH}\""
        ;;
    *)
        cat <<endhelp
./build.sh <Command>
      Where command is one of
          sctp: Check if sctp library is present.
          build: Build the project (this includes framework and all tests).
          build_rel: Build a release of the project (this includes framework and all tests).
          build_fmwk: Just build framework.
          build_test: Build a particular test.
          build_test_rel: Build a particular test in release mode.
          test: Run unit tests.
          run: Run one of the examples (Must specify example name and arguments).
          run_rel: Run one of the examples in release mode (Must specify example name and arguments).
          debug: Debug one of the examples (Must specify example name and examples).
          doc: Run rustdoc and produce documentation
          lint: Run clippy to lint the project
          clean: Remove all built files
          env: Environment variables, run as eval \`./build.sh env\`.
endhelp
        ;;
esac
