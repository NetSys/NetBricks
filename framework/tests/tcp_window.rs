extern crate e2d2;
use e2d2::state::*;
use e2d2::utils::*;
use std::str;
use std::u32;

/// Test rounding up to number of pages.
#[test]
fn round_pages_test() {
    assert_eq!(round_to_pages(1),
               4096,
               "Rounding up 1 byte did not result in PAGE_SIZE");
    assert_eq!(round_to_pages(0),
               0,
               "Rounding up 0 bytes did not result in 0");
    assert_eq!(round_to_pages(8),
               4096,
               "Rounding up failure, expected 4096, got {}",
               round_to_pages(8));
    assert_eq!(round_to_pages(512),
               4096,
               "Rounding up failure, expected 4096, got {}",
               round_to_pages(512));
    assert_eq!(round_to_pages(4096),
               4096,
               "Rounding up exactly 1 page failed, expected 4096, got {}",
               round_to_pages(4096));
    assert_eq!(round_to_pages(4097),
               8192,
               "Rounding up > 1 page failed, expected 8192, got {}",
               round_to_pages(4097));
}

/// Test rounding up to power of 2.
#[test]
fn round_to_power_of_2_test() {
    assert_eq!(round_to_power_of_2(0),
               0,
               "Rounding to power of 2 failed, expected 0");
    assert_eq!(round_to_power_of_2(1),
               1,
               "Rounding to power of 2 failed, expected 1");
    assert_eq!(round_to_power_of_2(2),
               2,
               "Rounding to power of 2 failed, expected 2");
    assert_eq!(round_to_power_of_2(3),
               4,
               "Rounding to power of 2 failed, expected 4");
    assert_eq!(round_to_power_of_2(4),
               4,
               "Rounding to power of 2 failed, expected 4");
    assert_eq!(round_to_power_of_2(5),
               8,
               "Rounding to power of 2 failed, expected 8");
}

/// Test that creation proceeds without a hitch.
#[test]
fn creation_test() {
    let mut i = 32;
    while i <= 1073741824 {
        let ro = ReorderedBuffer::new(i).unwrap();
        drop(ro);
        let ro = ReorderedBuffer::new(i + 1).unwrap();
        drop(ro);
        i *= 2;
    }
}

/// Test that in order insertion works correctly.
#[test]
fn test_in_order_insertion() {
    let mut ro = ReorderedBuffer::new(65536).unwrap();
    let data0 = "food";
    let base_seq = 1232;
    if let InsertionResult::Inserted { written, available } = ro.seq(base_seq, data0.as_bytes()) {
        assert_eq!(written,
                   data0.len(),
                   "When writing with seq, not all data was written expected {} got {}",
                   written,
                   data0.len());
        assert_eq!(available,
                   data0.len(),
                   "When writing in-order, not all data is available. Expected {} got {}",
                   available,
                   data0.len());
    } else {
        panic!("Seq failed");
    }

    let data1 = ": hamburger";
    if let InsertionResult::Inserted { written, available } =
        ro.add_data(base_seq.wrapping_add(data0.len() as u32), data1.as_bytes()) {
        assert_eq!(written, data1.len());
        assert_eq!(available,
                   data0.len() + data1.len(),
                   "Incorrect data available: Expected {} got {}",
                   data0.len() + data1.len(),
                   available);
    } else {
        panic!("Writing data1 failed");
    }

    let read_buf_len = data0.len() + data1.len() + 1;
    let mut read_buffer: Vec<_> = (0..read_buf_len).map(|_| 0).collect();
    let read = ro.read_data(&mut read_buffer[..]);
    assert_eq!(read,
               data0.len() + data1.len(),
               "Read less than expected, read: {}, expected: {}",
               read,
               data0.len() + data1.len());
    let read_str = str::from_utf8(&read_buffer[..read]).unwrap();
    assert_eq!(read_str,
               format!("{}{}", data0, data1),
               "Read does not match expected, read: {}, expected: {}",
               read_str,
               format!("{}{}", data0, data1));
}

