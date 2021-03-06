use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Observation {
    pub compliance: Option<Compliance>,
    pub procedural_note: Option<[u8; 32]>,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum Compliance {
    NotApplicable,
    Compliant,
    NonCompliant,
}

impl Default for Compliance {
    fn default() -> Self {
        Compliance::NotApplicable
    }
}
