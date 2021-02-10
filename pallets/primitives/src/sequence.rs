use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Sequence {
    pub name: Vec<u8>,
    pub status: SequenceStatus,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum SequenceStatus {
    InProgress,
    Completed,
}

impl Default for SequenceStatus {
    fn default() -> Self {
        SequenceStatus::InProgress
    }
}
