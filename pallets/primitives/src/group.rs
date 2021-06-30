use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Group<GroupId, AccountId, MemberCount> {
    pub name: Vec<u8>,
    pub members: Vec<AccountId>,
    pub threshold: MemberCount,
    pub funding_account: AccountId,
    pub anonymous_account: AccountId,
    pub parent: Option<GroupId>,
}

// struct Ident {
//     account: Option<AccountId>,
//     did: Option<Did>,
// }

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
/// Info for keeping track of a motion being voted on.
pub struct Votes<AccountId, ProposalId, MemberCount> {
    /// The proposal's Id
    pub proposal_id: ProposalId,
    /// The number of approval votes that are needed to pass the motion.
    pub threshold: MemberCount,
    /// The current set of voters that approved it.
    pub ayes: Vec<AccountId>,
    /// The current set of voters that rejected it.
    pub nays: Vec<AccountId>,
}
