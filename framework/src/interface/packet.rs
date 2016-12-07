use common::*;
use io::MBuf;
use std::ptr;
use std::marker::PhantomData;
use std::slice;
use headers::{EndOffset, NullHeader};
use std::mem::size_of;

#[link(name = "zcsi")]
extern "C" {
    fn mbuf_alloc() -> *mut MBuf;
    fn mbuf_free(buf: *mut MBuf);
    fn mbuf_alloc_bulk(array: *mut *mut MBuf, len: u16, cnt: i32) -> i32;
}


/// A packet is a safe wrapper around mbufs, that can be allocated and manipulated.
/// We associate a header type with a packet to allow safe insertion of headers.
#[cfg(not(feature = "packet_offset"))]
pub struct Packet<T: EndOffset, M: Sized + Send> {
    mbuf: *mut MBuf,
    _phantom_t: PhantomData<T>,
    _phantom_m: PhantomData<M>,
    header: *mut T,
    offset: usize,
}

#[inline]
#[cfg(not(feature = "packet_offset"))]
fn create_packet<T: EndOffset, M: Sized + Send>(mbuf: *mut MBuf, hdr: *mut T, offset: usize) -> Packet<T, M> {
    Packet::<T, M> {
        mbuf: mbuf,
        _phantom_t: PhantomData,
        _phantom_m: PhantomData,
        offset: offset,
        header: hdr,
    }
}

#[cfg(feature = "packet_offset")]
pub struct Packet<T: EndOffset, M: Sized + Send> {
    mbuf: *mut MBuf,
    _phantom_t: PhantomData<T>,
    _phantom_m: PhantomData<M>,
}

#[inline]
#[cfg(feature = "packet_offset")]
fn create_packet<T: EndOffset, M: Sized + Send>(mbuf: *mut MBuf, hdr: *mut T, offset: usiz) -> Packet<T, M> {
    let mut pkt = Packet::<T> {
        mbuf: mbuf,
        _phantom_t: PhantomData,
        _phantom_m: PhantomData,
    };
    pkt.update_ptrs(hdr as *mut u8, offset);
    pkt
}

fn reference_mbuf(mbuf: *mut MBuf) {
    unsafe { (*mbuf).reference() };
}

pub const METADATA_SLOTS: u16 = 16;
const HEADER_SLOT: usize = 0;
const OFFSET_SLOT: usize = HEADER_SLOT + 1;
const STACK_DEPTH_SLOT: usize = OFFSET_SLOT + 1;
const STACK_OFFSET_SLOT: usize = STACK_DEPTH_SLOT + 1;
const STACK_SIZE: usize = 0;
#[allow(dead_code)]
const END_OF_STACK_SLOT: usize = STACK_OFFSET_SLOT + STACK_SIZE;
const FREEFORM_METADATA_SLOT: usize = END_OF_STACK_SLOT;
const FREEFORM_METADATA_SIZE: usize = (METADATA_SLOTS as usize - FREEFORM_METADATA_SLOT) * 8;

#[inline]
pub unsafe fn packet_from_mbuf<T: EndOffset>(mbuf: *mut MBuf, offset: usize) -> Packet<T, EmptyMetadata> {
    // Need to up the refcnt, so that things don't drop.
    reference_mbuf(mbuf);
    packet_from_mbuf_no_increment(mbuf, offset)
}

#[inline]
pub unsafe fn packet_from_mbuf_no_increment<T: EndOffset>(mbuf: *mut MBuf, offset: usize) -> Packet<T, EmptyMetadata> {
    // Compute the real offset
    let header = (*mbuf).data_address(offset) as *mut T;
    create_packet(mbuf, header, offset)
}

#[inline]
pub unsafe fn packet_from_mbuf_no_free<T: EndOffset>(mbuf: *mut MBuf, offset: usize) -> Packet<T, EmptyMetadata> {
    packet_from_mbuf_no_increment(mbuf, offset)
}