/// Test that out of order insertion works correctly.
#[test]
fn test_out_of_order_insertion() {
    let mut ro = ReorderedBuffer::new(65536).unwrap();
    let data0 = "food";
    let base_seq = 1232;
    if let InsertionResult::Inserted { written, available } = ro.seq(base_seq, data0.as_bytes()) {
        assert_eq!(written, data0.len());
        assert_eq!(available, data0.len());
    } else {
        panic!("Seq failed");
    }

    let data1 = ": hamburger";
    let data2 = " american";
    if let InsertionResult::Inserted { written, available } =
        ro.add_data(base_seq
                        .wrapping_add(data0.len() as u32)
                        .wrapping_add(data1.len() as u32),
                    data2.as_bytes()) {
        assert_eq!(written, data2.len());
        assert_eq!(available, data0.len());
    } else {
        panic!("Writing data2 failed");
    }

    if let InsertionResult::Inserted { written, available } =
        ro.add_data(base_seq.wrapping_add(data0.len() as u32), data1.as_bytes()) {
        assert_eq!(written,
                   data1.len(),
                   "Unexpected write, expected {} got {}",
                   data1.len(),
                   written);
        assert_eq!(available, data0.len() + data1.len() + data2.len());
    } else {
        panic!("Writing data1 failed");
    }

    let read_buf_len = ro.available();
    let mut read_buffer: Vec<_> = (0..read_buf_len).map(|_| 0).collect();
    let read = ro.read_data(&mut read_buffer[..]);
    assert_eq!(read, read_buf_len, "Read less than what is available");
    assert_eq!(ro.available(),
               0,
               "Read everything but data is still available");
    let read = str::from_utf8(&read_buffer[..read]).unwrap();
    assert_eq!(read,
               format!("{}{}{}", data0, data1, data2),
               "Read does not match expected, read: {}, expected: {}",
               read,
               format!("{}{}{}", data0, data1, data2));
}

/// Test that things work fine once state is changed.
#[test]
fn test_state_change() {
    let mut ro = ReorderedBuffer::new(65536).unwrap();
    let data0 = "food";
    let base_seq = 1232;
    if let InsertionResult::Inserted { written, available } = ro.seq(base_seq, data0.as_bytes()) {
        assert_eq!(written, data0.len());
        assert_eq!(available, data0.len());
    } else {
        panic!("Seq failed");
    }

    let data1 = ": hamburger";
    let data2 = " american";
    let data3 = " (w/fries)";
    let data2_seq = base_seq
        .wrapping_add(data0.len() as u32)
        .wrapping_add(data1.len() as u32);
    if let InsertionResult::Inserted { written, available } = ro.add_data(data2_seq, data2.as_bytes()) {
        assert_eq!(written, data2.len());
        assert_eq!(available,
                   data0.len(),
                   "Incorrect data available, expected {} found {} (seq {}, base {})",
                   data0.len(),
                   available,
                   data2_seq,
                   base_seq);
    } else {
        panic!("Writing data2 failed");
    }

    if let InsertionResult::Inserted { written, available } =
        ro.add_data(base_seq.wrapping_add(data0.len() as u32), data1.as_bytes()) {
        assert_eq!(written,
                   data1.len(),
                   "Unexpected write, expected {} got {}",
                   data1.len(),
                   written);
        assert_eq!(available, data0.len() + data1.len() + data2.len());
    } else {
        panic!("Writing data1 failed");
    }

    if let InsertionResult::Inserted { written, available } =
        ro.add_data(base_seq
                        .wrapping_add(data0.len() as u32)
                        .wrapping_add(data1.len() as u32)
                        .wrapping_add(data2.len() as u32),
                    data3.as_bytes()) {
        assert_eq!(written, data3.len());
        assert_eq!(available,
                   data0.len() + data1.len() + data2.len() + data3.len());
    } else {
        panic!("Writing data3 failed");
    }
    let read_buf_len = ro.available();
    let mut read_buffer: Vec<_> = (0..read_buf_len).map(|_| 0).collect();
    let read = ro.read_data(&mut read_buffer[..]);
    assert_eq!(read, read_buf_len, "Read less than what is available");
    assert_eq!(ro.available(),
               0,
               "Read everything but data is still available");
    let read = str::from_utf8(&read_buffer[..read]).unwrap();
    assert_eq!(read,
               format!("{}{}{}{}", data0, data1, data2, data3),
               "Read does not match expected, read: {}, expected: {}",
               read,
               format!("{}{}{}{}", data0, data1, data2, data3));
}

/// Test that things OOM correctly when out of memory.
#[test]
fn test_oom() {
    println!("Running OOM test");
    let mut r0 = ReorderedBuffer::new(4096).unwrap();
    let base_seq = 32;
    let mut seq = base_seq;
    let data0 = "food";

    let iters = (4096 / data0.len()) - 1;
    if let InsertionResult::Inserted { written, .. } = r0.seq(base_seq, data0.as_bytes()) {
        assert_eq!(written, data0.len());
    } else {
        panic!("Could not write");
    }

    for _ in 1..iters {
        seq = seq.wrapping_add(data0.len() as u32);
        if let InsertionResult::Inserted { written, .. } = r0.add_data(seq, data0.as_bytes()) {
            assert_eq!(written, data0.len());
        } else {
            panic!("Could not write");
        }
    }
    seq = seq.wrapping_add(data0.len() as u32);
    if let InsertionResult::OutOfMemory { written, available } = r0.add_data(seq, data0.as_bytes()) {
        assert_ne!(written, data0.len());
        assert_eq!(available, 4096 - 1);
    } else {
        panic!("No OOM?");
    }
}

