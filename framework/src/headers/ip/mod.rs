pub use self::v4::*;
pub use self::v6::*;
use headers::EndOffset;
use std::default::Default;

mod v4;
mod v6;

// Trait for all IP headers that contain L4 protocols like TCP and UDP, allowing
// the L4 headers to be generic.
pub trait IpHeader: EndOffset + Default {}
