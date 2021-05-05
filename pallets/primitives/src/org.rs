use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use frame_system::Account;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct OrgGroup<OrgGroupId, AccountId> {
    pub parent: Option<OrgGroupId>,
    pub name: Vec<u8>,
    pub members: Vec<AccountId>,
    pub required_votes: u8,
    /// If fund_source is None, parent group pays for transactions   
    pub fund_source: Option<AccountId>,
}
