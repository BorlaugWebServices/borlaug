use super::{Attribute, Did};
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct ProcessStep {
    pub attested_by: Did,
    pub attributes: Vec<Attribute>,
}
