use crate::AssetProperty;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Asset<Timestamp, Balance, BoundedStringName, BoundedStringFact> {
    pub properties: Vec<AssetProperty<BoundedStringName, BoundedStringFact>>,
    pub name: BoundedStringName,
    pub asset_number: Option<BoundedStringName>,
    pub status: AssetStatus,
    pub serial_number: Option<BoundedStringName>,
    pub total_shares: u64,
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
