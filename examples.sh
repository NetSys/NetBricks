#!/bin/bash
# Lists all the examples that are runnable and/or testable.
# This is used by the build script.
export examples=(
        examples/echo-reply
        examples/ipv4or6
        examples/macswap
        examples/op-errors
        examples/mtu-too-big
        # examples/delay-test
        # examples/srv6-sighup-flow
        # examples/chain-test
        # examples/lpm
        # examples/lpm-embedded
        # examples/maglev
        # examples/sctp-test
        # examples/tcp-reconstruction
        # examples/acl-fw
        ### Runnable examples | No Tests associated
        ### =======================================
        examples/config-test
        examples/shutdown-test
        examples/embedded-scheduler-test
        examples/embedded-scheduler-dependency-test
        examples/nat-tcp-v4
)
