error_chain! {
    errors {
        FailedAllocation {
            description("Failed to allocate memory")
            display("Failed to allocate memory")
        }
        FailedDeallocation {
            description("Failed to deallocate memory")
            display("Failed to deallocate memory")
        }
        FailedToInitializePort(port: i32) {
            description("Failed to initialize port")
            display("Failed to initialize port: {}", port)
        }
        BadQueue {
            description("Invalid queue request")
            display("Invalid queue request")
        }
        CannotSend {
            description("Cannot send data out port")
            display("Cannot send data out port")
        }
        BadDev(dev: String) {
            description("Cannot find device")
            display("Cannot find device: {}", dev)
        }
        BadVdev(vdev: String) {
            description("Bad vdev specification")
            display("Bad vdev specification: {}", vdev)
        }
        BadTxQueue(port: i32, queue: i32) {
            description("Bad TX queue")
            display("Bad TX queue {} for port {}", queue, port)
        }
        BadRxQueue(port: i32, queue: i32) {
            description("Bad RX queue")
            display("Bad RX queue {} for port {}", queue, port)
        }
        BadOffset(offset: usize) {
            description("Attempt to access bad packet offset")
            display("Attempt to access bad packet offset {}", offset)
        }

        MetadataTooLarge {
            description("Metadata is too large")
            display("Metadata is too large")
        }

        RingAllocationFailure {
            description("Could not allocate ring")
            display("Could not allocate ring")
        }

        InvalidRingSize(size: usize) {
            description("Bad ring size, must be power of 2")
            display("Bad ring size {}, must be a power of 2", size)
        }

        RingDuplicationFailure {
            description("Address of second copy of ring does not match expected address")
            display("Address of second copy of ring does not match expected address")
        }

        ConfigurationError(description: String) {
            description("Configuration error")
            display("Configuration error: {}", description)
        }

        NoRunningSchedulerOnCore(core: i32) {
            description("No scheduler running on core")
            display("No scheduler running on core {}", core)
        }

        FailedToInsertHeader {
            description("Failed to insert header into packet")
            display("Failed to insert header into packet")
        }

        FailedToSwapHeader(new_header: String) {
            description("Failed to swap-in new header in packet")
                display("Failed to swap-in new header - {} - in packet", new_header)
        }
    }

    foreign_links {
        Io(::std::io::Error);
    }
}
