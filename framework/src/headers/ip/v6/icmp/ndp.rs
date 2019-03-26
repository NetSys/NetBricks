use headers::EndOffset;

/// Marker trait for all types of NdpMessages
/// Ensures all implementers support EndOffset and Sized
pub trait NdpMessageContents: EndOffset + Sized {}
