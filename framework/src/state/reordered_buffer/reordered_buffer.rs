use common::*;
use state::RingBuffer;
use std::cmp::{max, min};
use std::u16;
use utils::*;

/// Results from inserting into `ReorderedBuffer`
pub enum InsertionResult {
    /// Successfully inserted all the data (`written` should always be the same as the length of the input).
    Inserted { written: usize, available: usize },
    /// Inserted some of the data (recorded in `written`) but the buffer is out of space.
    OutOfMemory { written: usize, available: usize },
}

enum State {
    Closed,
    Connected,
    ConnectedOutOfOrder,
}

/// Structure to record unordered data.
#[derive(Default)]
struct Segment {
    pub prev: isize,
    pub seq: u32,
    pub length: u16,
    pub next: isize,
    pub idx: isize,
}

impl Segment {
    pub fn new(idx: isize, seq: u32, length: u16) -> Segment {
        Segment {
            idx: idx,
            prev: -1,
            next: -1,
            seq: seq,
            length: length,
        }
    }
}

/// A linked list of segments, this tries to avoid allocations whenever possible. In steady state (regardless of bad
/// choices made by the developer) there should be no allocations.
struct SegmentList {
    storage: Vec<Segment>,
    available: Vec<isize>,
    head: isize,
    tail: isize,
}

impl SegmentList {
    /// Create a segement list expecting that we will need no more than `length` segments.
    pub fn new(length: usize) -> SegmentList {
        SegmentList {
            storage: (0..(length as isize))
                .map(|i| Segment::new(i, 0, 0))
                .collect(),
            available: (0..(length as isize)).collect(),
            head: -1,
            tail: -1,
        }
    }

    /// Remove a node.
    fn remove_node(&mut self, node: isize) {
        self.storage[node as usize].length = 0;
        self.available.push(node);
    }

    /// Find an empty node that we can insert into the list.
    #[inline]
    fn find_available_node(&mut self) -> isize {
        if let Some(nidx) = self.available.pop() {
            nidx
        } else {
            let idx = self.storage.len() as isize;
            self.storage.push(Segment::new(idx, 0, 0));
            idx
        }
    }

    /// Insert a node before `next`.
    #[inline]
    fn insert_before_node(&mut self, next: isize, seq: u32, len: u16) -> isize {
        let idx = self.find_available_node();
        self.storage[idx as usize].seq = seq;
        self.storage[idx as usize].length = len;
        self.storage[idx as usize].next = next;
        if next != -1 {
            let prev = self.storage[next as usize].prev;
            self.storage[idx as usize].prev = prev;
            self.storage[next as usize].prev = idx;
            if prev != -1 {
                self.storage[prev as usize].next = idx;
            } else {
                self.head = idx; // No prev => this is head.
            }
        } else {
            self.storage[idx as usize].prev = -1;
        }
        idx
    }

    /// Insert a node after `prev`.
    #[inline]
    fn insert_after_node(&mut self, prev: isize, seq: u32, len: u16) -> isize {
        let idx = self.find_available_node();
        self.storage[idx as usize].seq = seq;
        self.storage[idx as usize].length = len;

        self.storage[idx as usize].prev = prev;

        self.storage[idx as usize].next = self.storage[prev as usize].next;
        self.storage[prev as usize].next = idx;

        if self.storage[idx as usize].next == -1 {
            self.tail = idx;
        }
        idx
    }

    /// Insert a node at the tail of the list.
    #[inline]
    fn insert_at_tail(&mut self, seq: u32, len: u16) -> isize {
        let idx = self.find_available_node();
        let idx_u = idx as usize;
        self.storage[idx_u].seq = seq;
        self.storage[idx_u].length = len;
        self.storage[idx_u].next = -1;
        self.storage[idx_u].prev = self.tail;
        self.storage[self.tail as usize].next = idx;
        self.tail = idx;
        idx
    }

