use failure::{Error, Fail};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum NetBricksError {
    #[fail(display = "Failed to allocate memory")]
    FailedAllocation,

    #[fail(display = "Failed to free memory buffer (mbuf) {}", _0)]
    FailedToFreeMBuf(i32),

    #[fail(display = "Failed to remove/drop packets from buffer")]
    FailedToDropPackets,

    #[fail(display = "Failed to deallocate memory")]
    FailedDeallocation,

    #[fail(display = "Failed to initialize port: {}", _0)]
    FailedToInitializePort(i32),

    #[fail(display = "Invalid queue request")]
    BadQueue,

    #[fail(display = "Cannot send data out port")]
    CannotSend,

    #[fail(display = "Cannot find device: {}", _0)]
    BadDev(String),

    #[fail(display = "Bad vdev specification: {}", _0)]
    BadVdev(String),

    #[fail(display = "Bad TX queue {} for port {}", _0, _1)]
    BadTxQueue(i32, i32),

    #[fail(display = "Bad RX queue {} for port {}", _0, _1)]
    BadRxQueue(i32, i32),

    #[fail(display = "Attempt to access bad packet offset {}", _0)]
    BadOffset(usize),

    #[fail(display = "Metadata is too large")]
    MetadataTooLarge,

    #[fail(display = "Could not allocate ring")]
    RingAllocationFailure,

    #[fail(display = "Address of second copy of ring does not match expected address")]
    RingDuplicationFailure,

    #[fail(display = "Bad ring size {}, must be a power of 2", _0)]
    InvalidRingSize(usize),

    #[fail(display = "Configuration error: {}", _0)]
    ConfigurationError(String),

    #[fail(display = "No scheduler running on core {}", _0)]
    NoRunningSchedulerOnCore(i32),

    #[fail(display = "Failed to insert header into packet")]
    FailedToInsertHeader,

    #[fail(
        display = "Failed to swap-in new header - {} - in packet, new_header",
        _0
    )]
    FailedToSwapHeader(String),

    #[fail(display = "Failed to remove header from packet")]
    FailedToRemoveHeader,

    #[fail(display = "Failed to parse MAC address: '{}'", _0)]
    FailedToParseMacAddress(String),

    #[fail(display = "_")]
    #[doc(hidden)]
    __Nonexhaustive,
}

#[macro_export]
macro_rules! error_chain {
    ($error:expr) => {
        error!("{}", $crate::common::errors::string_chain($error))
    };
}

#[macro_export]
macro_rules! warn_chain {
    ($error:expr) => {
        warn!("{}", $crate::common::errors::string_chain($error))
    };
}

/// Read a `failure` `Error` and print out the causes and a backtrace as
/// `log::error`s
pub fn string_chain(e: &Error) -> String {
    let mut error = e.to_string();

    for cause in e.iter_causes() {
        error.push_str(&format!("\nCaused by: {}", cause));
    }

    if let Ok("1") = ::std::env::var("RUST_BACKTRACE")
        .as_ref()
        .map(|s| s.as_str())
    {
        error.push_str(&format!("\nBacktrace:\n{}", e.backtrace()))
    }

    error
}
