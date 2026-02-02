use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use core::fmt::Debug;
use scale_info::TypeInfo;

#[derive(
    Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo,DecodeWithMemTracking
)]
pub struct DidDocument<AccountId> {
    pub subject: AccountId,
}
