use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Attestation<AccountId, Moment> {
    /// Claim verifier attests a claim
    pub attested_by: AccountId,
    /// When attestation was issued
    pub issued: Moment,
    /// Attesttation valid until
    pub valid_until: Moment,
}
