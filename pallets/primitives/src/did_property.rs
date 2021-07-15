use crate::Fact;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct DidProperty<BoundedStringName, BoundedStringFact> {
    pub name: BoundedStringName,
    pub fact: Fact<BoundedStringFact>,
}