/// Test that reseting `ReorderedBuffer` works as expected.
#[test]
fn test_reset() {
    println!("Running test_reset");
    let mut r0 = ReorderedBuffer::new(4096).unwrap();
    let base_seq = 155;
    let mut seq = base_seq;
    let data0 = "food";

    let iters = (4096 / data0.len()) - 1;
    if let InsertionResult::Inserted { written, .. } = r0.seq(base_seq, data0.as_bytes()) {
        assert_eq!(written, data0.len());
    } else {
        panic!("Could not write");
    }

    for _ in 1..iters {
        seq = seq.wrapping_add(data0.len() as u32);
        if let InsertionResult::Inserted { written, .. } = r0.add_data(seq, data0.as_bytes()) {
            assert_eq!(written, data0.len());
        } else {
            panic!("Could not write");
        }
    }

    seq = seq.wrapping_add(data0.len() as u32);
    if let InsertionResult::OutOfMemory { written, available } = r0.add_data(seq, data0.as_bytes()) {
        assert_ne!(written, data0.len());
        assert_eq!(available, 4096 - 1);
    } else {
        panic!("No OOM?");
    }

    r0.reset();
    let base_seq = 72;
    let mut seq = base_seq;

    if let InsertionResult::Inserted { written, .. } = r0.seq(base_seq, data0.as_bytes()) {
        assert_eq!(written, data0.len());
    } else {
        panic!("Could not write");
    }

    for _ in 1..iters {
        seq = seq.wrapping_add(data0.len() as u32);
        if let InsertionResult::Inserted { written, .. } = r0.add_data(seq, data0.as_bytes()) {
            assert_eq!(written, data0.len());
        } else {
            panic!("Could not write");
        }
    }

    seq = seq.wrapping_add(data0.len() as u32);
    if let InsertionResult::OutOfMemory { written, available } = r0.add_data(seq, data0.as_bytes()) {
        assert_ne!(written, data0.len());
        assert_eq!(available, 4096 - 1);
    } else {
        panic!("No OOM?");
    }

}

/// Test that reading after writing allows us to write infinitely
#[test]
fn test_read_after_write() {
    println!("Running test_read_after_write");
    let mut r0 = ReorderedBuffer::new(4096).unwrap();
    let mut base_seq = u32::MAX - 30;
    let iters = 5000;
    let data = "testtest";

    if let InsertionResult::Inserted { written, .. } = r0.seq(base_seq, data.as_bytes()) {
        assert_eq!(written, data.len(), "Could not write during seq");
        base_seq = base_seq.wrapping_add(written as u32);
    } else {
        panic!("Could not seq");
    }

    let mut read_buf: Vec<_> = (0..data.len()).map(|_| 0).collect();

    for i in 0..iters {
        if let InsertionResult::Inserted { written, .. } = r0.add_data(base_seq, data.as_bytes()) {
            assert_eq!(written, data.len());
            base_seq = base_seq.wrapping_add(written as u32);
        } else {
            panic!("Could not write data, iter {} seq {}", i, base_seq);
        }

        let available_before_read = r0.available();

        let read = r0.read_data(&mut read_buf[..]);

        assert_eq!(available_before_read,
                   r0.available() + read,
                   "Available bytes not adjusted by the right amount");
    }
}

/// Test that overlapping writes work correctly.
#[test]
fn test_overlapping_write() {
    println!("Running test_overlapping_write");
    let mut r0 = ReorderedBuffer::new(4096).unwrap();
    let base_seq = 289;

    let data0 = "hello wo";
    let data1 = " world";

    if let InsertionResult::Inserted { written, .. } = r0.seq(base_seq, data0.as_bytes()) {
        assert_eq!(written, data0.len());
    } else {
        panic!("Could not write data");
    }

    if let InsertionResult::Inserted { written, .. } =
        r0.add_data(base_seq + ("hello".len() as u32), data1.as_bytes()) {
        assert_eq!(written,
                   "rld".len(),
                   "Overlapping write returns inconsistent result, expected {} got {}",
                   "rld".len(),
                   written);
    } else {
        panic!("Could not write data");
    }

    let mut read_buf: Vec<_> = (0..r0.available()).map(|_| 0).collect();
    let read = r0.read_data(&mut read_buf[..]);
    let read_str = str::from_utf8(&read_buf[..read]).unwrap();
    assert_eq!(read_str,
               "hello world",
               "Read value {} expected {}",
               read_str,
               "hello world");

    if let InsertionResult::Inserted { written, .. } = r0.add_data(base_seq, data0.as_bytes()) {
        assert_eq!(written, 0, "Wrote even though packet is from the past");
    } else {
        panic!("Could not write data");
    }
}
