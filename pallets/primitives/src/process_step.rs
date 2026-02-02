use alloc::vec::Vec;
use super::Attribute;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use polkadot_sdk::polkadot_sdk_frame::prelude::*;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,DecodeWithMemTracking
)]
pub struct ProcessStep<ProposalId, Moment, BoundedStringName, BoundedStringFact> {
    pub proposal_id: Option<ProposalId>,
    pub attested: Moment,
    pub attributes: Vec<Attribute<BoundedStringName, BoundedStringFact>>,
}
