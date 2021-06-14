#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::group::Group;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait GroupsApi<AccountId,GroupId,MemberCount>
    where
    AccountId: Codec,
    GroupId: Codec,
    MemberCount: Codec
    {
        fn member_of(account:AccountId) -> Vec<GroupId>;
        fn get_group(group:GroupId) -> Option<Group<GroupId, AccountId, MemberCount>>;
        fn get_sub_groups(group:GroupId) -> Option<Vec<(GroupId,Group<GroupId, AccountId, MemberCount>)>>;
    }
}
