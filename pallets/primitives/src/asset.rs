use crate::AssetProperty;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Asset<Timestamp, Balance> {
    pub properties: Option<Vec<AssetProperty>>,
    pub name: Option<Vec<u8>>,
    pub asset_number: Option<Vec<u8>>,
    pub status: Option<AssetStatus>,
    pub serial_number: Option<Vec<u8>>,
    pub total_shares: Option<u64>,
    pub residual_value: Option<Balance>,
    pub purchase_value: Option<Balance>,
    pub acquired_date: Option<Timestamp>,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum AssetStatus {
    Draft,
    Active,
    InActive,
}

impl Default for AssetStatus {
    fn default() -> Self {
        AssetStatus::Active
    }
}
