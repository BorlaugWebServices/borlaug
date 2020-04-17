use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Observation<ObservationId> {
    pub observation_id: Option<ObservationId>,
    pub compliance: Compliance,
    pub procedural_note: [u8; 32],
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum Compliance {
    NotApplicable,
    Compliant,
    NonCompliant,
}
