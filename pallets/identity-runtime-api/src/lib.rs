#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::{Catalog, Did, DidDocument};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait IdentityApi<AccountId,CatalogId>
    where
    AccountId: Codec,
    CatalogId: Codec,

     {
        fn get_catalogs(owner_did:Did) -> Vec<(CatalogId,Catalog)>;

        fn get_catalog(owner_did:Did,catalog_id:CatalogId) -> Option<Catalog>;

        fn get_dids_in_catalog(catalog_id:CatalogId) -> Vec<(Did,Option<Vec<u8>>)>;

        fn get_did_in_catalog(catalog_id:CatalogId, did:Did) -> Option<(Option<Vec<u8>>,  DidDocument<AccountId>)>;

        fn get_did(did:Did) -> Option<DidDocument<AccountId>>;

        fn get_dids_by_subject( subject: AccountId) -> Vec<(Did, Option<Vec<u8>>)>;

        fn get_dids_by_controller( controller: AccountId) -> Vec<(Did, Option<Vec<u8>>)>;


    }
}
