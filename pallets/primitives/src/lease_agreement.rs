use crate::Did;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct LeaseAgreement<RegistryId, AssetId, Timestamp> {
    pub contract_number: Vec<u8>,
    pub lessor: Did,
    pub lessee: Did,
    pub effective_ts: Timestamp,
    pub expiry_ts: Timestamp,
    pub allocations: Vec<AssetAllocation<RegistryId, AssetId>>,
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct AssetAllocation<RegistryId, AssetId> {
    pub registry_id: RegistryId,
    pub asset_id: AssetId,
    pub allocated_shares: u64,
}
