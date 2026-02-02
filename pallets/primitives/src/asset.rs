use alloc::vec::Vec;
use crate::AssetProperty;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use polkadot_sdk::polkadot_sdk_frame::prelude::*;
use scale_info::TypeInfo;

#[derive(
    Clone,
    Encode,
    Decode,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    DecodeWithMemTracking,
    MaxEncodedLen,
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
    Encode,
    Decode,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Clone,
    RuntimeDebug,
    MaxEncodedLen,
    TypeInfo,
    DecodeWithMemTracking,
)]
pub enum AssetStatus {
    Draft,
    Active,
    InActive,
}
