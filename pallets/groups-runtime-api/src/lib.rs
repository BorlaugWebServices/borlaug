#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::group::{Group, GroupMember, Votes};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait GroupsApi<AccountId,GroupId,MemberCount,ProposalId,Hash,BoundedString,Balance>
    where
    AccountId: Codec,
    GroupId: Codec,
    MemberCount: Codec,
    ProposalId: Codec,
    Hash: Codec + AsRef<[u8]>,
    BoundedString: Codec + Into<Vec<u8>>,
    Balance: Codec
    {
        fn member_of(account:AccountId) -> Vec<(GroupId, Group<GroupId, AccountId, MemberCount, BoundedString>,Vec<GroupMember<AccountId, MemberCount>>,Balance)> ;
        fn is_member(group:GroupId,account:AccountId) -> bool;
        fn get_group_by_account(account:AccountId) -> Option<(GroupId,Group<GroupId, AccountId, MemberCount,BoundedString>,Vec<GroupMember<AccountId, MemberCount>>,Balance)>;
        fn get_group_account(group:GroupId) -> Option<AccountId>;
        fn get_group(group:GroupId) -> Option<(Group<GroupId, AccountId, MemberCount,BoundedString>,Vec<GroupMember<AccountId, MemberCount>>,Balance)>;
        fn get_sub_groups(group:GroupId) -> Vec<(GroupId,Group<GroupId, AccountId, MemberCount,BoundedString>,Vec<GroupMember<AccountId, MemberCount>>,Balance)>;
        fn get_proposal(proposal_id:ProposalId) -> Option<(ProposalId, GroupId,Vec<GroupMember<AccountId, MemberCount>>,Option<(Hash,u32)>,Votes<AccountId, MemberCount>)> ;
        fn get_proposals_by_group(group:GroupId) -> Vec<(ProposalId,GroupId, Vec<GroupMember<AccountId, MemberCount>>,Option<(Hash,u32)>,Votes<AccountId, MemberCount>)>;
        fn get_proposals_by_account(account_id: AccountId) -> Vec<(GroupId, Vec<(ProposalId, GroupId,Vec<GroupMember<AccountId, MemberCount>>,Option<(Hash,u32)>,Votes<AccountId, MemberCount>)>)>;

    }
}
