use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Evidence<ProposalId, BoundedString, BoundedStringUrl> {
    pub proposal_id: ProposalId,
    pub name: BoundedString,
    pub content_type: BoundedString,
    pub url: Option<BoundedStringUrl>,
    pub hash: BoundedString,
}
