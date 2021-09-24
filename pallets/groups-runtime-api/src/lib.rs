#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::group::{Group, Votes};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait GroupsApi<AccountId,GroupId,MemberCount,ProposalId,Hash,BoundedString>
    where
    AccountId: Codec,
    GroupId: Codec,
    MemberCount: Codec,
    ProposalId: Codec,
    Hash: Codec + AsRef<[u8]>,
    BoundedString: Codec + Into<Vec<u8>>
    {
        fn member_of(account:AccountId) -> Vec<GroupId>;
        fn is_member(group:GroupId,account:AccountId) -> bool;
        fn get_group_account(group:GroupId) -> Option<AccountId>;
        fn get_group(group:GroupId) -> Option<(Group<GroupId, AccountId, MemberCount,BoundedString>,Vec<(AccountId, MemberCount)>)>;
        fn get_sub_groups(group:GroupId) -> Vec<(GroupId,Group<GroupId, AccountId, MemberCount,BoundedString>,Vec<(AccountId, MemberCount)>)>;
        fn get_proposal(group_id: GroupId,proposal_id:ProposalId) -> Option<(Hash,u32)> ;
        fn get_proposals(group:GroupId) -> Vec<(ProposalId, Hash,u32)>;
        fn get_voting(group:GroupId, proposal:ProposalId) -> Option<Votes<AccountId, MemberCount>>;
    }
}
