#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::{Claim, Did, DidDocument, DidProperty};

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
        fn is_catalog_owner(account_id: AccountId, catalog_id:CatalogId) -> bool;

        fn get_catalogs(account_id: AccountId) -> Vec<CatalogId>;

        fn get_dids_in_catalog(catalog_id:CatalogId) -> Vec<Did>;

        fn get_catalogs_by_did(did:Did) -> Vec<CatalogId>;

        fn get_did_in_catalog(catalog_id:CatalogId, did:Did) -> Option<(DidDocument<AccountId>,Vec<DidProperty<BoundedStringName,BoundedStringFact>>,Vec<AccountId>)>;

        fn is_controller(account_id: AccountId, did:Did) -> bool;

        fn get_did(did:Did) -> Option<(DidDocument<AccountId>,Vec<DidProperty<BoundedStringName,BoundedStringFact>>,Vec<AccountId>)>;

        fn get_dids_by_subject( subject: AccountId) -> Vec<Did>;

        fn get_dids_by_controller( controller: AccountId,) -> Vec<Did>;

        fn find_did_by_text_or_did_property( catalog_id: CatalogId, name:Vec<u8> ,filter: Vec<u8>) -> Vec<Did>;

        fn find_did_by_integer_property( catalog_id: CatalogId, name:Vec<u8> ,min: Option<u128>,max: Option<u128>) -> Vec<Did>;

        fn find_did_by_float_property( catalog_id: CatalogId, name:Vec<u8> ,min: Option<[u8;8]>,max: Option<[u8;8]>) -> Vec<Did>;

        fn find_did_by_date_property( catalog_id: CatalogId, name:Vec<u8> ,min: Option<(u16, u8, u8)>,max: Option<(u16, u8, u8)>) -> Vec<Did>;

        // fn find_did_by_iso8601_property( catalog_id: CatalogId, name:Vec<u8> ,min: Option<(u16, u8, u8,u8, u8, u8, Vec<u8>)>,max: Option<(u16, u8, u8,u8, u8, u8, Vec<u8>)>) -> Vec<Did>;

        fn get_claims(did: Did) -> Vec<(ClaimId, Claim<AccountId,MemberCount,Moment,BoundedStringName, BoundedStringFact>)>;

        fn get_claim(did: Did, claim_id:ClaimId) -> Option<Claim<AccountId,MemberCount,Moment,BoundedStringName, BoundedStringFact>>;

        fn get_claim_consumers(did: Did) -> Vec<(AccountId,Moment)>;

        fn get_claim_issuers(did: Did) -> Vec<(AccountId,Moment)>;

        fn get_dids_by_consumer(consumer:AccountId) -> Vec<(Did,Moment)>;

        fn get_dids_by_issuer(issuer:AccountId) -> Vec<(Did,Moment)>;

        fn get_outstanding_claims(consumer:AccountId) -> Vec<(Did,Moment)>;

        fn get_outstanding_attestations(issuer:AccountId) -> Vec<(Did,Moment)>;


    }
}