    #[inline]
    fn merge_at_idx(&mut self, idx: isize) {
        // Check if we should/can merge. This only checks subsequent segments (since the prior ones should
        // already have been checked).
        let mut next = self.storage[idx as usize].next;
        while next != -1 {
            let end = self.storage[idx as usize]
                .seq
                .wrapping_add(self.storage[idx as usize].length as u32);
            if end >= self.storage[next as usize].seq {
                // We have at least some overlap, and should merge.
                let merge_len = self.storage[next as usize].length as usize -
                                (end - self.storage[next as usize].seq) as usize;
                let new_len = merge_len as usize + self.storage[idx as usize].length as usize;
                if new_len <= u16::MAX as usize {
                    self.storage[idx as usize].length = new_len as u16;
                    let to_free = next;
                    next = self.storage[to_free as usize].next;
                    self.storage[idx as usize].next = next;
                    if next != -1 {
                        self.storage[next as usize].prev = idx;
                    }
                    self.remove_node(to_free);
                } else {
                    let max_len = u16::MAX - self.storage[idx as usize].length;
                    // Add to previous segment
                    self.storage[idx as usize].length += max_len;
                    // Remove from next.
                    self.storage[next as usize].length -= max_len;
                    // Update seq
                    self.storage[next as usize].seq = self.storage[next as usize].seq.wrapping_add(max_len as u32);
                    // No more merges are possible so exit this loop.
                    break;
                }
            }
        }
    }

    /// Insert a segment.
    #[inline]
    pub fn insert_segment(&mut self, seq: u32, len: u16) -> Option<isize> {
        let mut idx = self.head;
        if idx == -1 {
            // Special case the first insertion.
            idx = self.insert_before_node(-1, seq, len);
            self.head = idx;
            self.tail = idx;
            Some(idx)
        } else {
            let end = seq.wrapping_add(len as u32);
            while idx != -1 {
                let seg_seq = self.storage[idx as usize].seq;
                let seg_len = self.storage[idx as usize].length;
                let seg_end = seg_seq.wrapping_add(seg_len as u32);
                if seg_end == seq {
                    // println!("Adjusting segment");
                    // We can just add to the current segment.
                    // We do not let lengths exceed 2^16 - 1.
                    let new_len = seg_len as usize + len as usize;
                    if new_len <= u16::MAX as usize {
                        // println!("No overflow, upping len to {}", new_len);
                        // No overflow.
                        self.storage[idx as usize].length = new_len as u16;
                    } else {
                        // println!("OVERFLOW adding another segment");
                        // Overflow.
                        // Compute how much can be safely added.
                        // We can only add bytes to get to u16::MAX.
                        let max_len = u16::MAX - self.storage[idx as usize].length;
                        // Add this much to the old segment.
                        self.storage[idx as usize].length += max_len;
                        let seq_new = seq.wrapping_add(max_len as u32);
                        let len_new = len - max_len;
                        self.insert_after_node(idx, seq_new, len_new);
                    }
                    break;
                } else if seg_seq >= end {
                    // println!("Adding before");
                    // We are on to segments that are further down, insert
                    idx = self.insert_before_node(idx, seq, len);
                    break;
                } else if self.storage[idx as usize].seq <= seq {
                    // println!("Overlapping");
                    // Overlapping segment
                    let new_end = max(seg_end, end);
                    self.storage[idx as usize].length = (new_end - self.storage[idx as usize].seq) as u16;
                    break;
                } else {
                    idx = self.storage[idx as usize].next;
                }
            }

            if idx == -1 {
                // Nothing matched, so let us insert at the tail.
                idx = self.insert_at_tail(seq, len);
                Some(idx)
            } else {
                self.merge_at_idx(idx);
                Some(idx)
            }
        }
    }

    /// Is `seg` the head of the list.
    pub fn is_head(&self, seg: isize) -> bool {
        self.head == seg
    }

    /// Remove the head of the list.
    #[inline]
    fn remove_head(&mut self) {
        let head = self.head;
        self.head = self.storage[head as usize].next;
        self.remove_node(head);
    }

    /// Consume some amount of data from the beginning.
    pub fn consume_head_data(&mut self, seq: u32, consumed: u16) -> bool {
        let idx = self.head as usize;
        // This is just an integrity check.
        if self.storage[idx].seq != seq {
            false
        } else {
            // No loops are necessary since we always
            let consume = min(consumed, self.storage[idx].length);
            self.storage[idx].seq = self.storage[idx].seq.wrapping_add(consume as u32);
            self.storage[idx].length -= consume;
            if self.storage[idx].length == 0 {
                self.remove_head();
            } else {
                self.merge_at_idx(idx as isize);
            }
            consume == consumed
        }
    }

