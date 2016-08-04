NetBricks is a Rust based framework for NFV development. Please refer to the paper (available soon) for information
about the architecture and design. Currently NetBricks requires a relatively modern Linux version.

Building
--------
NetBricks can be built either using a Rust nightly build or using Rust built from the current Git head. In the later
case we also build [`musl`](https://www.musl-libc.org/) and statically link to things. Below we provide basic
instructions for both.

Using Rust Nightly
------------------
First obtain Rust nightly. I use [rustup](rustup.rs), in which case the following works

```
curl https://sh.rustup.rs -sSf | sh  # Install rustup
rustup install nightly
```

Then clone this repository and run `build.sh`

```
./build.sh
```

This should download DPDK, and build all of NetBricks.

Using Rust from Git
-------------------
The instructions for doing so are simple, however building takes significantly longer in this case (and consumes tons of
memory), so do this only if you have lots of time and memory. Building is as simple as

```
export RUST_STATIC=1
./build.sh
```

Example NFs
-----------
Coming Soon.
