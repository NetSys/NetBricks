use io::MBuf;
use std::ptr;
use std::marker::PhantomData;
use std::slice;
use headers::{EndOffset, NullHeader};
// Some low level functions.
#[link(name = "zcsi")]
extern "C" {
    fn mbuf_alloc() -> *mut MBuf;
    fn mbuf_free(buf: *mut MBuf);
    fn mbuf_alloc_bulk(array: *mut *mut MBuf, len: u16, cnt: i32) -> i32;
}

/// A packet is a safe wrapper around mbufs, that can be allocated and manipulated.
/// We associate a header type with a packet to allow safe insertion of headers.
pub struct Packet<T: EndOffset> {
    mbuf: *mut MBuf,
    offset: usize,
    _phantom_t: PhantomData<T>,
}

fn reference_mbuf(mbuf: *mut MBuf) {
    unsafe { (*mbuf).reference() };
}

/// All the iterators should switch to returning this type, it makes much more sense.
pub trait PacketFromMbuf {
    fn from_mbuf<T: EndOffset>(mbuf: *mut MBuf, offset: usize) -> Packet<T> {
        // Need to up the refcnt, so that things don't drop.
        reference_mbuf(mbuf);
        Packet {
            mbuf: mbuf,
            offset: offset,
            _phantom_t: PhantomData,
        }
    }
}

impl<T: EndOffset> Drop for Packet<T> {
    fn drop(&mut self) {
        if !self.mbuf.is_null() {
            unsafe { mbuf_free(self.mbuf) };
        }
    }
}

impl<T: EndOffset> PacketFromMbuf for Packet<T> {}

/// Allocate a new packet.
pub fn new_packet() -> Option<Packet<NullHeader>> {
    unsafe {
        // This sets refcnt = 1
        let mbuf = mbuf_alloc();
        if mbuf.is_null() {
            None
        } else {
            Some(Packet {
                mbuf: mbuf,
                offset: 0,
                _phantom_t: PhantomData,
            })
        }
    }
}

pub fn new_packet_array(count: usize) -> Vec<Packet<NullHeader>> {
    let mut array = Vec::with_capacity(count);
    unsafe {
        let alloc_ret = mbuf_alloc_bulk(array.as_mut_ptr(), 0, count as i32);
        if alloc_ret == 0 {
            array.set_len(count);
        }
    }
    array.iter().map(|m| Packet {mbuf: m.clone(), offset: 0, _phantom_t: PhantomData}).collect()
}

impl<T: EndOffset> Packet<T> {
    #[inline]
    fn data(&self) -> *mut u8 {
        unsafe { (*self.mbuf).data_address(self.offset) }
    }

    #[inline]
    fn data_len(&self) -> usize {
        unsafe { (*self.mbuf).data_len() }
    }

    #[inline]
    fn payload_size(&self) -> usize {
        self.data_len() - self.offset
    }

    pub fn push_header<T2: EndOffset<PreviousHeader = T>>(mut self, header: &T2) -> Option<Packet<T2>> {
        unsafe {
            let len = self.data_len();
            let size = header.offset();
            let added = (*self.mbuf).add_data_end(size);
            if added >= size {
                let dst = if len != self.offset as usize {
                    // Need to move down the rest of the data down.
                    let final_dst = self.data();
                    let move_loc = final_dst.offset(size as isize);
                    let to_move = len - self.offset;
                    ptr::copy_nonoverlapping(final_dst, move_loc, to_move);
                    final_dst as *mut T2
                } else {
                    self.data() as *mut T2
                };
                ptr::copy_nonoverlapping(header, dst, 1);
                let mbuf = self.mbuf;
                self.mbuf = ptr::null_mut(); // We null this out here because drop is more expensive than this.
                Some(Packet {
                    mbuf: mbuf,
                    offset: self.offset + size,
                    _phantom_t: PhantomData,
                })
            } else {
                None
            }
        }
    }

    #[inline]
    pub fn get_mut_payload(&mut self) -> &mut [u8] {
        unsafe {
            let len = self.payload_size();
            let ptr = self.data();
            slice::from_raw_parts_mut(ptr, len)
        }
    }

    #[inline]
    pub fn get_payload(&self) -> &[u8] {
        unsafe {
            let len = self.payload_size();
            let ptr = self.data();
            slice::from_raw_parts(ptr, len)
        }
    }

    #[inline]
    pub fn increase_payload_size(&mut self, increase_by: usize) -> usize {
        unsafe { (*self.mbuf).add_data_end(increase_by) }
    }

    #[inline]
    pub fn trim_payload_size(&mut self, trim_by: usize) -> usize {
        unsafe { (*self.mbuf).remove_data_end(trim_by) }
    }

    #[inline]
    pub fn copy_payload(&mut self, other: &Self) -> usize {
        let copy_len = other.payload_size();
        let dst = self.data();
        let src = other.data();

        let payload_size = self.payload_size();

        let should_copy = if payload_size < copy_len {
            let increment = copy_len - payload_size;
            self.increase_payload_size(increment)
        } else {
            copy_len
        };

        unsafe {
            ptr::copy_nonoverlapping(src, dst, should_copy);
            should_copy
        }
    }

    #[inline]
    pub fn refcnt(&self) -> u16 {
        unsafe { (*self.mbuf).refcnt() }
    }

    /// Get the mbuf reference by this packet.
    ///
    /// # Safety
    /// The reference held by this Packet is nulled out as a result of this code. The callee is responsible for
    /// appropriately freeing this mbuf from here-on out.
    #[inline]
    pub unsafe fn get_mbuf(&mut self) -> *mut MBuf {
        let mbuf = self.mbuf;
        self.mbuf = ptr::null_mut();
        mbuf
    }
}
