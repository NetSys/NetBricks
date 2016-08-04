use common::*;
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
#[cfg(not(feature = "packet_offset"))]
pub struct Packet<T: EndOffset> {
    mbuf: *mut MBuf,
    _phantom_t: PhantomData<T>,
    header: *mut T,
    offset: usize,
}

#[cfg(feature = "packet_offset")]
pub struct Packet<T: EndOffset> {
    mbuf: *mut MBuf,
    _phantom_t: PhantomData<T>,
}

fn reference_mbuf(mbuf: *mut MBuf) {
    unsafe { (*mbuf).reference() };
}

#[cfg(feature = "packet_offset")]
const HEADER_SLOT: usize = 0;
#[cfg(feature = "packet_offset")]
const OFFSET_SLOT: usize = 1;

#[inline]
#[cfg(feature = "packet_offset")]
fn create_packet<T: EndOffset>(mbuf: *mut MBuf, hdr: *mut T, offset: usize) -> Packet<T> {
    let mut pkt = Packet::<T> {
        mbuf: mbuf,
        _phantom_t: PhantomData,
    };
    pkt.update_ptrs(hdr as *mut u8, offset);
    pkt
}

#[inline]
#[cfg(not(feature = "packet_offset"))]
fn create_packet<T: EndOffset>(mbuf: *mut MBuf, hdr: *mut T, offset: usize) -> Packet<T> {
    Packet::<T> {
        mbuf: mbuf,
        _phantom_t: PhantomData,
        offset: offset,
        header: hdr,
    }
}

#[inline]
pub fn packet_from_mbuf<T: EndOffset>(mbuf: *mut MBuf, offset: usize) -> Packet<T> {
    // Need to up the refcnt, so that things don't drop.
    reference_mbuf(mbuf);
    packet_from_mbuf_no_increment(mbuf, offset)
}

#[inline]
pub fn packet_from_mbuf_no_increment<T: EndOffset>(mbuf: *mut MBuf, offset: usize) -> Packet<T> {
    unsafe {
        // Compute the real offset
        let header = (*mbuf).data_address(offset) as *mut T;
        create_packet(mbuf, header, offset)
    }
}

#[inline]
pub unsafe fn packet_from_mbuf_no_free<T: EndOffset>(mbuf: *mut MBuf, offset: usize) -> Packet<T> {
    packet_from_mbuf_no_increment(mbuf, offset)
}