/// Allocate a new packet.
pub fn new_packet() -> Option<Packet<NullHeader, EmptyMetadata>> {
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

/// Allocate an array of packets.
pub fn new_packet_array(count: usize) -> Vec<Packet<NullHeader, EmptyMetadata>> {
    let mut array = Vec::with_capacity(count);
    unsafe {
        let alloc_ret = mbuf_alloc_bulk(array.as_mut_ptr(), 0, count as i32);
        if alloc_ret == 0 {
            array.set_len(count);
        }
        array.iter().map(|m| packet_from_mbuf_no_increment(*m, 0)).collect()
    }
}


impl<T: EndOffset, M: Sized + Send> Packet<T, M> {
    // --------------------- Not using packet offsets ------------------------------------------------------
    #[inline]
    #[cfg(not(feature="packet_offset"))]
    fn header(&self) -> *mut T {
        self.header
    }

    #[inline]
    #[cfg(not(feature="packet_offset"))]
    fn header_u8(&self) -> *mut u8 {
        self.header as *mut u8
    }


    #[inline]
    #[cfg(not(feature="packet_offset"))]
    fn offset(&self) -> usize {
        self.offset
    }

    // ----------------- Using packet offsets -------------------------------------------------------------
    #[inline]
    #[cfg(feature="packet_offset")]
    fn header(&self) -> *mut T {
        self.read_header()
    }

    #[inline]
    #[cfg(feature="packet_offset")]
    fn header_u8(&self) -> *mut u8 {
        MBuf::read_metadata_slot(self.mbuf, HEADER_SLOT) as *mut u8
    }


    #[inline]
    #[cfg(feature="packet_offset")]
    fn offset(&self) -> usize {
        self.read_offset()
    }

    // -----------------Common code ------------------------------------------------------------------------
    #[inline]
    fn read_stack_depth(&self) -> usize {
        MBuf::read_metadata_slot(self.mbuf, STACK_DEPTH_SLOT)
    }

    #[inline]
    fn write_stack_depth(&mut self, new_depth: usize) {
        MBuf::write_metadata_slot(self.mbuf, STACK_DEPTH_SLOT, new_depth);
    }

    #[inline]
    fn read_stack_offset(&mut self, depth: usize) -> usize {
        MBuf::read_metadata_slot(self.mbuf, STACK_OFFSET_SLOT + depth)
    }

    #[inline]
    fn write_stack_offset(&mut self, depth: usize, offset: usize) {
        MBuf::write_metadata_slot(self.mbuf, STACK_OFFSET_SLOT + depth, offset)
    }

    #[inline]
    pub fn reset_stack_offset(&mut self) {
        self.write_stack_depth(0)
    }

    #[inline]
    #[cfg_attr(feature = "dev", allow(absurd_extreme_comparisons))]
    fn push_offset(&mut self, offset: usize) -> Option<usize> {
        let depth = self.read_stack_depth();
        if depth < STACK_SIZE {
            self.write_stack_depth(depth + 1);
            self.write_stack_offset(depth, offset);
            Some(depth + 1)
        } else {
            None
        }
    }

    #[inline]
    fn pop_offset(&mut self) -> Option<usize> {
        let depth = self.read_stack_depth();
        if depth > 0 {
            self.write_stack_depth(depth - 1);
            Some(self.read_stack_offset(depth - 1))
        } else {
            None
        }
    }

    #[inline]
    pub fn free_packet(self) {
        if !self.mbuf.is_null() {
            unsafe { mbuf_free(self.mbuf) };
        }
    }

    #[inline]
    fn update_ptrs(&mut self, header: *mut u8, offset: usize) {
        MBuf::write_metadata_slot(self.mbuf, HEADER_SLOT, header as usize);
        MBuf::write_metadata_slot(self.mbuf, OFFSET_SLOT, offset as usize);
    }

    /// Save the header and offset into the MBuf. This is useful for later restoring this information.
    #[inline]
    pub fn save_header_and_offset(&mut self) {
        let header = self.header_u8();
        let offset = self.offset();
        self.update_ptrs(header, offset)
    }

    #[inline]
    fn read_header<T2: EndOffset>(&self) -> *mut T2 {
        MBuf::read_metadata_slot(self.mbuf, HEADER_SLOT) as *mut T2
    }

    #[inline]
    fn read_offset(&self) -> usize {
        MBuf::read_metadata_slot(self.mbuf, OFFSET_SLOT)
    }

    #[inline]
    fn payload(&self) -> *mut u8 {
        unsafe {
            let payload_offset = self.payload_offset();
            self.header_u8().offset(payload_offset as isize)
        }
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

    #[inline]
    pub fn read_metadata(&self) -> &M {
        assert!(size_of::<M>() < FREEFORM_METADATA_SIZE);
        unsafe {
            let ptr = MBuf::metadata_as::<M>(self.mbuf, FREEFORM_METADATA_SLOT);
            &(*(ptr))
        }
    }

    #[inline]
    pub fn write_metadata<M2: Sized + Send>(&mut self, metadata: &M2) -> Result<()> {
        if size_of::<M2>() >= FREEFORM_METADATA_SIZE {
            Err(ErrorKind::MetadataTooLarge.into())
        } else {
            unsafe {
                let ptr = MBuf::mut_metadata_as::<M2>(self.mbuf, FREEFORM_METADATA_SLOT);
                ptr::copy_nonoverlapping(metadata, ptr, 1);
                Ok(())
            }
        }
    }

    #[inline]
    pub fn reinterpret_metadata<M2: Sized + Send>(mut self) -> Packet<T, M2> {
        let hdr = self.header();
        let offset = self.offset();
        unsafe { create_packet(self.get_mbuf_ref(), hdr, offset) }
    }

    /// When constructing a packet, take a packet as input and add a header.
    #[inline]
    pub fn push_header<T2: EndOffset<PreviousHeader = T>>(mut self, header: &T2) -> Option<Packet<T2, M>> {
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
                Some(create_packet(self.get_mbuf_ref(), hdr, offset))
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
                Err(ErrorKind::FailedAllocation.into())
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
                Err(ErrorKind::FailedAllocation.into())
            }
        }
    }

    #[inline]
    pub fn write_header<T2: EndOffset + Sized>(&mut self, header: &T2, offset: usize) -> Result<()> {
        if offset > self.payload_size() {
            Err(ErrorKind::BadOffset(offset).into())
        } else {
            unsafe {
                let dst = self.payload().offset(offset as isize);
                ptr::copy_nonoverlapping(header, dst as *mut T2, 1);
            }
            Ok(())
        }
    }

    #[inline]
    pub fn parse_header<T2: EndOffset<PreviousHeader = T>>(mut self) -> Packet<T2, M> {
        unsafe {
            assert!{self.payload_size() > T2::size()}
            let hdr = self.payload() as *mut T2;
            let offset = self.offset() + self.payload_offset();
            create_packet(self.get_mbuf_ref(), hdr, offset)
        }
    }

    #[inline]
    pub fn parse_header_and_record<T2: EndOffset<PreviousHeader = T>>(mut self) -> Packet<T2, M> {
        unsafe {
            assert!{self.payload_size() > T2::size()}
            let hdr = self.payload() as *mut T2;
            let payload_offset = self.payload_offset();
            let offset = self.offset() + payload_offset;
            // FIXME: Log failure?
            self.push_offset(payload_offset).unwrap();
            create_packet(self.get_mbuf_ref(), hdr, offset)
        }
    }

    #[inline]
    pub fn restore_saved_header<T2: EndOffset, M2: Sized + Send>(mut self) -> Option<Packet<T2, M2>> {
        unsafe {
            let hdr = self.read_header::<T2>();
            if hdr.is_null() {
                None
            } else {
                let offset = self.read_offset();
                Some(create_packet(self.get_mbuf_ref(), hdr, offset))
            }
        }
    }

    #[inline]
    pub fn replace_header(&mut self, hdr: &T) {
        unsafe {
            ptr::copy_nonoverlapping(hdr, self.header(), 1);
        }
    }

    #[inline]
    pub fn deparse_header(mut self, offset: usize) -> Packet<T::PreviousHeader, M> {
        let offset = offset as isize;
        unsafe {
            let header = self.header_u8().offset(-offset) as *mut T::PreviousHeader;
            let new_offset = self.offset() - offset as usize;
            create_packet(self.get_mbuf_ref(), header, new_offset)
        }
    }

    #[inline]
    pub fn deparse_header_stack(mut self) -> Option<Packet<T::PreviousHeader, M>> {
        self.pop_offset().map(|offset| self.deparse_header(offset))
    }


    #[inline]
    pub fn reset(mut self) -> Packet<NullHeader, EmptyMetadata> {
        unsafe {
            let header = self.data_base() as *mut NullHeader;
            create_packet(self.get_mbuf_ref(), header, 0)
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
    pub unsafe fn get_mbuf(mut self) -> *mut MBuf {
        self.get_mbuf_ref()
    }

    #[inline]
    unsafe fn get_mbuf_ref(&mut self) -> *mut MBuf {
        let mbuf = self.mbuf;
        self.mbuf = ptr::null_mut();
        mbuf
    }
}
