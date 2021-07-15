use crate::DidProperty;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, core::fmt::Debug)]
pub struct DidDocument<AccountId, GroupId, BoundedString> {
    pub short_name: Option<BoundedString>,
    pub subject: AccountId,
    pub controller: GroupId,
    pub properties: Vec<DidProperty<BoundedString>>,
}
