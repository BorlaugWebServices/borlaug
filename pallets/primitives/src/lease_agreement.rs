use crate::Did;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{dispatch::Vec, scale_info::TypeInfo};
use sp_runtime::RuntimeDebug;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct LeaseAgreement<ProposalId, RegistryId, AssetId, Moment, BoundedString> {
    pub proposal_id: Option<ProposalId>,
    pub contract_number: BoundedString,
    pub lessor: Did,
    pub lessee: Did,
    pub effective_ts: Moment,
    pub expiry_ts: Moment,
    pub allocations: Vec<AssetAllocation<RegistryId, AssetId>>,
}

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct AssetAllocation<RegistryId, AssetId> {
    pub registry_id: RegistryId,
    pub asset_id: AssetId,
    pub allocated_shares: u64,
}
