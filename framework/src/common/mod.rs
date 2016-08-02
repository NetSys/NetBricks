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
}

pub type Result<T> = result::Result<T, ZCSIError>;
