use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct DefinitionStep<GroupId, MemberCount, BoundedString> {
    pub name: BoundedString,
    pub group_id: Option<GroupId>,
    pub threshold: MemberCount,
}
