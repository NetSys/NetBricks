use utils::RingBuffer;
use std::cmp::max;

#[allow(dead_code)]
enum InsertionResult {
    Inserted,
    OutOfMemory {length: usize},
}

#[allow(dead_code)]
enum State {
    Closed,
    Connected,
    ConnectedOutOfOrder,
}

#[allow(dead_code)]
#[derive(Default)]
struct Segment {
    pub prev: isize,
    pub valid: bool,
    pub begin: usize,
    pub length: usize,
    pub next: isize,
    pub idx: isize,
}

impl Segment {
    pub fn new(idx: isize, begin: usize, length: usize) -> Segment {
        Segment { idx: idx, prev: -1, next: -1, valid: false, begin: begin, length: length }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    #[inline]
    pub fn reset(&mut self, length: usize) {
        self.valid = false;
        self.length = length;
        self.prev = -1;
        self.next = -1;
    }
}

#[allow(dead_code)]
struct SegmentList {
    storage: Vec<Segment>,
    available: Vec<isize>,
    head: isize,
    tail: isize,
}

impl SegmentList {
    pub fn new(length: usize) -> SegmentList {
        SegmentList {
            storage: (0..(length as isize)).map(|i| Segment::new(i, 0, 0)).collect(),
            available: (0..(length as isize)).collect(),
            head: -1,
            tail: -1,
        }
    }

    #[inline]
    pub fn head<'a>(&'a self) -> Option<&'a Segment> {
        if self.head == -1 {
            None
        } else {
            Some(&self.storage[self.head as usize])
        }
    }

    #[inline]
    pub fn tail<'a>(&'a self) -> Option<&'a Segment> {
        if self.tail == -1 {
            None
        } else {
            Some(&self.storage[self.tail as usize])
        }
    }
    fn remove_node(&mut self, node: isize) {
        self.storage[node as usize].valid = false;
        self.storage[node as usize].length = 0;
        self.available.push(node);
    }

    #[inline]
    pub fn remove_head(&mut self) {
        if self.head != -1 {
            let head = self.head;
            self.head = self.storage[self.head as usize].next;
            self.remove_node(head);
        }
    }

    #[inline]
    pub fn remove_tail(&mut self) {
        if self.tail != -1 {
            let tail = self.tail;
            self.tail = self.storage[self.tail as usize].prev;
            self.remove_node(tail);
        }
    }

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
    #[inline]
    fn insert_before_node(&mut self, next: isize, begin: usize, len: usize) -> isize {
        let idx = self.find_available_node();
        self.storage[idx as usize].begin = begin;
        self.storage[idx as usize].length = len;
        self.storage[idx as usize].valid = true;
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

    #[inline]
    fn insert_at_tail(&mut self, begin: usize, len: usize) -> isize {
        let idx = self.find_available_node();
        let idx_u = idx as usize;
        self.storage[idx_u].begin = begin;
        self.storage[idx_u].length = len;
        self.storage[idx_u].valid = true;
        self.storage[idx_u].next = -1;
        self.storage[idx_u].prev = self.tail;
        self.storage[self.tail as usize].next = idx;
        self.tail = idx;
        idx
    }

    #[inline]
    pub fn insert_segment<'a>(&'a mut self, begin: usize, len: usize) -> Option<&'a Segment> {
        let mut idx = self.head;
        if idx == -1 { // Special case the first insertion.
            idx = self.insert_before_node(-1, begin, len);
            self.head = idx;
            self.tail = idx;
            Some(&self.storage[idx as usize])
        } else {
            let end = begin + len;
            while idx != -1 {
                let segment_end = self.storage[idx as usize].begin + self.storage[idx as usize].length;
                if segment_end == begin { // We can just add to the current segment.
                    self.storage[idx as usize].length += len;
                    //return Some(&self.storage[idx as usize])
                } else if self.storage[idx as usize].begin > end { // We are on to segments that are further down, insert
                    idx = self.insert_before_node(idx, begin, len);
                    //return Some(&self.storage[nidx as usize])
                } else if self.storage[idx as usize].begin <= begin { // Overlapping segment
                    let new_end = max(segment_end, end);
                    self.storage[idx as usize].length = new_end - self.storage[idx as usize].begin;
                    //return Some(&self.storage[idx as usize]);
                }
            }

            if idx == -1 {
                // Nothing matched, so let us insert at the tail.
                idx = self.insert_at_tail(begin, len);
                Some(&self.storage[idx as usize])
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
                Some(&self.storage[idx as usize])
            }
        }
    }

    pub fn is_head(&self, seg: &Segment) -> bool {
        self.head == seg.idx
    }

    pub fn is_tail(&self, seg: &Segment) -> bool {
        self.tail == seg.idx
    }

    pub fn consume_head_data(&mut self, seq: usize, consumed: usize) -> bool {
        let idx = self.head as usize;
        if self.storage[idx].begin != seq {
            false
        } else {
            self.storage[idx].begin += consumed;
            self.storage[idx].length -= consumed;
            if self.storage[idx].length == 0 {
                self.remove_head();
            }
            true
        }
    }
}

