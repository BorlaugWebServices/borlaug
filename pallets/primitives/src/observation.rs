use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_core::H256 as Hash;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Observation<ObservationId> {
    pub observation_id: Option<ObservationId>,
    pub compliance: Compliance,
    pub procedural_note: Vec<Hash>,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum Compliance {
    NotApplicable,
    Compliant,
    NonCompliant,
}
