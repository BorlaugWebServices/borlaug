use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Evidence<ProposalId, BoundedString> {
    pub proposal_id: ProposalId,
    pub name: BoundedString,
    pub content_type: BoundedString,
    pub url: Option<BoundedString>,
    pub hash: BoundedString,
}
