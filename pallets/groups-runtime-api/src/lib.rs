#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait GroupsApi<AccountId,GroupId>
    where
    AccountId: Codec,
    GroupId: Codec
     {

        fn member_of(account:AccountId) -> Vec<GroupId>;
    }
}
