/*
  Helper for inserting an SRH into a V6 Header and having it generate concrete
  types for a range of segment lengths.

  This ONLY operates within a v6 or extension header parsing step.

  Macro parameters include:

  o $segments - a vector of segments to insert as part of creating an SRH.
  o $pkt - the current packet being operated on within a network function.
  o $sr_info - SRH information, e.g. segments_left, tag, etc... that you want to
               set upon insertion/creation.
  o $prev_hdr_type - previous header type for SegmentRoutingHeader construction.
  o [$len => $utype] - number-to-type pairs to allow for static writes of
                       runtime numeric values, e.g. [1 => U1, 2 => U2]

  This returns an Option-Tuple containing the SRH offset after the swap and the
  new destination address (for a v6 header for example).

*/
#[macro_export]
macro_rules! srh_insert {
    ($segments:expr, $pkt:expr, $sr_info:expr, $prev_hdr_type:ty, [$($len:expr => $utype:ty),*]) => {
        match $segments.len() {
            $(
                $len => {
                    // TODO: Eventually maybe make this a hashmap, iterator (k,v), struct
                    let mut srh = <SegmentRoutingHeader<$prev_hdr_type, $utype>>::new_from_tuple(
                        $sr_info,
                        *GenericArray::<_, $utype>::from_slice(&$segments[..]),
                    );

                    Some($pkt.insert_header(NextHeader::Routing, &srh))
                }
            )*
                _ => {
                    None
                }
        };
    };
}

/*
  Helper for swapping a SRH with a new one and having it generate concrete
  types for a range of segment lengths.

  This ONLY operates within the SRH parsing step.

  Macro parameters include:

  o $segments - a vector of segments to insert as part of creating an SRH.
  o $pkt - the current packet being operated on within a network function.
  o $sr_info - SRH information, e.g. segments_left, tag, etc... that you want to
               set upon insertion/creation.
  o $prev_hdr_type - previous header type for SegmentRoutingHeader construction.
  o [$len => $utype] - number-to-type pairs to allow for static writes of
                       runtime numeric values, e.g. [1 => U1, 2 => U2]

  This returns an Option-Tuple containing the diff after the swap and
  the new destination address.
*/
#[macro_export]
macro_rules! srh_swap {
    ($segments:expr, $pkt:expr, $sr_info:expr, $prev_hdr_type:ty, [$($len:expr => $utype:ty),*]) => {
        match $segments.len() {
            $(
                $len => {
                    // TODO: Eventually maybe make this a hashmap, iterator (k,v), struct
                    let mut srh = <SegmentRoutingHeader<$prev_hdr_type, $utype>>::new_from_tuple(
                        $sr_info,
                        *GenericArray::<_, $utype>::from_slice(&$segments[..]),
                    );

                    if let Ok(swap_diff) =
                        $pkt.swap_header::<SegmentRoutingHeader<$prev_hdr_type, $utype>>(&srh) {
                            Some(swap_diff)
                        } else {
                            None
                        }
                }
            )*
                _ => {
                    None
                }
        };
    };
}
