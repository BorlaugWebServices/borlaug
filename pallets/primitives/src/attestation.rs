use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Attestation<AccountId, Timestamp> {
    /// Claim verifier attests a claim
    pub attested_by: AccountId,
    /// When attestation was issued
    pub issued: Timestamp,
    /// Attesttation valid until
    pub valid_until: Timestamp,
}
