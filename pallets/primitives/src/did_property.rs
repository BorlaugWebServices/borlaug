use crate::Fact;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use polkadot_sdk::polkadot_sdk_frame::prelude::*;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug,DecodeWithMemTracking, MaxEncodedLen, TypeInfo,
)]
pub struct DidProperty<BoundedStringName, BoundedStringFact> {
    pub name: BoundedStringName,
    pub fact: Fact<BoundedStringFact>,
}
