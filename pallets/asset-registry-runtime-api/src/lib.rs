#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::*;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait AssetRegistryApi<AccountId,ProposalId,RegistryId,AssetId,LeaseId,Moment,Balance,BoundedStringName,BoundedStringFact>
    where
    AccountId: Codec,
    ProposalId: Codec,
    RegistryId: Codec,
    AssetId: Codec,
    LeaseId: Codec,
    Moment: Codec,
    Balance: Codec,
    BoundedStringName: Codec + Into<Vec<u8>>,
    BoundedStringFact: Codec + Into<Vec<u8>>,
     {
        fn get_registries(did: Did) -> Vec<(RegistryId,Registry<BoundedStringName>)>;

        fn get_registry(did: Did,registry_id:RegistryId) -> Option<Registry<BoundedStringName>>;

        fn get_assets(registry_id:RegistryId) -> Vec<(AssetId,Asset<Moment,Balance,BoundedStringName,BoundedStringFact>)>;

        fn get_asset(registry_id:RegistryId, asset_id:AssetId) -> Option<Asset<Moment,Balance,BoundedStringName,BoundedStringFact>>;

        fn get_leases(lessor: Did) -> Vec<(LeaseId,LeaseAgreement<ProposalId,RegistryId,AssetId,Moment,BoundedStringName>)>;

        fn get_lease(lessor: Did, lease_id:LeaseId) -> Option<LeaseAgreement<ProposalId,RegistryId,AssetId,Moment,BoundedStringName>>;

        fn get_lease_allocations(registry_id:RegistryId, asset_id:AssetId) -> Option<Vec<(LeaseId, u64, Moment)>>;


    }
}
