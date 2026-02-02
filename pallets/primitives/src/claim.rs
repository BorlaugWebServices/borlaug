use alloc::vec::Vec;
use crate::{Attestation, Fact};
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use polkadot_sdk::polkadot_sdk_frame::prelude::*;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,DecodeWithMemTracking
)]
pub struct Claim<AccountId, MemberCount, Moment, BoundedStringName, BoundedStringFact> {
    /// A claim description
    pub description: BoundedStringName,
    /// Statements contained in this claim
    pub statements: Vec<Statement<BoundedStringName, BoundedStringFact>>,
    /// Claim consumer creates a claim
    pub created_by: AccountId,
    /// Attestation by claim verifier
    pub attestation: Option<Attestation<AccountId, Moment>>,
    /// Minimum number of votes required for attestation
    pub threshold: MemberCount,
}

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, DecodeWithMemTracking,MaxEncodedLen, TypeInfo,
)]
pub struct Statement<BoundedStringName, BoundedStringFact> {
    /// Name of the property
    pub name: BoundedStringName,
    /// Fact in question
    pub fact: Fact<BoundedStringFact>,
    /// To be completed by verifier
    pub for_issuer: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, DecodeWithMemTracking, MaxEncodedLen, TypeInfo)]
pub struct ClaimConsumer<AccountId, Moment> {
    pub consumer: AccountId,
    /// Expiration time
    pub expiration: Moment,
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, DecodeWithMemTracking, MaxEncodedLen, TypeInfo)]
pub struct ClaimIssuer<AccountId, Moment> {
    pub issuer: AccountId,
    /// Expiration time
    pub expiration: Moment,
}
