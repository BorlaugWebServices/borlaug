use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use polkadot_sdk::polkadot_sdk_frame::prelude::*;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,DecodeWithMemTracking
)]
pub struct Observation<ProposalId> {
    /// The proposal that created the observation.
    pub proposal_id: ProposalId,
    pub compliance: Option<Compliance>,
    pub procedural_note_hash: Option<[u8; 32]>,
}

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug,DecodeWithMemTracking, MaxEncodedLen, TypeInfo,
)]
pub enum Compliance {
    NotApplicable,
    Compliant,
    NonCompliant,
}
