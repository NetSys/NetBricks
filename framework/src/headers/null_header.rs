use super::super::io;
use std::fmt;

#[derive(Debug)]
#[repr(C, packed)]
pub struct NullHeader;

impl fmt::Display for NullHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "null")
    }
}

impl io::EndOffset for NullHeader {
    #[inline]
    fn offset(&self) -> usize {
        0
    }
    #[inline]
    fn size() -> usize {
        0
    }
}

impl NullHeader {
    pub fn new() -> Self {
        NullHeader{}
    }
}
