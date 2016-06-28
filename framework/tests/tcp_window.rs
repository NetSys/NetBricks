extern crate e2d2;
use e2d2::state::*;
use std::str;

/// Check rounding up to number of pages.
#[test]
fn round_pages_test() {
    assert!(ReorderedBuffer::round_to_pages(1) == 4096, "Rounding up 1 byte did not result in PAGE_SIZE");
    assert!(ReorderedBuffer::round_to_pages(0) == 0, "Rounding up 0 bytes did not result in 0");
    assert!(ReorderedBuffer::round_to_pages(8) == 4096, "Rounding up failure, expected 4096, got {}", 
            ReorderedBuffer::round_to_pages(8));
    assert!(ReorderedBuffer::round_to_pages(512) == 4096, "Rounding up failure, expected 4096, got {}",
            ReorderedBuffer::round_to_pages(512));
    assert!(ReorderedBuffer::round_to_pages(4096) == 4096, "Rounding up exactly 1 page failed, expected 4096, got {}",
                    ReorderedBuffer::round_to_pages(4096));
    assert!(ReorderedBuffer::round_to_pages(4097) == 8192, "Rounding up > 1 page failed, expected 8192, got {}",
                    ReorderedBuffer::round_to_pages(4097));
}

/// Test rounding up to power of 2.
#[test]
fn round_to_power_of_2_test() {
    assert!(ReorderedBuffer::round_to_power_of_2(0) == 0, "Rounding to power of 2 failed, expected 0");
    assert!(ReorderedBuffer::round_to_power_of_2(1) == 1, "Rounding to power of 2 failed, expected 1");
    assert!(ReorderedBuffer::round_to_power_of_2(2) == 2, "Rounding to power of 2 failed, expected 2");
    assert!(ReorderedBuffer::round_to_power_of_2(3) == 4, "Rounding to power of 2 failed, expected 4");
    assert!(ReorderedBuffer::round_to_power_of_2(4) == 4, "Rounding to power of 2 failed, expected 4");
    assert!(ReorderedBuffer::round_to_power_of_2(5) == 8, "Rounding to power of 2 failed, expected 8");
}

/// Check that creation proceeds without a hitch.
#[test]
fn creation_test() {
    for i in 128..131072 {
        let ro = ReorderedBuffer::new(i);
        drop(ro);
    }
}

/// Check that in order insertion works correctly.
#[test]
fn in_order_insertion() {
    let mut ro = ReorderedBuffer::new(65536);
    let data0 = "food";
    let base_seq = 1232;
    if let InsertionResult::Inserted{ written, available } = ro.seq(base_seq, data0.as_bytes()) {
        assert!(written == data0.len(), "When writing with seq, not all data was written expected {} got {}",
                written, data0.len());
        assert!(available == data0.len(), "When writing in-order, not all data is available. Expected {} got {}",
                available, data0.len());
    } else {
        panic!("Seq failed");
    }

    let data1 = ": hamburger";
    if let InsertionResult::Inserted{ written, available } = ro.add_data(base_seq + data0.len(), data1.as_bytes()) {
        assert!(written == data1.len());
        assert!(available == data0.len() + data1.len());
    } else {
        panic!("Writing data1 failed");
    }

    let read_buf_len = data0.len() + data1.len() + 1;
    let mut read_buffer: Vec<_> = (0..read_buf_len).map(|_| 0).collect();
    let read = ro.read_data(&mut read_buffer[..]);
    assert!(read == data0.len() + data1.len(), "Read less than expected, read: {}, expected: {}",
            read, data0.len() + data1.len());
    let read_str = str::from_utf8(&read_buffer[..read]).unwrap();
    assert!(read_str == format!("{}{}", data0, data1),
            "Read does not match expected, read: {}, expected: {}", read_str, format!("{}{}", data0, data1));
}

/// Check that out of order insertion works correctly.
#[test]
fn out_of_order_insertion() {
    let mut ro = ReorderedBuffer::new(65536);
    let data0 = "food";
    let base_seq = 1232;
    if let InsertionResult::Inserted{ written, available } = ro.seq(base_seq, data0.as_bytes()) {
        assert!(written == data0.len());
        assert!(available == data0.len());
    } else {
        panic!("Seq failed");
    }

    let data1 = ": hamburger";
    let data2 = " american";
    if let InsertionResult::Inserted{ written, available } = ro.add_data(base_seq + data0.len() + data1.len(),
                                                                         data2.as_bytes()) {
        assert!(written == data2.len());
        assert!(available == data0.len());
    } else {
        panic!("Writing data2 failed");
    }

    if let InsertionResult::Inserted{ written, available } = ro.add_data(base_seq + data0.len(), data1.as_bytes()) {
        assert!(written == data1.len());
        assert!(available == data0.len() + data1.len() + data2.len());
    } else {
        panic!("Writing data1 failed");
    }

    let read_buf_len = ro.available();
    let mut read_buffer: Vec<_> = (0..read_buf_len).map(|_| 0).collect();
    let read = ro.read_data(&mut read_buffer[..]);
    assert!(read == read_buf_len, "Read less than what is available");
    assert!(ro.available() == 0, "Read everything but data is still available");
    let read = str::from_utf8(&read_buffer[..read]).unwrap();
    assert!(read == format!("{}{}{}", data0, data1, data2),
            "Read does not match expected, read: {}, expected: {}",
            read, format!("{}{}{}", data0, data1, data2));
}

