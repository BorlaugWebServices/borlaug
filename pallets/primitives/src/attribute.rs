use super::fact::Fact;
use codec::{Decode, Encode,DecodeWithMemTracking, MaxEncodedLen};
use scale_info::TypeInfo;
use polkadot_sdk::polkadot_sdk_frame::prelude::*;

#[derive(
    Encode,
    Decode,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Clone,
    DecodeWithMemTracking,
    RuntimeDebug,
    MaxEncodedLen,
    TypeInfo,
)]
pub struct Attribute<BoundedStringName, BoundedStringFact> {
    pub name: BoundedStringName,
    pub fact: Fact<BoundedStringFact>,
}
