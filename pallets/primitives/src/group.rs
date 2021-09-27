use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Group<GroupId, AccountId, MemberCount, BoundedString> {
    pub name: BoundedString,
    pub members: Vec<AccountId>,
    pub threshold: MemberCount,
    pub anonymous_account: AccountId,
    pub parent: Option<GroupId>,
}
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
/// Info for keeping track of a motion being voted on.
pub struct Votes<AccountId, MemberCount> {
    /// The number of approval votes that are needed to pass the motion.
    pub threshold: MemberCount,
    /// The current set of voters that approved it.
    pub ayes: Vec<AccountId>,
    /// The current set of voters that rejected it.
    pub nays: Vec<AccountId>,
}
