use crate::Fact;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct DidProperty<BoundedString> {
    pub name: BoundedString,
    pub fact: Fact<BoundedString>,
}
