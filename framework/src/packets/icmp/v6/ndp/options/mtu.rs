#![allow(clippy::mut_from_ref)]

use super::MTU;
use common::Result;
use native::mbuf::MBuf;
use packets::{buffer, Fixed, ParseError};
use std::fmt;

/*  From https://tools.ietf.org/html/rfc4861#section-4.6.4
    MTU

     0                   1                   2                   3
     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |     Type      |    Length     |           Reserved            |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                              MTU                              |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

    Type            5

    Length          1

    Reserved        This field is unused.  It MUST be initialized to
                    zero by the sender and MUST be ignored by the
                    receiver.

    MTU             32-bit unsigned integer.  The recommended MTU for
                    the link.
*/

#[derive(Debug)]
#[repr(C, packed)]
struct MtuFields {
    option_type: u8,
    length: u8,
    reserved: u16,
    mtu: u32,
}

impl Default for MtuFields {
    fn default() -> MtuFields {
        MtuFields {
            option_type: MTU,
            length: 1,
            reserved: 0,
            mtu: 0,
        }
    }
}

/// Maximum transmission unit option
pub struct Mtu {
    fields: *mut MtuFields,
    offset: usize,
}

impl Mtu {
    /// Parses the MTU option from the message buffer at offset
    #[inline]
    pub fn parse(mbuf: *mut MBuf, offset: usize) -> Result<Mtu> {
        let fields = buffer::read_item::<MtuFields>(mbuf, offset)?;
        if unsafe { (*fields).length } != (MtuFields::size() as u8 / 8) {
            Err(ParseError::new("Invalid MTU option length").into())
        } else {
            Ok(Mtu { fields, offset })
        }
    }

    /// Returns the message buffer offset for this option
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    fn fields(&self) -> &mut MtuFields {
        unsafe { &mut (*self.fields) }
    }

    #[inline]
    pub fn option_type(&self) -> u8 {
        self.fields().option_type
    }

    pub fn length(&self) -> u8 {
        self.fields().length
    }

    pub fn mtu(&self) -> u32 {
        u32::from_be(self.fields().mtu)
    }

    pub fn set_mtu(&mut self, mtu: u32) {
        self.fields().mtu = u32::to_be(mtu);
    }
}

impl fmt::Display for Mtu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {}, length: {}, mtu: {}",
            self.option_type(),
            self.length(),
            self.mtu()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_mtu() {
        assert_eq!(8, MtuFields::size());
    }
}
