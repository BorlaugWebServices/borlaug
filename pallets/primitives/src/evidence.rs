use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use polkadot_sdk::polkadot_sdk_frame::prelude::*;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,DecodeWithMemTracking
)]
pub struct Evidence<ProposalId, BoundedString, BoundedStringUrl> {
    pub proposal_id: ProposalId,
    pub name: BoundedString,
    pub content_type: BoundedString,
    pub url: Option<BoundedStringUrl>,
    pub hash: BoundedString,
}
