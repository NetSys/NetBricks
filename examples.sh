#!/bin/bash
# Lists all the examples that are runnable and/or testable.
# This is used by the build script.
export examples=(
        examples/echo-reply
        examples/ipv4or6
        examples/macswap
        examples/op-errors
        examples/mtu-too-big
        # examples/srv6-sighup-flow
        # examples/chain-test
        # examples/maglev
        # examples/sctp-test
        # examples/tcp-reconstruction
        # examples/acl-fw
        ### Runnable examples | No Tests associated
        ### =======================================
        examples/config
        examples/shutdown
        examples/embedded-scheduler
        examples/embedded-scheduler-dependency
        examples/nat-tcp-v4
        examples/lpm
)
