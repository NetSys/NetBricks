extern crate e2d2;
use e2d2::state::*;

// Check rounding
#[test]
fn round_pages_test() {
    assert!(ReorderedData::round_to_pages(1) == 4096);
    assert!(ReorderedData::round_to_pages(0) == 0);
    assert!(ReorderedData::round_to_pages(8) == 4096);
    assert!(ReorderedData::round_to_pages(512) == 4096);
    assert!(ReorderedData::round_to_pages(4096) == 4096);
    assert!(ReorderedData::round_to_pages(4097) == 8192);
}

#[test]
fn round_to_power_of_2_test() {
    assert!(ReorderedData::round_to_power_of_2(0) == 0);
    assert!(ReorderedData::round_to_power_of_2(1) == 1);
    assert!(ReorderedData::round_to_power_of_2(2) == 2);
    assert!(ReorderedData::round_to_power_of_2(3) == 4);
    assert!(ReorderedData::round_to_power_of_2(4) == 4);
    assert!(ReorderedData::round_to_power_of_2(5) == 8);
}

// Check that creation proceeds without a hitch
#[test]
fn creation_test() {
    let ro = ReorderedData::new(65536);
    drop(ro);
    let ro2 = ReorderedData::new(128);
    drop(ro2);
}

#[test]
fn in_order_insertion() {
    let mut ro = ReorderedData::new(65536);
    let data0 = "food";
    let base_seq = 1232;
    if let InsertionResult::Inserted{ written, available } = ro.seq(base_seq, data0.as_bytes()) {
        assert!(written == data0.len());
        assert!(available == data0.len());
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
}

#[test]
fn out_of_order_insertion() {
    let mut ro = ReorderedData::new(65536);
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

    drop(ro);

    let mut r1 = ReorderedData::new(4096);

    let iters = (4096 / data0.len()) - 1;
    if let InsertionResult::Inserted { written, .. } = r1.seq(base_seq, data0.as_bytes()) {
        assert!(written == data0.len());
    } else {
        panic!("Could not write");
    }

    for i in 1..iters {
        if let InsertionResult::Inserted{ written, .. } = r1.add_data(base_seq + (i * data0.len()),
                                                                             data0.as_bytes()) {
            assert!(written == data0.len());
        } else {
            panic!("Could not write");
        }
    }

    if let InsertionResult::OutOfMemory{ written, available } = r1.add_data(base_seq + (iters * data0.len()),
                                                                            data0.as_bytes()) {
        assert!(written != data0.len());
        assert!(available == 4096 - 1);
    } else {
        panic!("No OOM?");
    }
}
