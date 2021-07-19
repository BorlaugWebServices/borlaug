use crate::{Attestation, Fact};

use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Claim<AccountId, Moment, BoundedStringName, BoundedStringFact> {
    /// A claim description
    pub description: BoundedStringName,
    /// Statements contained in this claim
    pub statements: Vec<Statement<BoundedStringName, BoundedStringFact>>,
    /// Claim consumer creates a claim
    pub created_by: AccountId,
    /// Attesttation by claim verifier
    pub attestation: Option<Attestation<AccountId, Moment>>,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Statement<BoundedStringName, BoundedStringFact> {
    /// Name of the property
    pub name: BoundedStringName,
    /// Fact in question
    pub fact: Fact<BoundedStringFact>,
    /// To be completed by verifier
    pub for_issuer: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct ClaimConsumer<AccountId, Moment> {
    pub consumer: AccountId,
    /// Expiration time
    pub expiration: Moment,
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct ClaimIssuer<AccountId, Moment> {
    pub issuer: AccountId,
    /// Expiration time
    pub expiration: Moment,
}
