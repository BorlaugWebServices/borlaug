#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::{Catalog, Claim, Did, DidDocument, DidProperty};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait IdentityApi<AccountId,CatalogId,GroupId,ClaimId,Moment,BoundedStringName,BoundedStringFact>
    where
    AccountId: Codec,
    CatalogId: Codec,
    GroupId: Codec,
    ClaimId: Codec,
    Moment: Codec,
    BoundedStringName: Codec + Into<Vec<u8>>,
    BoundedStringFact: Codec + Into<Vec<u8>>

     {
        fn get_catalogs(group_id: GroupId,) -> Vec<(CatalogId,Catalog<BoundedStringName>)>;

        fn get_catalog(group_id: GroupId,catalog_id:CatalogId) -> Option<Catalog<BoundedStringName>>;

        fn get_dids_in_catalog(catalog_id:CatalogId) -> Vec<(Did,Option<BoundedStringName>)>;

        fn get_did_in_catalog(catalog_id:CatalogId, did:Did) -> Option<(Option<BoundedStringName>,  DidDocument<AccountId,GroupId,BoundedStringName>,Vec<DidProperty<BoundedStringName,BoundedStringFact>>)>;

        fn get_did(did:Did) -> Option<(DidDocument<AccountId,GroupId,BoundedStringName>,Vec<DidProperty<BoundedStringName,BoundedStringFact>>)>;

        fn get_dids_by_subject( subject: AccountId) -> Vec<(Did, Option<BoundedStringName>)>;

        fn get_dids_by_controller( group_id: GroupId,) -> Vec<(Did, Option<BoundedStringName>)>;

        fn get_claims(did: Did) -> Vec<(ClaimId, Claim<GroupId,Moment,BoundedStringName, BoundedStringFact>)>;
    }
}
