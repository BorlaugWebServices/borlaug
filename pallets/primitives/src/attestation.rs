use crate::did::Did;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Attestation<Timestamp> {
    /// Claim verifier attests a claim
    pub attested_by: Did,
    /// Attesttation valid until
    pub valid_until: Timestamp,
}
