use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Process {
    pub name: Vec<u8>,
    pub status: ProcessStatus,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum ProcessStatus {
    InProgress,
    Completed,
}

impl Default for ProcessStatus {
    fn default() -> Self {
        ProcessStatus::InProgress
    }
}
