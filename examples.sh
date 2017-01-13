#!/bin/bash
# Lists all the examples in Bess. This is used by the build script.
export examples=(
        test/framework-test
        test/delay-test
        test/shutdown-test
        test/chain-test
        test/lpm
        test/lpm-embedded
        test/nat
        test/maglev
        test/tcp_check
        test/sctp-test
        test/config-test
        test/reset-parse
        test/tcp_reconstruction
        test/acl-fw
        test/packet_generation
        test/packet_test
        test/embedded-scheduler-test
        test/embedded-scheduler-dependency-test
	test/bessmod
)

