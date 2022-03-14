use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct DefinitionStep<AccountId, MemberCount, BoundedString> {
    pub name: BoundedString,
    pub attestor: AccountId,
    pub required: bool,
    pub threshold: MemberCount,
}
