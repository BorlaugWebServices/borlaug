//! # AssetRegistry Module
//!
//! ## Overview
//!
//! An asset registry is a data registry that mediates the creation, verification, updating, and
//! deactivation of digital and physical assets. Any account holder can create an asset registry.
//! An asset can be owned, shared and transferred to an account or a DID.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For general users
//! * `create_registry` - Creates a new asset registry
//! * `create_asset` - Creates a new asset within a registry
//! * `update_asset` - Updates a properties of an asset within a registry
//! * `delete_asset` - Delete an asset
//! * `new_lease` - Creates a new lease agreement between lessor and lessee for a set of assets
//! * `void_lease` - Void a lease and release assets from lease

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod tests;

use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, Parameter};
use frame_system::{self as system, ensure_signed};
use primitives::{
    asset::Asset,
    did::Did,
    lease_agreement::{AssetAllocation, LeaseAgreement},
};
use sp_runtime::{
    traits::{AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One},
    DispatchResult,
};
use sp_std::prelude::*;

pub trait Trait: frame_system::Trait + timestamp::Trait {
    type RegistryId: Parameter
        + Member
        + AtLeast32Bit
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + PartialEq;

    type AssetId: Parameter
        + Member
        + AtLeast32Bit
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + PartialEq;

    type LeaseId: Parameter
        + Member
        + AtLeast32Bit
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + PartialEq;

    type Balance: Parameter
        + Member
        + AtLeast32Bit
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + PartialEq;

    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
        where
        // <T as frame_system::Trait>::AccountId,
        <T as Trait>::RegistryId,
        <T as Trait>::AssetId,
        <T as Trait>::LeaseId,
    {
        /// New registry created (owner, registry id)
        RegistryCreated(Did, RegistryId),

        RegistryDeleted(RegistryId),
        /// New asset created in registry (registry id, asset id)
        AssetCreated(RegistryId, AssetId),
        /// Asset was updated in registry (registry id, asset id)
        AssetUpdated(RegistryId, AssetId),

        AssetDeleted(RegistryId, AssetId),

        LeaseCreated(LeaseId,Did,Did),

        LeaseVoided(LeaseId,Did),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Value was None
        NoneValue,
        /// the calling account is not the subject of owner_did
        NotDidSubject,
        /// A non-registry owner account attempted to  modify a registry or asset in the registry
        NotRegistryOwner,
        /// Id out of bounds
        NoIdAvailable
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as AssetRegistry {

        /// Incrementing nonce
        pub Nonce get(fn nonce) build(|_| 1u64): u64;

        /// The next available registry index
        pub NextRegistryId get(fn next_registry_id) config(): T::RegistryId;

        /// The next available asset index
        pub NextAssetId get(fn next_asset_id) config(): T::AssetId;

        /// The next available lease index
        pub NextLeaseId get(fn next_lease_id) config(): T::LeaseId;

        /// Permission options for a given asset.
        pub Registries get(fn registries):
            map hasher(blake2_128_concat) Did => Vec<T::RegistryId>;

        /// Registry of assets
        pub Assets get(fn assets):
            double_map hasher(twox_64_concat) T::RegistryId, hasher(twox_64_concat) T::AssetId =>
            Asset<T::Moment, T::Balance>;

        /// Lease allocations of assets
        pub LeaseAllocations get(fn allocations):
            double_map hasher(twox_64_concat) T::RegistryId, hasher(twox_64_concat) T::AssetId => Vec<(T::LeaseId,u64,T::Moment)>;


        /// Lease agreements by lessor
        pub LeaseAgreements get(fn leases):
            double_map hasher(blake2_128_concat) Did, hasher(twox_64_concat) T::LeaseId =>
            LeaseAgreement<T::RegistryId, T::AssetId, T::Moment>;

    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Create a new registry
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        #[weight = 100_000]
        fn create_registry(origin, owner_did: Did) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_did_subject(sender, owner_did),
            Error::<T>::NotDidSubject);

            let registry_id = Self::next_registry_id();
            let next_id = registry_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextRegistryId<T>>::put(next_id);

            <Registries<T>>::append(owner_did, &registry_id);

            Self::deposit_event(RawEvent::RegistryCreated(owner_did, registry_id));
        }


