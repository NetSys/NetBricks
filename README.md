This needs a nightly rust and a modified Cargo (available 
[here](https://github.com/apanda/cargo)). The Cargo modifications mostly enable 
the use of SIMD in Rust. These are now already included in the repository.

To build:

-   Install Rust nightly. Make sure the `RUST_HOME` environment variable points to some directory where Rust will be
    installed. Also ensure that `${RUST_HOME}/bin` is in your path.
    ```curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --disable-sudo --prefix=$RUST_HOME --verbose```
-   Run `./build.sh`. This will download and build DPDK, the framework and examples. 
-   To build documentation run `./build.sh doc`
