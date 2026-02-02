use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use polkadot_sdk::polkadot_sdk_frame::prelude::*;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,DecodeWithMemTracking
)]
pub struct DefinitionStep<AccountId, MemberCount, BoundedString> {
    pub name: BoundedString,
    pub attestor: AccountId,
    pub required: bool,
    pub threshold: MemberCount,
}
