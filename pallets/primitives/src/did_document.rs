use crate::DidProperty;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, core::fmt::Debug)]
pub struct DidDocument<AccountId> {
    pub short_name: Option<Vec<u8>>,
    pub subject: AccountId,
    pub controllers: Vec<AccountId>,
    pub properties: Vec<DidProperty>,
}