        /// Remove a registry
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Registry to be removed
        #[weight = 100_000]
        fn delete_registry(origin,owner_did: Did, registry_id: T::RegistryId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_did_subject(sender, owner_did),
            Error::<T>::NotDidSubject);

            <Registries<T>>::mutate(owner_did, |registries| {
                registries.retain(|rid| *rid != registry_id)
            });

            //TODO: is there any asset cleanup to do?

            Self::deposit_event(RawEvent::RegistryDeleted(registry_id));
        }

        /// Create a new asset within a given registry
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is created in this registry
        /// - `asset` instance to be added
        #[weight = 100_000]
        fn create_asset(
            origin,
            owner_did: Did,
            registry_id: T::RegistryId,
            asset: Asset<T::Moment, T::Balance>,
        )  {

            let sender = ensure_signed(origin)?;

            ensure!(Self::is_did_subject(sender, owner_did),
            Error::<T>::NotDidSubject);

            ensure!(Self::is_registry_owner(owner_did, registry_id),
            <Error<T>>::NotRegistryOwner);

            Self::create_registry_asset(registry_id, asset)?;
        }

        /// Update asset
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is in this registry
        /// - `asset_id` ID of Asset
        /// - `asset` instance to be updated
        #[weight = 100_000]
        fn update_asset(
            origin,
            owner_did: Did,
            registry_id: T::RegistryId,
            asset_id: T::AssetId,
            asset: Asset<T::Moment, T::Balance>,
        ) {
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_did_subject(sender, owner_did),
            Error::<T>::NotDidSubject);

            //TODO: does this fail correctly if asset does not exist?

            //TODO: we are replacing everything - should we instead update in some way?

            <Assets<T>>::remove(&registry_id, &asset_id);

            <Assets<T>>::insert(&registry_id, &asset_id, asset);

            Self::deposit_event(RawEvent::AssetUpdated(registry_id, asset_id));

        }

        /// Delete asset. Asset can't be removed if it's part of an active lease.
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is created in this registry
        /// - `asset_id` Asset to be deleted
        #[weight = 100_000]
        fn delete_asset(
            origin,
            owner_did: Did,
            registry_id: T::RegistryId,
            asset_id: T::AssetId
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_did_subject(sender, owner_did),
            Error::<T>::NotDidSubject);

            <Assets<T>>::remove(&registry_id, &asset_id);

            Self::deposit_event(RawEvent::AssetDeleted(registry_id, asset_id));

            Ok(())
        }

        /// Creates a new lease agreement.
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is created in this registry
        /// - `asset_id` Asset to be deleted
        #[weight = 100_000]
        fn new_lease(
            origin,
            lease: LeaseAgreement<T::RegistryId, T::AssetId, T::Moment>,
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;
            ensure!(Self::is_did_subject(sender, lease.lessor),
            Error::<T>::NotDidSubject);

            Self::create_new_lease( lease)?;

            Ok(())
        }

        /// Void a lease agreement. Allocations are un-reserved.
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is created in this registry
        /// - `asset_id` Asset to be deleted
        #[weight = 100_000]
        fn void_lease(
            origin,
            lessor: Did,
            lease_id: T::LeaseId
        ) -> DispatchResult {

            let sender = ensure_signed(origin)?;
            ensure!(Self::is_did_subject(sender, lessor),
            Error::<T>::NotDidSubject);

            <LeaseAgreements<T>>::remove(&lessor, &lease_id);

            //TODO: clean up lease allocations on asset

            Self::deposit_event(RawEvent::LeaseVoided(lease_id, lessor));

            Ok(())
        }


    }
}

// public functions
impl<T: Trait> Module<T> {
    pub fn get_asset(
        _registry_id: T::RegistryId,
        _asset_id: T::AssetId,
    ) -> Option<Asset<T::Moment, T::Balance>> {
        None
    }
}