    /// Clear the list.
    pub fn clear(&mut self) {
        let mut idx = self.head;
        while idx != -1 {
            let next = self.storage[idx as usize].next;
            self.remove_node(idx);
            idx = next;
        }
        self.head = -1;
        self.tail = -1;
    }

    /// Get a particular segment.
    #[inline]
    pub fn get_segment(&self, idx: isize) -> &Segment {
        &self.storage[idx as usize]
    }

    #[inline]
    pub fn one_segment(&self) -> bool {
        self.head == -1 || self.storage[self.head as usize].next == -1
    }
}

/// A structure for storing data that might be received out of order. This allows data to be inserted in any order, but
/// then allows ordered read from data.
pub struct ReorderedBuffer {
    data: RingBuffer,
    segment_list: SegmentList,
    buffer_size: usize,
    state: State,
    head_seq: u32,
    tail_seq: u32,
}


impl ReorderedBuffer {
    /// Return the size (the maximum amount of data) this buffer can hold.
    #[inline]
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Report how much data is available.
    #[inline]
    pub fn available(&self) -> usize {
        self.data.available()
    }

    /// Create a new buffer with space for `buffer_size` bytes.
    pub fn new(buffer_size: usize) -> Result<ReorderedBuffer> {
        ReorderedBuffer::new_with_segments(buffer_size, buffer_size / 64)
    }

    /// Create a new buffer with space for `buffer_size` bytes and a segment list with `segment_size` elements. The
    /// latter should be adjusted to reflect the expected number of out-of-order segments at a time.
    pub fn new_with_segments(buffer_size: usize, segment_size: usize) -> Result<ReorderedBuffer> {
        let rounded_bytes = round_to_power_of_2(buffer_size);
        let ring_buffer = try!{RingBuffer::new(rounded_bytes)};
        Ok(ReorderedBuffer {
               data: ring_buffer,
               buffer_size: rounded_bytes,
               state: State::Closed,
               head_seq: 0,
               tail_seq: 0,
               segment_list: SegmentList::new(segment_size), // Assuming we don't receive small chunks.
           })
    }


    /// Reset buffer state.
    pub fn reset(&mut self) {
        self.state = State::Closed;
        self.segment_list.clear();
        self.data.clear();
    }

    /// Set the current sequence number for the buffer.
    pub fn seq(&mut self, seq: u32, data: &[u8]) -> InsertionResult {
        match self.state {
            State::Closed => {
                self.state = State::Connected;
                self.head_seq = seq;
                self.tail_seq = seq;
                self.fast_path_insert(data)
            }
            _ => panic!("Cannot seq a buffer that has already been sequed"),
        }
    }

    /// Add data at a given sequence number.
    pub fn add_data(&mut self, seq: u32, data: &[u8]) -> InsertionResult {
        match self.state {
            State::Connected => {
                if seq == self.tail_seq {
                    // Fast path
                    self.fast_path_insert(data)
                } else {
                    // Slow path
                    self.slow_path_insert(seq, data)
                }
            }
            State::ConnectedOutOfOrder => self.out_of_order_insert(seq, data),
            State::Closed => {
                panic!("Unexpected data");
            }
        }
    }

    /// Read data from the buffer. The amount of data read is limited by the amount of in-order data available at the
    /// moment.
    pub fn read_data(&mut self, mut data: &mut [u8]) -> usize {
        match self.state {
            State::Connected => self.read_data_common(data),
            State::ConnectedOutOfOrder => {
                let seq = self.head_seq;
                let read = self.read_data_common(data);
                // Record this in the segment list.
                self.segment_list.consume_head_data(seq, read as u16);
                read
            }
            State::Closed => 0,
        }
    }

    #[inline]
    pub fn is_established(&self) -> bool {
        match self.state {
            State::Closed => false,
            _ => true,
        }
    }

