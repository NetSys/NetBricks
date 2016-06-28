use utils::RingBuffer;
use std::cmp::max;

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
    pub begin: usize,
    pub length: usize,
    pub next: isize,
    pub idx: isize,
}

impl Segment {
    pub fn new(idx: isize, begin: usize, length: usize) -> Segment {
        Segment { idx: idx, prev: -1, next: -1,  begin: begin, length: length }
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
            storage: (0..(length as isize)).map(|i| Segment::new(i, 0, 0)).collect(),
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
    fn insert_before_node(&mut self, next: isize, begin: usize, len: usize) -> isize {
        let idx = self.find_available_node();
        self.storage[idx as usize].begin = begin;
        self.storage[idx as usize].length = len;
        self.storage[idx as usize].next = next;
        if next != -1 {
            let prev = self.storage[next as usize].prev;
            self.storage[idx as usize].prev = prev;
            self.storage[next as usize].prev = idx;
            if prev != -1 {
                self.storage[prev as usize].next = idx;
            }
        } else {
            self.storage[idx as usize].prev = -1;
        }
        idx
    }

    /// Insert a node at the tail of the list.
    #[inline]
    fn insert_at_tail(&mut self, begin: usize, len: usize) -> isize {
        let idx = self.find_available_node();
        let idx_u = idx as usize;
        self.storage[idx_u].begin = begin;
        self.storage[idx_u].length = len;
        self.storage[idx_u].next = -1;
        self.storage[idx_u].prev = self.tail;
        self.storage[self.tail as usize].next = idx;
        self.tail = idx;
        idx
    }

    /// Insert a segment.
    #[inline]
    pub fn insert_segment(&mut self, begin: usize, len: usize) -> Option<isize> {
        let mut idx = self.head;
        if idx == -1 { // Special case the first insertion.
            idx = self.insert_before_node(-1, begin, len);
            self.head = idx;
            self.tail = idx;
            Some(idx)
        } else {
            let end = begin + len;
            while idx != -1 {
                let segment_end = self.storage[idx as usize].begin + self.storage[idx as usize].length;
                if segment_end == begin { // We can just add to the current segment.
                    self.storage[idx as usize].length += len;
                    break;
                } else if self.storage[idx as usize].begin >= end { // We are on to segments that are further down, insert
                    idx = self.insert_before_node(idx, begin, len);
                    break;
                } else if self.storage[idx as usize].begin <= begin { // Overlapping segment
                    let new_end = max(segment_end, end);
                    self.storage[idx as usize].length = new_end - self.storage[idx as usize].begin;
                    break;
                } else {
                    idx = self.storage[idx as usize].next;
                }
            }

            if idx == -1 {
                // Nothing matched, so let us insert at the tail.
                idx = self.insert_at_tail(begin, len);
                Some(idx)
            } else {
                // Check if we should/can merge. This only checks subsequent segments (since the prior ones should
                // already have been checked).
                let mut next = self.storage[idx as usize].next;
                while next != -1 {
                    let end = self.storage[idx as usize].begin + self.storage[idx as usize].length;
                    if  end <= self.storage[next as usize].begin {
                        // We should merge.
                        // Figure out how much of the next segment is non-overlapping
                        let merge_len = (end - self.storage[next as usize].begin) + self.storage[next as usize].length;
                        let to_free = next;
                        // Fix up pointers and length
                        self.storage[idx as usize].length += merge_len;
                        next = self.storage[to_free as usize].next;
                        self.storage[idx as usize].next = next;
                        self.storage[next as usize].prev = idx;
                        // Free the node
                        self.remove_node(to_free);

                    }
                }
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
    pub fn consume_head_data(&mut self, seq: usize, consumed: usize) -> bool {
        let idx = self.head as usize;
        // This is just an integrity check.
        if self.storage[idx].begin != seq {
            false
        } else {
            // Note we do not need to look beyond the head since we always merge segments.
            self.storage[idx].begin += consumed;
            self.storage[idx].length -= consumed;
            if self.storage[idx].length == 0 {
                self.remove_head();
            }
            true
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
    pub fn get_segment<'a>(&'a self, idx: isize) -> &'a Segment {
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
    head_seq: usize,
    tail_seq: usize,
}

const PAGE_SIZE: usize = 4096; // Page size in bytes, not using huge pages here.

impl ReorderedBuffer {
    // This is pub for testing, probably just move it out to utils
    #[inline]
    pub fn round_to_pages(buffer_size: usize) -> usize {
        (buffer_size + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1)
    }

    // This is pub for testing, probably just move it out to utils
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

    #[inline]
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    #[inline]
    pub fn available(&self) -> usize {
        self.data.available()
    }

    pub fn new(buffer_size: usize) -> ReorderedBuffer {
        ReorderedBuffer::new_with_segments(buffer_size, buffer_size / 64)
    }

    pub fn new_with_segments(buffer_size: usize, segment_size: usize) -> ReorderedBuffer {
        let page_aligned_size = ReorderedBuffer::round_to_pages(buffer_size);
        let pages = ReorderedBuffer::round_to_power_of_2(page_aligned_size / PAGE_SIZE);
        ReorderedBuffer {
            data: RingBuffer::new(pages).unwrap(),
            buffer_size: page_aligned_size,
            state: State::Closed,
            head_seq: 0,
            tail_seq: 0,
            segment_list: SegmentList::new(segment_size), // Assuming we don't receive small chunks.
        }
    }

    pub fn reset(&mut self) {
        self.state = State::Closed;
        self.segment_list.clear();
        self.data.clear();
    }

    fn fast_path_insert(&mut self, data: &[u8]) -> InsertionResult {
        let written = self.data.write_at_tail(data);
        self.tail_seq += written;
        if written == data.len() {
            InsertionResult::Inserted { written: written, available: self.available() }
        } else {
            InsertionResult::OutOfMemory { written: written, available: self.available() }
        }
    }

    fn slow_path_insert(&mut self, seq: usize, data: &[u8]) -> InsertionResult {
        let end = seq + data.len();

        if self.tail_seq > seq && end > self.tail_seq { // Some of the data overlaps with stuff we have received before.
            let begin = self.tail_seq - seq;
            self.fast_path_insert(&data[begin..])
        } else if end < self.tail_seq { // All the data overlaps.
            InsertionResult::Inserted { written: data.len(), available: self.available() }
        } else { // We are about to insert out of order data.
            // Change state to indicate we have out of order data; this means we need to do additional processing when
            // receiving data.
            self.state = State::ConnectedOutOfOrder;
            // Insert current in-order data into the segment list.
            self.segment_list.insert_segment(self.head_seq, self.data.available());
            // Call out-of-order insertion.
            self.out_of_order_insert(seq, data)
        }
    }

    fn out_of_order_insert(&mut self, seq: usize, data: &[u8]) -> InsertionResult {
        // FIXME: Transition back to Connected
        if self.tail_seq == seq { // Writing at tail
            // Write some data
            let mut written = self.data.write_at_tail(data);
            // Advance tail_seq based on written data (since write_at_tail already did that).
            self.tail_seq += written;
            {
                // Insert into segment list.
                let segment = self.segment_list.insert_segment(seq, written).unwrap();
                // Since we are writing to the beginning, this must always be the head.
                assert!(self.segment_list.is_head(segment));
                // Compute the end of the segment, this might in fact be larger than size
                let seg = self.segment_list.get_segment(segment);
                let seg_end = seg.begin + seg.length;
                // Integrity test.
                assert!(seg_end >= self.tail_seq);
                // We need to know the increment.
                let incr = seg_end - self.tail_seq;

                // If we have overlapped into data we received before, just drop the overlapping data.
                if written < incr {
                    written = incr;
                }
                self.tail_seq = seg_end; // Advance tail_seq
                self.data.seek_tail(incr); // Increment tail for the ring buffer.

            }
            if self.segment_list.one_segment() { // We only have one segment left, so now is a good time to switch
                                                 // back to fast path.
                self.segment_list.clear();
                self.state = State::Connected;
            }
            InsertionResult::Inserted { written: written, available: self.available() }
        } else if self.tail_seq >= seq {
            let offset = self.tail_seq - seq;
            let remaining = data.len() - offset;
            if remaining > 0 {
                let tail_seq = self.tail_seq;
                self.out_of_order_insert(tail_seq, &data[offset..])
            } else {
                InsertionResult::Inserted { written: data.len(), available: self.available() }
            }
        } else { // self.tail_seq < seq
            // Compute offset from tail where this should be written
            let offset = seq - self.tail_seq;
            // Write stuff
            let written = self.data.write_at_offset_from_tail(offset, data);
            // Insert segment at the right place
            self.segment_list.insert_segment(seq, written);
            if written == data.len() {
                InsertionResult::Inserted { written: written, available: self.available() }
            } else {
                InsertionResult::OutOfMemory { written: written, available: self.available() }
            }
        }
    }

    pub fn seq(&mut self, seq: usize, data: &[u8]) -> InsertionResult {
        self.state = State::Connected;
        self.head_seq = seq;
        self.tail_seq = seq;
        self.fast_path_insert(data)
    }

    pub fn add_data(&mut self, seq: usize, data: &[u8]) -> InsertionResult {
        match self.state {
            State::Connected => {
                if seq == self.tail_seq { // Fast path
                    self.fast_path_insert(data)
                } else { // Slow path
                    self.slow_path_insert(seq, data)
                }
            },
            State::ConnectedOutOfOrder => {
                self.out_of_order_insert(seq, data)
            },
            State::Closed => {
                panic!("Unexpected data");
            }
        }
    }

    #[inline]
    fn read_data_common(&mut self, mut data: &mut [u8]) -> usize {
        let read = self.data.read_from_head(data);
        self.head_seq = self.head_seq.wrapping_add(read);
        read
    }

    /// Read data from the buffer. The amount of data read is limited by the amount of in-order-data available at the
    /// moment.
    pub fn read_data(&mut self, mut data: &mut [u8]) -> usize {
        match self.state {
            State::Connected => self.read_data_common(data),
            State::ConnectedOutOfOrder => {
                let seq = self.head_seq;
                let read = self.read_data_common(data);
                // Record this in the segment list.
                self.segment_list.consume_head_data(seq, read);
                read
            },
            State::Closed => 0
        }
    }
}