// private functions
impl<T: Trait> Module<T> {
    /// Create an asset and store it in the given registry
    fn create_registry_asset(
        registry_id: T::RegistryId,
        asset: Asset<T::Moment, T::Balance>,
    ) -> DispatchResult {
        let asset_id = Self::next_asset_id();
        let next_id = asset_id
            .checked_add(&One::one())
            .ok_or(Error::<T>::NoIdAvailable)?;
        <NextAssetId<T>>::put(next_id);

        <Assets<T>>::insert(&registry_id, &asset_id, asset);
        Self::deposit_event(RawEvent::AssetCreated(registry_id, asset_id));

        Ok(())
    }
    /// Create an asset and store it in the given registry
    fn create_new_lease(
        lease: LeaseAgreement<T::RegistryId, T::AssetId, T::Moment>,
    ) -> DispatchResult {
        let can_allocate = !lease
            .allocations
            .clone()
            .into_iter()
            .any(|allocation| Self::check_allocation(allocation));

        ensure!(can_allocate, "Cannot allocate some assets");

        let lease_id = Self::next_lease_id();
        let next_id = lease_id
            .checked_add(&One::one())
            .ok_or(Error::<T>::NoIdAvailable)?;
        <NextLeaseId<T>>::put(next_id);

        let lessor = lease.lessor.clone();
        let lessee = lease.lessee.clone();

        lease
            .allocations
            .clone()
            .into_iter()
            .for_each(|allocation| Self::make_allocation(lease_id, allocation, lease.expiry_ts));

        <LeaseAgreements<T>>::insert(&lessor, &lease_id, lease);

        Self::deposit_event(RawEvent::LeaseCreated(lease_id, lessor, lessee));

        Ok(())
    }

    fn is_registry_owner(owner_did: Did, registry_id: T::RegistryId) -> bool {
        if <Registries<T>>::contains_key(owner_did.clone()) {
            let registry_ids = <Registries<T>>::get(owner_did);
            registry_ids.contains(&registry_id)
        } else {
            false
        }
    }
    fn is_did_subject(_sender: T::AccountId, _did: Did) -> bool {
        //TODO: verify that sender is the subject of did
        true
    }
    ///should return false if allocation is possible.
    fn check_allocation(asset_allocation: AssetAllocation<T::RegistryId, T::AssetId>) -> bool {
        if <Assets<T>>::contains_key(asset_allocation.registry_id, asset_allocation.asset_id) {
            let asset = <Assets<T>>::get(asset_allocation.registry_id, asset_allocation.asset_id);
            if let Some(total_shares) = asset.total_shares {
                if asset_allocation.allocated_shares > total_shares {
                    return true;
                }

                if <LeaseAllocations<T>>::contains_key(
                    asset_allocation.registry_id,
                    asset_allocation.asset_id,
                ) {
                    let now: T::Moment = <timestamp::Module<T>>::get();

                    let lease_allocations = <LeaseAllocations<T>>::get(
                        asset_allocation.registry_id,
                        asset_allocation.asset_id,
                    );
                    let not_expired_allocations: Vec<(T::LeaseId, u64, T::Moment)> =
                        lease_allocations
                            .into_iter()
                            .filter(|(_, _, expiry)| *expiry > now)
                            .collect();
                    <LeaseAllocations<T>>::insert(
                        &asset_allocation.registry_id,
                        &asset_allocation.asset_id,
                        &not_expired_allocations,
                    );
                    let total_allocated: u64 = not_expired_allocations
                        .into_iter()
                        .map(|(_, allocation, _)| allocation)
                        .sum();
                    total_allocated + asset_allocation.allocated_shares > total_shares
                } else {
                    false
                }
            } else {
                true
            }
        } else {
            true
        }
    }
    fn make_allocation(
        lease_id: T::LeaseId,
        asset_allocation: AssetAllocation<T::RegistryId, T::AssetId>,
        lease_expiry: T::Moment,
    ) {
        <LeaseAllocations<T>>::append(
            asset_allocation.registry_id,
            asset_allocation.asset_id,
            &(lease_id, asset_allocation.allocated_shares, lease_expiry),
        )
    }
}
