#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::{Catalog, Claim, Did, DidDocument, DidProperty};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait IdentityApi<AccountId,CatalogId,ClaimId,MemberCount,Moment,BoundedStringName,BoundedStringFact>
    where
    AccountId: Codec,
    CatalogId: Codec,
    ClaimId: Codec,
    MemberCount: Codec,
    Moment: Codec,
    BoundedStringName: Codec + Into<Vec<u8>>,
    BoundedStringFact: Codec + Into<Vec<u8>>

     {
        fn get_catalogs(account_id: AccountId) -> Vec<(CatalogId,Catalog<BoundedStringName>)>;

        fn get_catalog(account_id: AccountId,catalog_id:CatalogId) -> Option<Catalog<BoundedStringName>>;

        fn get_dids_in_catalog(catalog_id:CatalogId) -> Vec<(Did,BoundedStringName)>;

        fn get_did_in_catalog(catalog_id:CatalogId, did:Did) -> Option<(BoundedStringName,  DidDocument<AccountId,BoundedStringName>,Vec<DidProperty<BoundedStringName,BoundedStringFact>>,Vec<AccountId>)>;

        fn get_did(did:Did) -> Option<(DidDocument<AccountId,BoundedStringName>,Vec<DidProperty<BoundedStringName,BoundedStringFact>>,Vec<AccountId>)>;

        fn get_dids_by_subject( subject: AccountId) -> Vec<(Did, Option<BoundedStringName>)>;

        fn get_dids_by_controller( controller: AccountId,) -> Vec<(Did, Option<BoundedStringName>)>;

        fn get_claims(did: Did) -> Vec<(ClaimId, Claim<AccountId,MemberCount,Moment,BoundedStringName, BoundedStringFact>)>;

        fn get_claim(did: Did, claim_id:ClaimId) -> Option<Claim<AccountId,MemberCount,Moment,BoundedStringName, BoundedStringFact>>;

        fn get_claim_consumers(did: Did) -> Vec<(AccountId,Moment)>;

        fn get_claim_issuers(did: Did) -> Vec<(AccountId,Moment)>;

        fn get_dids_by_consumer(consumer:AccountId) -> Vec<(Did,Moment)>;

        fn get_dids_by_issuer(issuer:AccountId) -> Vec<(Did,Moment)>;


    }
}
