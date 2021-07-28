#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::*;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait AssetRegistryApi<AccountId,RegistryId,AssetId,LeaseId,BoundedStringName>
    where
    AccountId: Codec,
    RegistryId: Codec,
    AssetId: Codec,
    LeaseId: Codec,
    BoundedStringName: Codec + Into<Vec<u8>>,
     {
        fn get_registries(did: Did) -> Vec<(RegistryId,Registry<BoundedStringName>)>;

    }
}
