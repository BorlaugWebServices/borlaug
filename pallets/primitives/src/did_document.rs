use codec::{Decode, Encode};

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, core::fmt::Debug)]
pub struct DidDocument<AccountId, GroupId, BoundedString> {
    pub short_name: Option<BoundedString>,
    pub subject: AccountId,
    pub controller: GroupId,
}
