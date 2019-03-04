use byteorder::*;
use headers::mac::MacAddress;
use headers::EndOffset;
use num::FromPrimitive;
use std::collections::HashMap;
use std::fmt;
use std::net::Ipv6Addr;
use std::slice;

#[derive(FromPrimitive, Debug, PartialEq, Hash, Eq, Clone, Copy)]
#[repr(u8)]
pub enum NDPOptionType {
    SourceLinkLayerAddress = 1,
    TargetLinkLayerAddress = 2,
    PrefixInformation = 3,
    RedirectHeader = 4,
    MTU = 5,
}

impl fmt::Display for NDPOptionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NDPOptionType::SourceLinkLayerAddress => write!(f, "Source Link Layer Address"),
            NDPOptionType::TargetLinkLayerAddress => write!(f, "Target Link Layer Address"),
            NDPOptionType::PrefixInformation => write!(f, "Prefix Information"),
            NDPOptionType::RedirectHeader => write!(f, "Redirect Header"),
            NDPOptionType::MTU => write!(f, "MTU"),
        }
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct MtuOption {
    reserved: u16,
    mtu: u32,
}

impl MtuOption {
    pub fn get_mtu(&self) -> u32 {
        u32::from_be(self.mtu)
    }

    pub fn get_reserved(&self) -> u16 {
        u16::from_be(self.reserved)
    }
}

// This is a helper that allows us to parse options and do fancy pattern matching;
// cannot pattern match on a marker trait as they cannot down cast :(
// Lifetime marker 'a is necessary
// We must pass a _reference_ to the actual option because we don't want to copy
pub enum NdpOption<'a> {
    SourceLinkLayerOpt(&'a MacAddress),
    MtuOpt(&'a MtuOption)
}

// Searches through a packet for an option, returns the first it finds or none
pub fn find_ndp_option<'a, A : EndOffset + Sized>(ndp_message: &A, payload_length: u16, opt_type: NDPOptionType) -> Option<NdpOption<'a>> {
    unsafe {
        // the start of the options we offset from is the start of the ndp message
        let opt_start = (ndp_message as *const A) as *const u8;
        let mut payload_offset = ndp_message.offset();

        // make sure we do not scan beyond the end of the packet
        while (payload_offset as u16) < payload_length {
            println!("scan_opt offset={}, length={}, opt_type={}", payload_offset, payload_length, opt_type);

            // we want to pull the first two bytes off, option type and option length
            let seek_to = opt_start.offset(payload_offset as isize);
            let option_meta = slice::from_raw_parts(seek_to, 2);

            // first option meta is the option type, let's cast it to our NDPOptionType
            let cur_type: NDPOptionType = FromPrimitive::from_u8(option_meta[0]).unwrap();

            // second option is the length in octets (8 byte chunks).  So a length of 1 means 8 bytes
            let cur_size = option_meta[1] * 8;

            println!("scan_opt cur_type={}, cur_size={}", cur_type, cur_size);

            if cur_size == 0 {
                // Don't know how we will be here, but it is an error condition as we will
                // go into an infinite loop and blow the stack
                println!("WHY IS cur_size 0!!!");
                return None;
            } else if cur_type != opt_type {
                // the current type does not match the requested type, so skip to next
                payload_offset = payload_offset + (cur_size as usize);
                println!("Skipping to next offset {}", payload_offset);
            } else {
                // we have a winner!
                println!("Found option type {}", opt_type);

                // skip over the first two bytes as those are option type and length
                let cur_start = opt_start.offset((payload_offset + 2) as isize);

                // add more option types here...
                match opt_type {
                    NDPOptionType::MTU => {
                        let found_opt = cur_start as *const MtuOption;
                        return Some(NdpOption::MtuOpt(&(*found_opt)))
                    },
                    NDPOptionType::SourceLinkLayerAddress => {
                        let found_opt = cur_start as *const MacAddress;
                        return Some(NdpOption::SourceLinkLayerOpt(&(*found_opt)))
                    },
                    other => {
                        panic!("Unknown option type {}", other)
                    }
                }
            }
        }
        None
    }
}
