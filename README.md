NetBricks is a Rust based framework for NFV development. Please refer to the paper (available soon) for information
about the architecture and design. Currently NetBricks requires a relatively modern Linux version.

Building
--------
NetBricks can be built either using a Rust nightly build or using Rust built from the current Git head. In the later
case we also build [`musl`](https://www.musl-libc.org/) and statically link to things. Below we provide basic
instructions for both.

Using Rust Nightly
------------------
First obtain Rust nightly. I use [rustup](https://rustup.rs), in which case the following works

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

Dependencies
------------
Building NetBricks requires `libcurl` with support for `gnutls`. On Debian these dependencies can be installed using:

```
apt-get install libgnutls30 libgnutls-openssl-dev libcurl4-gnutls-dev
```

NetBricks also supports using SCTP as a control protocol. SCTP support requires the use of `libsctp` (this is an
optional dependency) which can be installed on Debian using:

```
apt-get install libsctp-dev
```

Example NFs
-----------
Coming Soon.

Future Work
-----------
Support for [`futures`](https://github.com/alexcrichton/futures-rs) for control plane functionality.
