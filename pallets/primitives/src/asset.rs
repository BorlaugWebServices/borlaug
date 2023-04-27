use crate::AssetProperty;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{dispatch::Vec, scale_info::TypeInfo};
use sp_runtime::RuntimeDebug;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Asset<Moment, Balance, BoundedStringName, BoundedStringFact> {
    pub properties: Vec<AssetProperty<BoundedStringName, BoundedStringFact>>,
    pub name: BoundedStringName,
    pub asset_number: Option<BoundedStringName>,
    pub status: AssetStatus,
    pub serial_number: Option<BoundedStringName>,
    pub total_shares: u64,
    pub residual_value: Option<Balance>,
    pub purchase_value: Option<Balance>,
    pub acquired_date: Option<Moment>,
}

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub enum AssetStatus {
    Draft,
    Active,
    InActive,
}
