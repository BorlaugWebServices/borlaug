use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;
use sp_core::H256 as Hash;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Observation<ObservationId> {
    pub observation_id: Option<ObservationId>,
    pub compliance: Option<Vec<u8>>,
    pub procedural_note: Option<Vec<Hash>>,
}
