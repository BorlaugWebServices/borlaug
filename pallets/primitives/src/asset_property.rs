use super::fact::Fact;
use scale_info::TypeInfo;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use polkadot_sdk::polkadot_sdk_frame::prelude::*;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,DecodeWithMemTracking
)]
pub struct AssetProperty<BoundedStringName, BoundedStringFact> {
    pub name: BoundedStringName,
    pub fact: Fact<BoundedStringFact>,
}
