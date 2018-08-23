use common::*;
use headers::ip::v6::Ipv6VarHeader;
use headers::{EndOffset, HeaderUpdates, NextHeader, NullHeader};
use native::zcsi::*;
use std::cmp::{self, Ordering};
use std::fmt::Display;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr;
use std::slice;

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
fn create_packet<T: EndOffset, M: Sized + Send>(
    mbuf: *mut MBuf,
    hdr: *mut T,
    offset: usize,
) -> Packet<T, M> {
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
fn create_packet<T: EndOffset, M: Sized + Send>(
    mbuf: *mut MBuf,
    hdr: *mut T,
    offset: usiz,
) -> Packet<T, M> {
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

#[cfg(feature = "packet_offset")]
impl<T, M> Clone for Packet<T, M>
where
    T: EndOffset,
    M: Sized + Send,
{
    fn clone(&self) -> Self {
        Packet {
            mbuf: self.mbuf.clone(),
            _phantom_t: PhantomData,
            _phantom_m: PhantomData,
        }
    }
}

#[cfg(not(feature = "packet_offset"))]
impl<T, M> Clone for Packet<T, M>
where
    T: EndOffset,
    M: Sized + Send,
{
    fn clone(&self) -> Self {
        Packet {
            mbuf: self.mbuf.clone(),
            _phantom_t: PhantomData,
            _phantom_m: PhantomData,
            offset: self.offset.clone(),
            header: self.header.clone(),
        }
    }
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
pub unsafe fn packet_from_mbuf<T: EndOffset>(
    mbuf: *mut MBuf,
    offset: usize,
) -> Packet<T, EmptyMetadata> {
    // Need to up the refcnt, so that things don't drop.
    reference_mbuf(mbuf);
    packet_from_mbuf_no_increment(mbuf, offset)
}

#[inline]
pub unsafe fn packet_from_mbuf_no_increment<T: EndOffset>(
    mbuf: *mut MBuf,
    offset: usize,
) -> Packet<T, EmptyMetadata> {
    // Compute the real offset
    let header = (*mbuf).data_address(offset) as *mut T;
    create_packet(mbuf, header, offset)
}

#[inline]
pub unsafe fn packet_from_mbuf_no_free<T: EndOffset>(
    mbuf: *mut MBuf,
    offset: usize,
) -> Packet<T, EmptyMetadata> {
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
        array
            .iter()
            .map(|m| packet_from_mbuf_no_increment(*m, 0))
            .collect()
    }
}

impl<T: EndOffset, M: Sized + Send> Packet<T, M> {
    // --------------------- Not using packet offsets ------------------------------------------------------
    #[inline]
    #[cfg(not(feature = "packet_offset"))]
    fn header(&self) -> *mut T {
        self.header
    }

    #[inline]
    #[cfg(not(feature = "packet_offset"))]
    fn header_u8(&self) -> *mut u8 {
        self.header as *mut u8
    }

    #[inline]
    #[cfg(not(feature = "packet_offset"))]
    fn offset(&self) -> usize {
        self.offset
    }

    // ----------------- Using packet offsets -------------------------------------------------------------
    #[inline]
    #[cfg(feature = "packet_offset")]
    fn header(&self) -> *mut T {
        self.read_header()
    }

    #[inline]
    #[cfg(feature = "packet_offset")]
    fn header_u8(&self) -> *mut u8 {
        MBuf::read_metadata_slot(self.mbuf, HEADER_SLOT) as *mut u8
    }

    #[inline]
    #[cfg(feature = "packet_offset")]
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

    /// Return the offset of the payload relative to the header.
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
        self.data_len() - self.offset() - self.payload_offset()
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

    #[inline]
    pub fn emit_metadata<M2: Sized + Send>(&self) -> &M2 {
        unsafe {
            let ptr = MBuf::metadata_as::<M2>(self.mbuf, FREEFORM_METADATA_SLOT);
            &(*(ptr))
        }
    }

    /*
    When constructing a packet, take a packet as input and add a header.
    Look at *fn insert_header* for a longer explanation. Use *push_header*
    to create a new packet with a sequence of headers upon first creation, as
    it returns an Option around the newly created packet itself.
     */
    #[inline]
    pub fn push_header<T2: EndOffset<PreviousHeader = T>>(
        mut self,
        header: &T2,
    ) -> Option<Packet<T2, M>> {
        unsafe {
            let packet_len = self.data_len();
            let header_size = header.offset();
            let added = (*self.mbuf).add_data_end(header_size);

            let hdr = header as *const T2;
            let offset = self.offset() + self.payload_offset();
            if added >= header_size {
                let fin_dst = self.payload();
                if packet_len != offset {
                    // Need to move down the rest of the data down.
                    let move_loc = fin_dst.offset(header_size as isize);
                    let to_move = packet_len - offset;
                    ptr::copy_nonoverlapping(fin_dst, move_loc, to_move);
                }
                let dst = fin_dst as *mut T2;
                ptr::copy_nonoverlapping(hdr, dst, 1);
                Some(create_packet(self.get_mbuf_ref(), dst, offset))
            } else {
                None
            }
        }
    }

    /*
    By Dan Jin: The entire network packet is stored in a mbuf by DPDK. A
    NetBricks packet is a wrapper with an offset that tells you where a
    particular header starts in the mbuf.

    If you have a Packet<TcpHeader>, then *self.offset() is where the first
    byte of the tcp header is*. self.payload_offset() is actually just
    header.size()---NAMES ARE HORRIBLE. In this example, self.offset() +
    self.payload_offset() give you where the tcp payload would start.

    Use *insert_a_header* on a mutable reference to insert a header into an
    already existing packet, after another header's offset basically. It
    returns an isize Result type, i.e. Err(e) or Ok(diff) upon inserting a header int
     */
    #[inline]
    fn insert_a_header<T2: EndOffset<PreviousHeader = T>>(&mut self, header: &T2) -> Result<isize> {
        unsafe {
            let packet_len = self.data_len();
            let header_size = header.offset();
            let added = (*self.mbuf).add_data_end(header_size);

            let hdr = header as *const T2;
            let offset = self.offset() + self.payload_offset();
            if added >= header_size {
                let fin_dst = self.payload();
                if packet_len != offset {
                    // Need to move down the rest of the data down.
                    let move_loc = fin_dst.offset(header_size as isize);
                    let to_move = packet_len - offset;
                    ptr::copy_nonoverlapping(fin_dst, move_loc, to_move);
                }
                ptr::copy_nonoverlapping(hdr, fin_dst as *mut T2, 1);
                Ok(header_size as isize)
            } else {
                Err(ErrorKind::FailedToInsertHeader.into())
            }
        }
    }

    /*
    Use *insert_header_fn* on a mutable reference to insert a header into an
    already existing packet, after another header's offset basically. It
    returns a Result type, i.e. Err(e) or Ok(()) upon inserting a header.

    This variant allows for passing in a higher-order function to make
    mutable updates to the *current* header after insertion.

    Works on V6 and V6 Extension Headers only via trait bounds.
    */
    #[inline]
    pub fn insert_header_fn<T2: EndOffset<PreviousHeader = T>>(
        &mut self,
        header_type: NextHeader,
        header: &T2,
        on_insert: &Fn(&mut T),
    ) -> Result<()>
    where
        T: HeaderUpdates,
        T2: Ipv6VarHeader,
    {
        unsafe {
            if let Ok(diff) = self.insert_a_header::<T2>(header) {
                let mut current_header: &mut T = &mut *self.header();
                current_header.update_payload_len(diff);
                current_header.update_next_header(header_type);
                on_insert(current_header);
                Ok(())
            } else {
                Err(ErrorKind::FailedToInsertHeader.into())
            }
        }
    }

    /// @see insert_header_fn
    pub fn insert_header<T2: EndOffset<PreviousHeader = T>>(
        &mut self,
        header_type: NextHeader,
        header: &T2,
    ) -> Result<()>
    where
        T: HeaderUpdates,
        T2: Ipv6VarHeader,
    {
        self.insert_header_fn::<T2>(header_type, header, &|_| ())
    }

    /// Remove the next header from the currently parsed packet
    #[inline]
    fn remove_a_header<T2: EndOffset<PreviousHeader = T>>(&mut self) -> Result<isize>
    where
        T2: Ipv6VarHeader,
    {
        unsafe {
            let packet_len = self.data_len(); // length of packet
            let var_header = self.payload() as *mut T2; // next_var_type_hdr ptr
            let var_header_size = (*var_header).offset(); // size of next_var_type_hdr

            // current_hdr ends and next_var_type_hdr starts in bytes
            let var_header_offset = self.offset() + self.payload_offset();

            if packet_len >= var_header_offset {
                // Need to move up the data up and then remove the ext/var header
                let src_loc = self.payload().offset(var_header_size as isize);
                let dst_loc = var_header;
                let to_move = packet_len - (var_header_offset + var_header_size);
                ptr::copy_nonoverlapping(src_loc as *const T2, dst_loc, to_move);
                let removed = (*self.mbuf).remove_data_end(var_header_size);

                if removed != var_header_size {
                    Err(ErrorKind::FailedToRemoveHeader.into())
                } else {
                    Ok(-(var_header_size as isize))
                }
            } else {
                Err(ErrorKind::FailedToRemoveHeader.into())
            }
        }
    }

    /*
    Use *remove_header_fn* on a mutable reference to remove a (following) header
    from an already existing packet, after another header's offset basically. It
    returns a Result type, i.e. Err(e) or Ok(()) upon removing a header.

    This variant allows for passing in a higher-order function to make
    mutable updates to the *current* header after removal.

    Works on V6 and V6 Extension Headers only via trait bounds.
     */
    #[inline]
    pub fn remove_header_fn<T2: EndOffset<PreviousHeader = T>>(
        &mut self,
        on_remove: &Fn(&mut T),
    ) -> Result<()>
    where
        T: HeaderUpdates,
        T2: Ipv6VarHeader,
    {
        unsafe {
            let var_header: Option<&T2> = {
                let payload = self.payload() as *mut T2;
                if self.data_len() > (*payload).offset() {
                    Some(&*payload)
                } else {
                    None
                }
            };

            match var_header.and_then(|hdr| Some((hdr.next_header(), self.remove_a_header::<T2>())))
            {
                Some((Some(next_header), Ok(diff))) => {
                    let mut current_header: &mut T = &mut *self.header();
                    current_header.update_payload_len(diff);
                    current_header.update_next_header(next_header);
                    on_remove(current_header);
                    Ok(())
                }
                _ => Err(ErrorKind::FailedToRemoveHeader.into()),
            }
        }
    }

    /// @see remove_header_fn
    #[inline]
    pub fn remove_header<T2: EndOffset<PreviousHeader = T>>(&mut self) -> Result<()>
    where
        T: HeaderUpdates,
        T2: Ipv6VarHeader,
    {
        self.remove_header_fn::<T2>(&|_| ())
    }

    /*
    Swap out old header of one type with a new header of the same type.
    Returns the diff in bytes as a signed integer for calculation needs, e.g.
    payload length for a v6 header, etc...
    */
    #[inline]
    pub fn swap_header<T2>(&mut self, new_header: &T2) -> Result<isize>
    where
        T2: EndOffset<PreviousHeader = T::PreviousHeader> + Display,
    {
        unsafe {
            let packet_len = self.data_len();
            let current_hdr = self.header();

            let current_hdr_size = (*current_hdr).offset();
            let new_hdr_size = new_header.offset();
            let payload = self.payload();
            let offset = self.offset();

            let to_move = packet_len - offset - current_hdr_size;

            match new_hdr_size.cmp(&current_hdr_size) {
                Ordering::Less => {
                    let diff: usize = current_hdr_size - new_hdr_size;
                    let move_loc = payload.offset(diff as isize);
                    ptr::copy(payload, move_loc, to_move);

                    let removed = (*self.mbuf).remove_data_end(diff);
                    if removed <= new_hdr_size && diff != removed {
                        return Err(ErrorKind::FailedToSwapHeader(format!("{}", new_header)).into());
                    }
                }
                Ordering::Greater => {
                    let diff: usize = new_hdr_size - current_hdr_size;
                    let move_loc = payload.offset(diff as isize);

                    let added = (*self.mbuf).add_data_end(diff);
                    if added >= new_hdr_size && diff != added {
                        return Err(ErrorKind::FailedToSwapHeader(format!("{}", new_header)).into());
                    }
                    ptr::copy(payload, move_loc, to_move);
                }
                Ordering::Equal => (),
            }

            ptr::copy_nonoverlapping(new_header as *const T2, self.header_u8() as *mut T2, 1);
            Ok(new_hdr_size as isize - current_hdr_size as isize)
        }
    }

    #[inline]
    pub fn swap_header_fn<T2>(&mut self, new_header: &T2, on_swap: &Fn(&mut T)) -> Result<isize>
    where
        T2: EndOffset<PreviousHeader = T::PreviousHeader> + Display,
    {
        unsafe {
            if let Ok(diff) = self.swap_header::<T2>(new_header) {
                let current_header: &mut T = &mut *self.header();
                on_swap(current_header);
                Ok(diff)
            } else {
                Err(ErrorKind::FailedToSwapHeader(format!("{}", new_header)).into())
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
    pub fn write_header<T2: EndOffset + Sized>(
        &mut self,
        header: &T2,
        offset: usize,
    ) -> Result<()> {
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
            assert!{self.payload_size() >= T2::size()}
            let hdr = self.payload() as *mut T2;
            let offset = self.offset() + self.payload_offset();
            create_packet(self.get_mbuf_ref(), hdr, offset)
        }
    }

    // TODO: Fix This.
    #[inline]
    pub fn parse_header_and_record<T2: EndOffset<PreviousHeader = T>>(mut self) -> Packet<T2, M> {
        unsafe {
            assert!{self.payload_size() >= T2::size()}
            let hdr = self.payload() as *mut T2;
            let payload_offset = self.payload_offset();
            let offset = self.offset() + payload_offset;
            // FIXME: Log failure?
            self.push_offset(payload_offset).unwrap();
            create_packet(self.get_mbuf_ref(), hdr, offset)
        }
    }

    #[inline]
    pub fn restore_saved_header<T2: EndOffset, M2: Sized + Send>(
        mut self,
    ) -> Option<Packet<T2, M2>> {
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

    /// Calculate current header + data calculations like for tcp/udp
    /// checksum.
    #[inline]
    pub fn segment_length(&self) -> u32 {
        unsafe {
            let current_hdr = self.header();
            let current_hdr_size = (*current_hdr).offset();
            let payload = self.payload_size();
            current_hdr_size as u32 + payload as u32 + self.sum_be_words(8)
        }
    }

    /// Sum all words (16 bit chunks) from a packet's data (). The word at word offset
    /// `skipword` will be skipped. Each word is treated as big endian.
    /// Taken from and inspired by:
    /// https://github.com/libpnet/libpnet/commit/db6c2fc0c4c96ec6583b3652655f3648f4aa2dd0#diff-0e88f9c4bcda8daf66293db8e37dda32R145.
    #[inline]
    fn sum_be_words(&self, mut skipword: usize) -> u32 {
        let data: &[u8] = self.get_data();
        let len = data.len();

        let wdata: &[u16] = unsafe { slice::from_raw_parts(data.as_ptr() as *const u16, len / 2) };
        skipword = cmp::min(skipword, wdata.len());

        let mut sum = 0u32;
        let mut i = 0;

        while i < skipword {
            sum += u16::from_be(unsafe { *wdata.get_unchecked(i) }) as u32;
            i += 1;
        }

        i += 1;
        while i < wdata.len() {
            sum += u16::from_be(unsafe { *wdata.get_unchecked(i) }) as u32;
            i += 1;
        }

        // If the length is odd, make sure to checksum the final byte
        if len & 1 != 0 {
            sum += (unsafe { *data.get_unchecked(len - 1) } as u32) << 8;
        }

        sum
    }

    /// Calculate the data from the current header offset all the way to the
    /// payload. Used for tcp/udp packets.
    #[inline]
    fn get_data(&self) -> &mut [u8] {
        unsafe {
            let current_hdr_size = (*self.header()).offset();
            let ptr = self.header_u8();
            slice::from_raw_parts_mut(ptr, current_hdr_size + self.payload_size())
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
            payload_size + self.increase_payload_size(increment)
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
