use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Observation<ProposalId> {
    /// The proposal that created the observation.
    pub proposal_id: ProposalId,
    pub compliance: Option<Compliance>,
    pub procedural_note_hash: Option<[u8; 32]>,
}

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub enum Compliance {
    NotApplicable,
    Compliant,
    NonCompliant,
}
