This needs a nightly rust and a modified Cargo (available 
[here](https://github.com/apanda/cargo)). The Cargo modifications mostly enable 
the use of SIMD in Rust. These are now already included in the repository.

To build:

-   If building on a Debian machine, you need to undo some of the craziness wrought by Debian maintainers. In particular
    libcurl by default will not correctly allow Cargo to check server identity. To solve this install the `libgnutls30 libgnutls-openssl-dev libcurl4-gnutls-dev`
-   Install Rust nightly. Make sure the `RUST_HOME` environment variable points to some directory where Rust will be
    installed. Also ensure that `${RUST_HOME}/bin` is in your path.
    ```curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --disable-sudo --prefix=$RUST_HOME --verbose```
-   Run `./build.sh`. This will download and build DPDK, the framework and examples. 
-   To build documentation run `./build.sh doc`

To run:

-   You first need to use dpdk_nic_bind.py to associate NICs with DPDK drivers.
    For example on my machines I use
    ```
    sudo modprobe uio_pci_generic
    sudo ~/e2d2/3rdparty/dpdk/tools/dpdk_nic_bind.py -b uio_pci_generic 07:00.0 07:00.1 07:00.2 07:00.3
    ```
    See the [DPDK documentation](http://dpdk.readthedocs.org/en/v2.2.0/linux_gsg/build_dpdk.html) for more information
    about this.
-   Once done, you can run the test program by running
    ```
    sudo env LD_LIBRARY_PATH=$LD_LIBRARY_PATH $ZCSI_HOME/test/framework-test/target/release/zcsi-test -m 0 -c 6 -w "07:00.0" -c 6 -w "07:00.1" -c 7 -w "07:00.2" -c 7 -w "07:00.3"
    ```
    We need to redeclare the env because by default `sudo` resets environment variables. You can edit the sudoers file
    to prevent this, but the current method also works. The `-m` parameter indicates the master core that ZCSI should
    use, while each `-c, -w` pair indicate that ZCSI should associate the given NIC with the given core. The test
    program currently only initializes one queue per core, but this is expected to change.
