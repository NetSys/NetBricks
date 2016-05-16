The build is entirely self contained, needing nothing.

To build
--------

- Run `./build.sh deps`. This will take a while initially as it downloads and
  builds all dependencies.

- Run `./build.sh` to build the project as a whole.

To run
------

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

Current usage
-------------

Currently the running program takes all received packets, exchanges the source
and destination MAC address (since we assume we are receiving and sending
packets back to the same packet generator), performs some other transformations
(this has been changing as I find more things to do) and sends packets out.