#[allow(dead_code)]
struct ReorderedData {
    pub data: RingBuffer,
    pub segment_list: SegmentList,
    pub buffer_size: usize,
    pub window_size: usize,
    pub state: State,
    pub head_seq: usize,
    pub tail_seq: usize,
}

const PAGE_SIZE: usize = 4096; // Page size in bytes, not using huge pages here.

#[allow(dead_code)]
impl ReorderedData {
    #[inline]
    fn round_to_pages(buffer_size: usize) -> usize {
        (buffer_size + (PAGE_SIZE - 1)) & PAGE_SIZE
    }

    #[inline]
    fn round_to_power_of_2(mut size: usize) -> usize {
        size -= 1;
        size |= size >> 1;
        size |= size >> 2;
        size |= size >> 4;
        size |= size >> 8;
        size |= size >> 16;
        size |= size >> 32;
        size += 1;
        size
    }

    #[inline]
    pub fn available(&self) -> usize {
        self.data.available()
    }

    pub fn new(buffer_size: usize, window_size: usize) -> Option<ReorderedData> {
        ReorderedData::new_with_segments(buffer_size, window_size, buffer_size / 64)
    }

    pub fn new_with_segments(buffer_size: usize, window_size: usize, segment_size: usize) -> Option<ReorderedData> {
        if window_size >= buffer_size {
            None
        } else {
            let page_aligned_size = ReorderedData::round_to_pages(buffer_size);
            let pages = ReorderedData::round_to_power_of_2(page_aligned_size / PAGE_SIZE);
            Some(ReorderedData {
                data: RingBuffer::new(pages).unwrap(),
                buffer_size: page_aligned_size,
                window_size: window_size,
                state: State::Closed,
                head_seq: 0,
                tail_seq: 0,
                segment_list: SegmentList::new(segment_size), // Assuming we don't receive small chunks.
            })
        }
    }

    fn fast_path_insert(&mut self, data: &[u8]) -> InsertionResult {
        let mut written = 0;
        while written < data.len() {
            written += self.data.write_at_tail(&data[written..]);
            while self.data.available() >= self.window_size {
                // Notify (thus potentially draining the buffer
            }
        }
        self.tail_seq += written;
        InsertionResult::Inserted
    }

    fn slow_path_insert(&mut self, seq: usize, data: &[u8]) -> InsertionResult {
        let end = seq + data.len();
 
        if self.tail_seq > seq && end < self.tail_seq { // Some of the data overlaps with stuff we have received before.
            let begin = self.tail_seq - seq;
            self.fast_path_insert(&data[begin..])
        } else if end > self.tail_seq { // All the data overlaps.
            InsertionResult::Inserted
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
        if self.tail_seq == seq { // Writing at tail
            let mut written = 0;
            while written < data.len() {
                // Write some data
                written += self.data.write_at_tail(&data[written..]);
                // Insert into segment list.
                let segment = self.segment_list.insert_segment(self.tail_seq, written).unwrap();
                // Since we are writing to the beginning, this must always be the head.
                //assert!(self.segment_list.is_head(segment));
                // Compute the end of the segment, this might in fact be larger than size
                let seg_end = segment.begin + segment.length;
                // Integrity test.
                assert!(seg_end >= self.tail_seq);
                // We need to know the increment.
                let incr = seg_end - self.tail_seq;
                // If we have overlapped into data we received before, just drop the overlapping data.
                if written < incr {
                    written = incr;
                }

                self.tail_seq = seg_end; // Advance tail_seq
                self.data.increment_tail(incr); // Increment tail for the ring buffer.

                while self.data.available() >= self.window_size {
                    // self.consume_head_data(...)
                }
            }
            InsertionResult::Inserted
        } else if self.tail_seq >= seq {
            let offset = self.tail_seq - seq;
            let remaining = data.len() - offset;
            if remaining > 0 {
                let tail_seq = self.tail_seq;
                self.out_of_order_insert(tail_seq, &data[offset..])
            } else {
                InsertionResult::Inserted
            }
        } else { // self.tail_seq < seq
            let offset = seq - self.tail_seq;
            let written = self.data.write_at_offset_from_tail(offset, data);
            if written == data.len() {
                InsertionResult::Inserted
            } else {
                InsertionResult::OutOfMemory {length: written}
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
}
