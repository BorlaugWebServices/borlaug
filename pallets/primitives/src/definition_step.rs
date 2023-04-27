use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct DefinitionStep<AccountId, MemberCount, BoundedString> {
    pub name: BoundedString,
    pub attestor: AccountId,
    pub required: bool,
    pub threshold: MemberCount,
}
