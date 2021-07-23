use super::Attribute;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct ProcessStep<BoundedStringName, BoundedStringFact> {
    pub attested: bool,
    pub attributes: Vec<Attribute<BoundedStringName, BoundedStringFact>>,
}
