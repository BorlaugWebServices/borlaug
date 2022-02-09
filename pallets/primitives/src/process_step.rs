use super::Attribute;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct ProcessStep<ProposalId, Moment, BoundedStringName, BoundedStringFact> {
    pub proposal_id: Option<ProposalId>,
    pub attested: Moment,
    pub attributes: Vec<Attribute<BoundedStringName, BoundedStringFact>>,
}
