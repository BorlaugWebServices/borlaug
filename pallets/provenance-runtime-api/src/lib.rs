#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::registry::Registry;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait ProvenanceApi<AccountId,RegistryId>
    where
    AccountId: Codec,
    RegistryId: Codec,
     {
        fn get_registries(account:AccountId) -> Vec<(RegistryId, Registry)>;
        fn get_registry(account:AccountId,registry_id:RegistryId) -> Registry;
    }
}
