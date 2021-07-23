use codec::{Decode, Encode};
use core::fmt::Debug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, Debug)]
pub struct DidDocument<AccountId, BoundedString> {
    pub short_name: Option<BoundedString>,
    pub subject: AccountId,
}
