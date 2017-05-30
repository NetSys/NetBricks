extern crate e2d2;
use e2d2::state::*;

#[test]
fn alloc_test() {
    let rb = RingBuffer::new(1).unwrap();
    drop(rb);
}

#[test]
fn write_at_offset_test() {
    let mut rb = RingBuffer::new(1).unwrap();
    let input = vec![1, 2, 3, 4];
    rb.write_at_offset(4095, &input[..]);
    let mut output: Vec<_> = (0..input.len()).map(|_| 0).collect();
    rb.read_from_offset(4095, &mut output[..]);
    for idx in 0..input.len() {
        assert_eq!(input[idx], output[idx]);
    }
}

#[test]
fn read_write_tail_test() {
    let mut rb = RingBuffer::new(1).unwrap();
    let input: Vec<_> = (0..8192).map(|i| (i & 0xff) as u8).collect();
    let written = rb.write_at_tail(&input[..]);
    assert_eq!(written, 4095);
    let mut output: Vec<_> = (0..8192).map(|_| 0).collect();
    let read = rb.read_from_head(&mut output[..]);
    assert_eq!(read, written);
    for idx in 0..read {
        assert_eq!(input[idx], output[idx]);
    }
}
