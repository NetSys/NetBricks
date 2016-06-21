use std::io::Write;
use utils::RingBuffer;

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
    pub valid: bool,
    pub begin: usize,
    pub length: usize,
}

impl Segment {
    pub fn new(begin: usize, length: usize) {
        Segment { valid: false, begin: begin, length: length}
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    #[inline]
    pub fn reset(&mut self, length: usize) {
        self.valid = false;
        self.length = length;
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
    pub segments: Vec<Segment>,
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
        size |= (size >> 1);
        size |= (size >> 2);
        size |= (size >> 4);
        size |= (size >> 8);
        size |= (size >> 16);
        size |= (size >> 32);
        size += 1;
        size
    }

    pub fn new(buffer_size: usize, window_size: usize) -> Option<ReorderedData> {
        if window_size >= buffer_size {
            None
        }
        let page_aligned_size = round_to_pages(buffer_size);
        let pages = round_to_power_of_2(page_aligned_size / PAGE_SIZE);
        Some(ReorderedData {
            data: RingBuffer::new(pages).unwrap(),
            buffer_size: page_aligned_size,
            window_size: window_size,
            state: State::Closed,
            head_seq: 0,
            tail_seq: 0,
            // Use less space than this if possible.
            segments: (0..page_aligned_size).map(|b| Segment::new(b, 1)).collect(),
        })
    }

    fn fast_path_insert(&mut self, data: &[u8]) -> InsertResult {
        let mut written = 0;
        while written < data.len() {
            written += self.data.write_at_tail(data);
            while self.data.available() >= window_size {
                // Notify (thus potentially draining the buffer
            } 
        }
        self.tail_seq += written;
        InsertResult::Inserted { length: self.data.available() }
    }

    fn slow_path_insert(&mut self, seq: usize, data: &[u8]) -> InsertResult {
        if self.tail_seq > seq { // Received an old segment, discard for now.
            InsertResult::InsertResult { length: self.data.available() }
        } else {
            let offset = seq - self.tail_seq;
            let inserted = self.data.write_at_offset_from_tail(offset, data);
            if inserted < data.len() {
                InsertResult::OutOfMemory
            } else {
                InsertResult::Inserted { length: self.data.available() }
            }
        }

    }

    pub fn seq(&mut self, seq: usize, data: &[u8]) -> InsertResult {
        self.state = State::Connected {seq: seq};
        self.head_seq = seq;
        self.tail_seq = seq;
        self.fast_path_insert(data)
    }

    pub fn add_data(&mut self, seq: usize, data: &[u8]) -> InsertResult {
        if seq == self.tail_seq { // Fast path
            self.fast_path_insert(data)
        } else { // Slow path
            self.slow_path_insert(seq, data)
        }
    }
}

//#[derive(Default)]
//struct Segment {
    //pub available: bool,
    //pub length: usize,
//}

//#[allow(dead_code)]
//struct WindowInternal {
    //pub data: Vec<u8>,
    //pub seq: usize,
    //pub recvd: Vec<Segment>,
    //pub complete: bool,
    //pub size: usize,
//}

//#[allow(dead_code)]
//impl WindowInternal {
    //pub fn new(size: usize) -> WindowInternal {
        //WindowInternal {
            //data: (0..size).map(|_| 0).collect(),
            //seq: 0,
            //recvd: (0..size).map(|_| Default::default()).collect(),
            //size: size,
            //complete: false,
        //}
    //}

    //#[inline]
    //pub fn reset(&mut self, seq: usize) {
        //self.recvd = (0..self.size).map(|_| Default::default()).collect();
        //self.seq = seq;
        //self.complete = false;
    //}

    //#[inline]
    //pub fn compute_offset(&self, seq: usize) -> Option<usize> {
        //let offset = seq - self.seq;
        //if offset < self.size {
            //Some(offset)
        //} else {
            //None
        //}
    //}

    //#[inline]
    //pub fn compute_offset_unchecked(&self, seq: usize) -> usize {
        //seq - self.seq
    //}

    //#[inline]
    //fn roll_up_adjacent(&mut self, idx: usize) -> usize {
        //// Combine all adjacent data segments.
        //let mut len = self.recvd[idx].length;
        //let max_len = self.size - idx;
        //while len < max_len && self.recvd[idx + len].available {
            //len += self.recvd[idx + len].length;
        //}
        //self.recvd[idx].length = len;
        //len
    //}

    //#[inline]
    //pub fn insert_at_seq(&mut self, seq: usize, data: &[u8]) -> InsertResult {
        //match self.compute_offset(seq) {
            //Some(offset) => {
                //// let len = min(self.data.len() - offset, data.len());
                //let idx = offset;
                //self.recvd[idx].length += (&mut self.data[offset..]).write(data).unwrap();
                //self.recvd[idx].available = true;
                //// Check if complete at head.
                //let len = self.roll_up_adjacent(0);
                //if len == self.size {
                    //self.complete = true; // Marked as complete
                    //InsertResult {
                        //result: Result::Completed,
                        //length: len,
                    //}
                //} else {
                    //InsertResult {
                        //result: Result::Inserted,
                        //length: len,
                    //}
                //}
            //}
            //None => {
                //InsertResult {
                    //result: Result::Failed,
                    //length: 0,
                //}
            //}
        //}
    //}
//}
