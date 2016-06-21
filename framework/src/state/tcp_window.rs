use std::io::Write;

#[allow(dead_code)]
enum Result {
    Completed,
    Inserted,
    Failed,
}

#[allow(dead_code)]
struct InsertResult {
    pub result: Result,
    pub length: usize,
}

#[derive(Default)]
struct Segment {
    pub available: bool,
    pub length: usize,

}

#[allow(dead_code)]
struct WindowInternal {
    pub data: Vec<u8>,
    pub seq: usize,
    pub recvd: Vec<Segment>,
    pub complete: bool,
    pub size: usize,
}

#[allow(dead_code)]
impl WindowInternal {
    pub fn new(size: usize) -> WindowInternal {
        WindowInternal {
            data: (0..size).map(|_| 0).collect(),
            seq: 0,
            recvd: (0..size).map(|_| Default::default()).collect(),
            size: size,
            complete:false,
        }
    }

    #[inline]
    pub fn reset(&mut self, seq: usize) {
        self.recvd = (0..self.size).map(|_| Default::default()).collect();
        self.seq = seq;
        self.complete = false;
    }

    #[inline]
    pub fn compute_offset(&self, seq: usize) -> Option<usize> {
        let offset = seq - self.seq;
        if offset < self.size {
            Some(offset)
        } else {
            None
        }
    }

    #[inline]
    pub fn compute_offset_unchecked(&self, seq: usize) -> usize {
        seq - self.seq
    }

    #[inline]
    fn roll_up_adjacent(&mut self, idx: usize) -> usize{
        // Combine all adjacent data segments.
        let mut len = self.recvd[idx].length;
        let max_len = self.size - idx;
        while len < max_len && self.recvd[idx + len].available {
            len += self.recvd[idx + len].length;
        }
        self.recvd[idx].length = len;
        len
    }

    #[inline]
    pub fn insert_at_seq(&mut self, seq: usize, data: &[u8]) -> InsertResult {
        match self.compute_offset(seq) {
            Some(offset) => {
                //let len = min(self.data.len() - offset, data.len());
                let idx = offset;
                self.recvd[idx].length += (&mut self.data[offset..]).write(data).unwrap();
                self.recvd[idx].available = true;
                // Check if complete at head.
                let len = self.roll_up_adjacent(0);
                if len == self.size {
                    self.complete = true; // Marked as complete
                    InsertResult {result: Result::Completed, length: len}
                } else {
                    InsertResult {result: Result::Inserted, length: len}
                }
            },
            None => InsertResult{result: Result::Failed, length: 0}
        }
    }
}
