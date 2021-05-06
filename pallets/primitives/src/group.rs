use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use frame_system::Account;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Group<GroupId, AccountId, MemberCount> {
    pub parent: Option<GroupId>,
    pub name: Vec<u8>,
    pub members: Vec<AccountId>,
    pub threshold: MemberCount,
    pub funding_account: AccountId,
}
