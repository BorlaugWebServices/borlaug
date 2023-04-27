use codec::{Decode, Encode, MaxEncodedLen};
use core::fmt::Debug;
use frame_support::scale_info::TypeInfo;

#[derive(
    Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, Debug, MaxEncodedLen, TypeInfo,
)]
pub struct DidDocument<AccountId> {
    pub subject: AccountId,
}
