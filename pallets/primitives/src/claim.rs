use crate::{Attestation, Did, Fact};

use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Claim<Timestamp> {
    /// A claim description
    pub description: Vec<u8>,
    /// Statements contained in this claim
    pub statements: Vec<Statement>,
    /// Claim consumer creates a claim
    pub created_by: Did,
    /// Attesttation by claim verifier
    pub attestation: Option<Attestation<Timestamp>>,
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Statement {
    /// Name of the property
    pub name: Vec<u8>,
    /// Fact in question
    pub fact: Fact,
    /// To be completed by verifier
    pub for_issuer: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct ClaimConsumer<Moment> {
    /// Did of claim issuer
    pub consumer: Did,
    /// Expiration time
    pub expiration: Moment,
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct ClaimIssuer<Moment> {
    /// Did of claim issuer
    pub issuer: Did,
    /// Expiration time
    pub expiration: Moment,
}
