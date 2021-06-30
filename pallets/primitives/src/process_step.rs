use super::Attribute;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct ProcessStep {
    pub attested: bool,
    pub attributes: Vec<Attribute>,
}