    fn fast_path_insert(&mut self, data: &[u8]) -> InsertionResult {
        let written = self.data.write_at_tail(data);
        self.tail_seq = self.tail_seq.wrapping_add(written as u32);
        if written == data.len() {
            InsertionResult::Inserted {
                written: written,
                available: self.available(),
            }
        } else {
            InsertionResult::OutOfMemory {
                written: written,
                available: self.available(),
            }
        }
    }

    #[inline]
    fn add_head_to_seg_list(&mut self) {
        let mut to_insert = self.data.available();
        let mut seq = self.head_seq;
        while to_insert > 0 {
            let insert = min(u16::MAX as usize, to_insert) as u16;
            self.segment_list.insert_segment(seq, insert);
            seq = seq.wrapping_add(insert as u32);
            to_insert -= insert as usize;
        }
    }

    fn slow_path_insert(&mut self, seq: u32, data: &[u8]) -> InsertionResult {
        let end = seq.wrapping_add(data.len() as u32);

        if self.tail_seq > seq && end > self.tail_seq {
            // Some of the data overlaps with stuff we have received before.
            let begin = (self.tail_seq - seq) as usize;
            self.fast_path_insert(&data[begin..])
        } else if end < self.tail_seq {
            // All the data overlaps.
            InsertionResult::Inserted {
                written: 0,
                available: self.available(),
            }
        } else {
            // We are about to insert out of order data.
            // Change state to indicate we have out of order data; this means we need to do additional processing when
            // receiving data.
            self.state = State::ConnectedOutOfOrder;
            // Insert current in-order data into the segment list.
            self.add_head_to_seg_list();
            // Call out-of-order insertion.
            self.out_of_order_insert(seq, data)
        }
    }

    fn out_of_order_insert(&mut self, seq: u32, data: &[u8]) -> InsertionResult {
        if self.tail_seq == seq {
            // Writing at tail
            // Write some data
            let mut written = self.data.write_at_tail(data);
            // Advance tail_seq based on written data (since write_at_tail already did that).
            self.tail_seq = self.tail_seq.wrapping_add(written as u32);
            {
                // Insert into segment list.
                let segment = self.segment_list
                    .insert_segment(seq, written as u16)
                    .unwrap();
                // Since we are writing to the beginning, this must always be the head.
                assert!(self.segment_list.is_head(segment));
                // Compute the end of the segment, this might in fact be larger than size
                let seg = self.segment_list.get_segment(segment);
                let seg_end = seg.seq.wrapping_add(seg.length as u32);
                // We need to know the increment.
                let incr = seg_end.wrapping_sub(self.tail_seq);

                // If we have overlapped into data we received before, just drop the overlapping data.
                if (written as u32) < incr {
                    written = incr as usize;
                }
                self.tail_seq = seg_end; // Advance tail_seq
                self.data.seek_tail(incr as usize); // Increment tail for the ring buffer.

            }

            if self.segment_list.one_segment() {
                // We only have one segment left, so now is a good time to switch
                // back to fast path.
                self.segment_list.clear();
                self.state = State::Connected;
            }

            InsertionResult::Inserted {
                written: written,
                available: self.available(),
            }
        } else if self.tail_seq >= seq {
            let offset = (self.tail_seq - seq) as usize;
            let remaining = data.len() - offset;
            if remaining > 0 {
                let tail_seq = self.tail_seq;
                self.out_of_order_insert(tail_seq, &data[offset..])
            } else {
                InsertionResult::Inserted {
                    written: 0,
                    available: self.available(),
                }
            }
        } else {
            // self.tail_seq < seq
            // Compute offset from tail where this should be written
            let offset = (seq - self.tail_seq) as usize;
            // Write stuff
            let written = self.data.write_at_offset_from_tail(offset, data);
            // Insert segment at the right place
            self.segment_list.insert_segment(seq, written as u16);
            if written == data.len() {
                InsertionResult::Inserted {
                    written: written,
                    available: self.available(),
                }
            } else {
                InsertionResult::OutOfMemory {
                    written: written,
                    available: self.available(),
                }
            }
        }
    }

    #[inline]
    fn read_data_common(&mut self, mut data: &mut [u8]) -> usize {
        let read = self.data.read_from_head(data);
        self.head_seq = self.head_seq.wrapping_add(read as u32);
        read
    }
}
