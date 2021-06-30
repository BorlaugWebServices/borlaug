use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct DefinitionStep<GroupId, MemberCount> {
    pub name: Vec<u8>,
    pub group_id: GroupId,
    pub threshold: MemberCount,
}
