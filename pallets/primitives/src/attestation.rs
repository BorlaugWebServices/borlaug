use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Attestation<GroupId, Timestamp> {
    /// Claim verifier attests a claim
    pub attested_by: GroupId,
    /// Attesttation valid until
    pub valid_until: Timestamp,
}
