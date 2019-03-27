use common::Result;
use packets::ip::IpAddrMismatchError;
use std::net::IpAddr;
use std::slice;

/// Computes the internet checksum
/// https://tools.ietf.org/html/rfc1071
///
/// (1) Adjacent octets to be checksummed are paired to form 16-bit
///     integers, and the 1's complement sum of these 16-bit integers is
///     formed.
///
/// (2) To generate a checksum, the checksum field itself is cleared,
///     the 16-bit 1's complement sum is computed over the octets
///     concerned, and the 1's complement of this sum is placed in the
///     checksum field.
///
/// (3) To check a checksum, the 1's complement sum is computed over the
///     same set of octets, including the checksum field.  If the result
///     is all 1 bits (-0 in 1's complement arithmetic), the check
///     succeeds.
pub fn compute(pseudo_header_sum: u16, payload: &[u8]) -> u16 {
    let len = payload.len();
    let mut data = payload;
    let mut checksum = pseudo_header_sum as u32;

    // odd # of bytes, we add the last byte with padding separately
    if len % 2 > 0 {
        checksum += (payload[len - 1] as u32) << 8;
        data = &payload[..(len - 1)];
    }

    // a bit of unsafe magic to cast [u8] to [u16]
    let data = unsafe { slice::from_raw_parts(data.as_ptr() as *const u16, len / 2) };

    checksum = data.iter().fold(checksum, |acc, &x| acc + x as u32);

    while checksum >> 16 != 0 {
        checksum = (checksum >> 16) + (checksum & 0xFFFF);
    }

    return !(checksum as u16);
}

/// Computes the internet checksum via incremental update
/// https://tools.ietf.org/html/rfc1624
///
/// Given the following notation:
/// - `HC`  - old checksum in header
/// - `HC'` - new checksum in header
/// - `m`   - old value of a 16-bit field
/// - `m'`  - new value of a 16-bit field
///
/// `HC' = ~(~HC + ~m + m')`
pub fn compute_inc(old_checksum: u16, old_value: &[u16], new_value: &[u16]) -> u16 {
    let mut checksum = old_value
        .iter()
        .zip(new_value.iter())
        .fold(!old_checksum as u32, |acc, (&old, &new)| {
            acc + !old as u32 + new as u32
        });

    while checksum >> 16 != 0 {
        checksum = (checksum >> 16) + (checksum & 0xFFFF);
    }

    !(checksum as u16)
}

/// Incrementally computes the new checksum for an IP address change
pub fn compute_with_ipaddr(
    old_checksum: u16,
    old_value: &IpAddr,
    new_value: &IpAddr,
) -> Result<u16> {
    match (old_value, new_value) {
        (IpAddr::V4(old), IpAddr::V4(new)) => {
            // a bit of unsafe magic to cast [u8; 4] to [u16; 2]
            let old = unsafe { slice::from_raw_parts((&old.octets()).as_ptr() as *const u16, 2) };
            let new = unsafe { slice::from_raw_parts((&new.octets()).as_ptr() as *const u16, 2) };
            Ok(compute_inc(old_checksum, old, new))
        }
        (IpAddr::V6(old), IpAddr::V6(new)) => {
            Ok(compute_inc(old_checksum, &old.segments(), &new.segments()))
        }
        _ => Err(IpAddrMismatchError.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_checksum_incrementally() {
        assert_eq!(0x0000, compute_inc(0xdd2f, &[0x5555], &[0x3285]));
    }
}
