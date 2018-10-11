#!/bin/bash
# Lists all the examples in Bess. This is used by the build script.
export examples=(
        test/framework-test
        test/packet-test
        test/tcp-payload
        test/macswap
        test/ipv4or6
        test/srv6-compose
        test/srv6-sighup-flow
        test/srv6-inject
        test/tcp-checksum/
        test/icmpv6
        # test/delay-test
        # test/shutdown-test
        # test/chain-test
        # test/lpm
        # test/lpm-embedded
        # test/nat
        # test/maglev
        # test/tcp-check
        # test/sctp-test
        # test/config-test
        # test/reset-parse
        # test/tcp-reconstruction
        # test/acl-fw
        # test/packet-generation
        # test/embedded-scheduler-test
        # test/embedded-scheduler-dependency-test
)