/// Allocate a new packet.
pub fn new_packet() -> Option<Packet<NullHeader>> {
    unsafe {
        // This sets refcnt = 1
        let mbuf = mbuf_alloc();
        if mbuf.is_null() {
            None
        } else {
            Some(packet_from_mbuf_no_increment(mbuf, 0))
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
    array.iter().map(|m| packet_from_mbuf_no_increment(m.clone(), 0)).collect()
}

pub const METADATA_SLOTS: u16 = 8;

impl<T: EndOffset> Packet<T> {
    #[inline]
    pub fn free_packet(self) {
        if !self.mbuf.is_null() {
            unsafe { mbuf_free(self.mbuf) };
        }
    }

    #[inline]
    #[cfg(feature="packet_offset")]
    fn update_ptrs(&mut self, header: *mut u8, offset: usize) {
        if false {
            println!("packet {:x} update_ptrs header {:x} offset {:x}", self.mbuf as usize, header as usize, offset as
                     usize);
        }
        MBuf::write_metadata_slot(self.mbuf, HEADER_SLOT, header as usize);
        MBuf::write_metadata_slot(self.mbuf, OFFSET_SLOT, offset as usize);
    }

    #[inline]
    #[cfg(feature="packet_offset")]
    fn header(&self) -> *mut T {
        MBuf::read_metadata_slot(self.mbuf, HEADER_SLOT) as *mut T
    }

    #[inline]
    #[cfg(not(feature="packet_offset"))]
    fn header(&self) -> *mut T {
        // MBuf::read_metadata_slot(self.mbuf, HEADER_SLOT) as *mut T
        self.header
    }

    #[inline]
    #[cfg(feature="packet_offset")]
    fn header_u8(&self) -> *mut u8 {
        MBuf::read_metadata_slot(self.mbuf, HEADER_SLOT) as *mut u8
    }

    #[inline]
    #[cfg(not(feature="packet_offset"))]
    fn header_u8(&self) -> *mut u8 {
        self.header as *mut u8
    }

    #[inline]
    fn payload(&self) -> *mut u8 {
        unsafe {
            let payload_offset = self.payload_offset();
            self.header_u8().offset(payload_offset as isize)
        }
    }

    #[inline]
    #[cfg(feature="packet_offset")]
    fn offset(&self) -> usize {
        MBuf::read_metadata_slot(self.mbuf, OFFSET_SLOT)
    }

    #[inline]
    #[cfg(not(feature="packet_offset"))]
    fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    fn payload_offset(&self) -> usize {
        unsafe { (*self.header()).offset() }
    }

    #[inline]
    fn data_base(&self) -> *mut u8 {
        unsafe { (*self.mbuf).data_address(0) }
    }

    #[inline]
    fn data_len(&self) -> usize {
        unsafe { (*self.mbuf).data_len() }
    }

    #[inline]
    fn payload_size(&self) -> usize {
        self.data_len() - self.offset()
    }

    #[inline]
    pub fn get_header(&self) -> &T {
        unsafe { &(*(self.header())) }
    }

    #[inline]
    pub fn get_mut_header(&mut self) -> &mut T {
        unsafe { &mut (*(self.header())) }
    }

    /// When constructing a packet, take a packet as input and add a header.
    #[inline]
    pub fn push_header<T2: EndOffset<PreviousHeader = T>>(mut self, header: &T2) -> Option<Packet<T2>> {
        unsafe {
            let len = self.data_len();
            let size = header.offset();
            let added = (*self.mbuf).add_data_end(size);

            let header = self.payload();

            let hdr = header as *mut T2;
            let offset = self.offset() + self.payload_offset();
            if added >= size {
                let dst = if len != offset {
                    // Need to move down the rest of the data down.
                    let final_dst = self.payload();
                    let move_loc = final_dst.offset(size as isize);
                    let to_move = len - offset;
                    ptr::copy_nonoverlapping(final_dst, move_loc, to_move);
                    final_dst as *mut T2
                } else {
                    self.payload() as *mut T2
                };
                ptr::copy_nonoverlapping(hdr, dst, 1);
                Some(create_packet(self.get_mbuf(), hdr, offset))
            } else {
                None
            }
        }
    }

    /// Remove data at the top of the payload, useful when removing headers.
    #[inline]
    pub fn remove_from_payload_head(&mut self, size: usize) -> Result<()> {
        unsafe {
            let src = self.data_base();
            let dst = src.offset(size as isize);
            ptr::copy_nonoverlapping(src, dst, size);
            (*self.mbuf).remove_data_beginning(size);
            Ok(())
        }
    }

    /// Add data to the head of the payload.
    #[inline]
    pub fn add_to_payload_head(&mut self, size: usize) -> Result<()> {
        unsafe {
            let added = (*self.mbuf).add_data_end(size);
            if added >= size {
                let src = self.payload();
                let dst = src.offset(size as isize);
                ptr::copy_nonoverlapping(src, dst, size);
                Ok(())
            } else {
                Err(ZCSIError::FailedAllocation)
            }
        }
    }

    #[inline]
    pub fn remove_from_payload_tail(&mut self, size: usize) -> Result<()> {
        unsafe {
            (*self.mbuf).remove_data_end(size);
            Ok(())
        }
    }

    #[inline]
    pub fn add_to_payload_tail(&mut self, size: usize) -> Result<()> {
        unsafe {
            let added = (*self.mbuf).add_data_end(size);
            if added >= size {
                Ok(())
            } else {
                Err(ZCSIError::FailedAllocation)
            }
        }
    }

    #[inline]
    pub fn write_header<T2: EndOffset + Sized>(&mut self, header: &T2, offset: usize) -> Result<()> {
        if offset > self.payload_size() {
            Err(ZCSIError::BadOffset)
        } else {
            unsafe {
                let dst = self.payload().offset(offset as isize);
                ptr::copy_nonoverlapping(header, dst as *mut T2, 1);
            }
            Ok(())
        }
    }

    #[inline]
    pub fn parse_header<T2: EndOffset<PreviousHeader = T>>(mut self) -> Packet<T2> {
        unsafe {
            let hdr = self.payload() as *mut T2;
            let offset = self.offset() + self.payload_offset();
            create_packet(self.get_mbuf(), hdr, offset)
        }
    }

    #[inline]
    pub fn replace_header(&mut self, hdr: &T) {
        unsafe {
            ptr::copy_nonoverlapping(hdr, self.header(), 1);
        }
    }

    #[inline]
    pub fn deparse_header(mut self, offset: usize) -> Packet<T::PreviousHeader> {
        let offset = offset as isize;
        unsafe {
            let header = self.header_u8().offset(-offset) as *mut T::PreviousHeader;
            let new_offset = self.offset() - offset as usize;
            create_packet(self.get_mbuf(), header, new_offset)
        }
    }


    #[inline]
    pub fn reset(mut self) -> Packet<NullHeader> {
        unsafe {
            let mbuf = self.get_mbuf();
            let header = self.data_base() as *mut NullHeader;
            create_packet(mbuf, header, 0)
        }
    }

    #[inline]
    pub fn get_mut_payload(&mut self) -> &mut [u8] {
        unsafe {
            let len = self.payload_size();
            let ptr = self.payload();
            slice::from_raw_parts_mut(ptr, len)
        }
    }

    #[inline]
    pub fn get_payload(&self) -> &[u8] {
        unsafe {
            let len = self.payload_size();
            slice::from_raw_parts(self.payload(), len)
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
        let dst = self.payload();
        let src = other.payload();

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
