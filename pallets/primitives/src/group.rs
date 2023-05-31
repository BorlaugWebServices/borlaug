use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{dispatch::Vec, scale_info::TypeInfo};
use sp_runtime::RuntimeDebug;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct GroupMember<AccountId, MemberCount> {
    /// The members account
    pub account: AccountId,
    /// The voting weight of the member in the group
    pub weight: MemberCount,
}

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Group<GroupId, AccountId, MemberCount, BoundedString> {
    /// A name for the group.
    pub name: BoundedString,
    /// Total weight of all group members. Useful for knowing if a proposal will fail.
    pub total_vote_weight: MemberCount,
    /// Group threshold used for some proposals. Not all proposals use this threshold.
    pub threshold: MemberCount,
    /// The groups account. Pays for proposals.
    pub anonymous_account: AccountId,
    /// If this group is a subgroup, this is the parent group.
    pub parent: Option<GroupId>,
}
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
/// Info for keeping track of a motion being voted on.
pub struct Votes<AccountId, MemberCount> {
    /// The number of approval votes that are needed to pass the motion.
    pub threshold: MemberCount,
    /// The total_vote_weight of group at the time the proposal was made.
    pub total_vote_weight: MemberCount,
    /// The current set of voters that approved it.
    pub ayes: Vec<GroupMember<AccountId, MemberCount>>,
    /// The current set of voters that rejected it.
    pub nays: Vec<GroupMember<AccountId, MemberCount>>,
    /// whether or not the vote was vetoed
    pub veto: Option<bool>,
}