/// Check that things OOM correctly when out of memory.
#[test]
fn check_oom() {
    let mut r0 = ReorderedBuffer::new(4096);
    let base_seq = 32;
    let data0 = "food";

    let iters = (4096 / data0.len()) - 1;
    if let InsertionResult::Inserted { written, .. } = r0.seq(base_seq, data0.as_bytes()) {
        assert!(written == data0.len());
    } else {
        panic!("Could not write");
    }

    for i in 1..iters {
        if let InsertionResult::Inserted{ written, .. } = r0.add_data(base_seq + (i * data0.len()),
                                                                             data0.as_bytes()) {
            assert!(written == data0.len());
        } else {
            panic!("Could not write");
        }
    }

    if let InsertionResult::OutOfMemory{ written, available } = r0.add_data(base_seq + (iters * data0.len()),
                                                                            data0.as_bytes()) {
        assert!(written != data0.len());
        assert!(available == 4096 - 1);
    } else {
        panic!("No OOM?");
    }
}

/// Check that reseting `ReorderedBuffer` works as expected.
#[test]
fn check_reset() {
    let mut r0 = ReorderedBuffer::new(4096);
    let base_seq = 155;
    let data0 = "food";

    let iters = (4096 / data0.len()) - 1;
    if let InsertionResult::Inserted { written, .. } = r0.seq(base_seq, data0.as_bytes()) {
        assert!(written == data0.len());
    } else {
        panic!("Could not write");
    }

    for i in 1..iters {
        if let InsertionResult::Inserted{ written, .. } = r0.add_data(base_seq + (i * data0.len()),
                                                                             data0.as_bytes()) {
            assert!(written == data0.len());
        } else {
            panic!("Could not write");
        }
    }

    if let InsertionResult::OutOfMemory{ written, available } = r0.add_data(base_seq + (iters * data0.len()),
                                                                            data0.as_bytes()) {
        assert!(written != data0.len());
        assert!(available == 4096 - 1);
    } else {
        panic!("No OOM?");
    }

    r0.reset();
    let base_seq = 72;

    if let InsertionResult::Inserted { written, .. } = r0.seq(base_seq, data0.as_bytes()) {
        assert!(written == data0.len());
    } else {
        panic!("Could not write");
    }

    for i in 1..iters {
        if let InsertionResult::Inserted{ written, .. } = r0.add_data(base_seq + (i * data0.len()),
                                                                             data0.as_bytes()) {
            assert!(written == data0.len());
        } else {
            panic!("Could not write");
        }
    }

    if let InsertionResult::OutOfMemory{ written, available } = r0.add_data(base_seq + (iters * data0.len()),
                                                                            data0.as_bytes()) {
        assert!(written != data0.len());
        assert!(available == 4096 - 1);
    } else {
        panic!("No OOM?");
    }
}

/// Check that reading after writing allows us to write infinitely
#[test]
fn check_read_after_write() {
    let mut r0 = ReorderedBuffer::new(4096);
    let mut base_seq = 255;
    let iters = 128000;
    let data = "testtest";
    
    if let InsertionResult::Inserted { written, .. } = r0.seq(base_seq, data.as_bytes()) {
        assert!(written == data.len(), "Could not write during seq");
        base_seq = base_seq.wrapping_add(written);
    } else {
        panic!("Could not seq");
    }

    let mut read_buf : Vec<_> = (0..data.len()).map(|_| 0).collect();

    for i in 0..iters {
        if let InsertionResult::Inserted { written, .. } = r0.add_data(base_seq, data.as_bytes()) {
            assert!(written == data.len());
            base_seq = base_seq.wrapping_add(written);
        } else {
            panic!("Could not write code, iter {} seq {}", i, base_seq);
        }

        let available_before_read = r0.available();

        let read = r0.read_data(&mut read_buf[..]);

        assert!(available_before_read == r0.available() + read, "Available bytes not adjusted by the right amount");
    }
}
