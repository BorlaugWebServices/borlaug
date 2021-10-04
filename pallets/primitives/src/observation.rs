use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Observation {
    pub compliance: Option<Compliance>,
    pub procedural_note_hash: Option<[u8; 32]>,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum Compliance {
    NotApplicable,
    Compliant,
    NonCompliant,
}
