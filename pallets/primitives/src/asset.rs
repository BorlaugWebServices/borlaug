use crate::AssetProperty;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Asset<Timestamp, Balance, BoundedString> {
    pub properties: Option<Vec<AssetProperty<BoundedString>>>,
    pub name: Option<BoundedString>,
    pub asset_number: Option<BoundedString>,
    pub status: Option<AssetStatus>,
    pub serial_number: Option<BoundedString>,
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
