use super::Attribute;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{dispatch::Vec, scale_info::TypeInfo};
use sp_runtime::RuntimeDebug;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct ProcessStep<ProposalId, Moment, BoundedStringName, BoundedStringFact> {
    pub proposal_id: Option<ProposalId>,
    pub attested: Moment,
    pub attributes: Vec<Attribute<BoundedStringName, BoundedStringFact>>,
}
