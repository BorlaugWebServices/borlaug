use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use polkadot_sdk::polkadot_sdk_frame::prelude::*;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,DecodeWithMemTracking
)]
pub struct Attestation<AccountId, Moment> {
    /// Claim verifier attests a claim
    pub attested_by: AccountId,
    /// When attestation was issued
    pub issued: Moment,
    /// Attestation valid until
    pub valid_until: Moment,
}
