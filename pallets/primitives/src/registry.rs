use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Registry<BoundedString> {
    pub name: BoundedString,
}
