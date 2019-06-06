//! Implementations of `proptest.arbitrary.Arbitrary` trait for
//! various types.

use crate::packets::MacAddr;
use proptest::arbitrary::{any, Arbitrary, StrategyFor};
use proptest::strategy::{MapInto, Strategy};

impl Arbitrary for MacAddr {
    type Parameters = ();
    type Strategy = MapInto<StrategyFor<[u8; 6]>, Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        any::<[u8; 6]>().prop_map_into()
    }
}
