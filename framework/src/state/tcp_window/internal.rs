use utils::RingBuffer;
use std::cmp::max;

#[allow(dead_code)]
enum InsertionResult {
    Inserted { length: usize },
    OutOfMemory,
}

#[allow(dead_code)]
enum State {
    Closed,
    Connected { seq: usize },
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
        Segment { idx: idx, prev: -1, next: -1, valid: false, begin: begin, length: length}
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

    #[inline]
    pub fn remove_head(&mut self) {
        if self.head != -1 {
            self.head = self.storage[self.head as usize].next;
        }
    }

    #[inline]
    pub fn remove_tail(&mut self) {
        if self.tail != -1 {
            self.tail = self.storage[self.tail as usize].prev;
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
                    return Some(&self.storage[idx as usize])
                } else if self.storage[idx as usize].begin > end { // We are on to segments that are further down, insert
                    let nidx = self.insert_before_node(idx, begin, len);
                    return Some(&self.storage[nidx as usize])
                } else if self.storage[idx as usize].begin <= begin { // Overlapping segment
                    let new_end = max(segment_end, end);
                    self.storage[idx as usize].length = new_end - self.storage[idx as usize].begin;
                    return Some(&self.storage[idx as usize]);
                }
            }
            // Nothing matched, so let us insert at the tail.
            idx = self.insert_at_tail(begin, len);
            Some(&self.storage[idx as usize])
        }
    }
    
    pub fn is_head(&self, seg: &Segment) -> bool {
        self.head == seg.idx
    }

    pub fn is_tail(&self, seg: &Segment) -> bool {
        self.tail == seg.idx
    }
}

#[allow(dead_code)]
struct ReorderedData {
    pub data: RingBuffer,
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

    pub fn new(buffer_size: usize, window_size: usize) -> Option<ReorderedData> {
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
                // Use less space than this if possible.
            })
        }
    }

    fn fast_path_insert(&mut self, data: &[u8]) -> InsertionResult {
        let mut written = 0;
        while written < data.len() {
            written += self.data.write_at_tail(data);
            while self.data.available() >= self.window_size {
                // Notify (thus potentially draining the buffer
            }
        }
        self.tail_seq += written;
        InsertionResult::Inserted { length: self.data.available() }
    }

    fn slow_path_insert(&mut self, seq: usize, data: &[u8]) -> InsertionResult {
        if self.tail_seq > seq { // Received an old segment, discard for now.
            InsertionResult::Inserted { length: self.data.available() }
        } else {
            let offset = seq - self.tail_seq;
            let inserted = self.data.write_at_offset_from_tail(offset, data);
            if inserted < data.len() {
                InsertionResult::OutOfMemory
            } else {
                InsertionResult::Inserted { length: self.data.available() }
            }
        }

    }

    pub fn seq(&mut self, seq: usize, data: &[u8]) -> InsertionResult {
        self.state = State::Connected {seq: seq};
        self.head_seq = seq;
        self.tail_seq = seq;
        self.fast_path_insert(data)
    }

    pub fn add_data(&mut self, seq: usize, data: &[u8]) -> InsertionResult {
        if seq == self.tail_seq { // Fast path
            self.fast_path_insert(data)
        } else { // Slow path
            self.slow_path_insert(seq, data)
        }
    }
}
