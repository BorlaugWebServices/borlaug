use crate::{Attestation, Fact};

use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Claim<GroupId, Timestamp, BoundedString> {
    /// A claim description
    pub description: BoundedString,
    /// Statements contained in this claim
    pub statements: Vec<Statement<BoundedString>>,
    /// Claim consumer creates a claim
    pub created_by: GroupId,
    /// Attesttation by claim verifier
    pub attestation: Option<Attestation<GroupId, Timestamp>>,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Statement<BoundedString> {
    /// Name of the property
    pub name: BoundedString,
    /// Fact in question
    pub fact: Fact<BoundedString>,
    /// To be completed by verifier
    pub for_issuer: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct ClaimConsumer<GroupId, Moment> {
    pub group_id: GroupId,
    /// Expiration time
    pub expiration: Moment,
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct ClaimIssuer<GroupId, Moment> {
    pub group_id: GroupId,
    /// Expiration time
    pub expiration: Moment,
}
