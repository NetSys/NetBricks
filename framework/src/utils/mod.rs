pub use self::asm::*;
pub use self::flow::*;
mod asm;
mod flow;

pub const PAGE_SIZE: usize = 4096; // Page size in bytes, not using huge pages here.

/// Round a given buffer to page size units.
#[inline]
pub fn round_to_pages(buffer_size: usize) -> usize {
    (buffer_size + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1)
}

/// Round a 64-bit integer to its nearest power of 2.
#[inline]
pub fn round_to_power_of_2(mut size: usize) -> usize {
    size = size.wrapping_sub(1);
    size |= size >> 1;
    size |= size >> 2;
    size |= size >> 4;
    size |= size >> 8;
    size |= size >> 16;
    size |= size >> 32;
    size = size.wrapping_add(1);
    size
}

// clears a bit in a byte, pos is the zero-based position in the byte starting from the left
#[inline]
pub fn clear_bit(original: u8, pos: u8) -> u8 {
    let swap_pos = 7 - pos;
    let mask = 1 << swap_pos;
    original & !mask
}

// sets a bit in a byte, pos is the zero-based position in the byte starting from the left
#[inline]
pub fn set_bit(original: u8, pos: u8) -> u8 {
    let swap_pos = 7 - pos;
    let mask = 1 << swap_pos;
    original | mask
}

// gets a bit value as a bool in a byte, pos is the zero-based position in the byte starting from left
#[inline]
pub fn get_bit(original: u8, pos: u8) -> bool {
    let swap_pos = 7 - pos;
    let mask = 1 << swap_pos;
    (original & mask) != 0
}

/// Flips a bit in a byte to on or off
///
/// # Arguments
///
/// * `original` - the byte to flip a bit on
/// * `pos` - the zero-based position of the bit in the byte to flip
/// * `on` - a boolean indicating whether the bit should be set (true) or cleared (false)
#[inline]
pub fn flip_bit(original: u8, pos: u8, on: bool) -> u8 {
    if on {
        set_bit(original, pos)
    } else {
        clear_bit(original, pos)
    }
}

// test flipping a bit at a known position
#[test]
fn flippin_bits() {
    let original: u8 = 0b01101000;

    // we will clear the 3rd bit (2nd position)
    let cleared: u8 = 0b01001000;

    // lets turn off the bit, and check that it is cleared
    let mut result = flip_bit(original, 2, false);
    assert_eq!(result, cleared);
    assert_eq!(get_bit(result, 2), false);

    // turn the bit back on, make sure it is set
    result = flip_bit(result, 2, true);
    assert_eq!(result, original);
    assert_eq!(get_bit(result, 2), true);
}
