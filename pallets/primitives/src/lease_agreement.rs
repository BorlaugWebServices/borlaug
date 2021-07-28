use crate::Did;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct LeaseAgreement<RegistryId, AssetId, Moment, BoundedString> {
    pub contract_number: BoundedString,
    pub lessor: Did,
    pub lessee: Did,
    pub effective_ts: Moment,
    pub expiry_ts: Moment,
    pub allocations: Vec<AssetAllocation<RegistryId, AssetId>>,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct AssetAllocation<RegistryId, AssetId> {
    pub registry_id: RegistryId,
    pub asset_id: AssetId,
    pub allocated_shares: u64,
}
