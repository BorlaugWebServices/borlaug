use super::fact::Fact;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Attribute<BoundedString> {
    pub name: BoundedString,
    pub fact: Fact<BoundedString>,
}
