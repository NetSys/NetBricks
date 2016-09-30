use std::result;
#[derive(Debug)]
pub enum ZCSIError {
    FailedAllocation,
    FailedDeallocation,
    FailedToInitializePort,
    BadQueue,
    CannotSend,
    BadDev,
    BadVdev,
    BadTxQueue,
    BadRxQueue,
    BadOffset,
    MetadataTooLarge,
}

pub type Result<T> = result::Result<T, ZCSIError>;

/// Null metadata associated with packets initially.
pub struct EmptyMetadata;
