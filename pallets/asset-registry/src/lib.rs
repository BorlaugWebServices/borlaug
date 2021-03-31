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

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
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

    #[pallet::config]
    /// Configure the pallet by specifying the parameters and types on which it depends.
    pub trait Config: frame_system::Config + timestamp::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

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
    }

    #[pallet::event]
    #[pallet::metadata(
        T::RegistryId = "RegistryId",
        T::AssetId = "AssetId",
        T::LeaseId = "LeaseId"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New registry created (owner, registry id)
        RegistryCreated(Did, T::RegistryId),

        RegistryDeleted(T::RegistryId),
        /// New asset created in registry (registry id, asset id)
        AssetCreated(T::RegistryId, T::AssetId),
        /// Asset was updated in registry (registry id, asset id)
        AssetUpdated(T::RegistryId, T::AssetId),

        AssetDeleted(T::RegistryId, T::AssetId),

        LeaseCreated(T::LeaseId, Did, Did),

        LeaseVoided(T::LeaseId, Did),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value was None
        NoneValue,
        /// the calling account is not the subject of owner_did
        NotDidSubject,
        /// A non-registry owner account attempted to  modify a registry or asset in the registry
        NotRegistryOwner,
        /// Id out of bounds
        NoIdAvailable,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    //TODO:initialize at 1
    /// Incrementing nonce
    pub type Nonce<T> = StorageValue<_, u64>;

    #[pallet::storage]
    #[pallet::getter(fn next_registry_id)]
    /// The next available registry index
    pub type NextRegistryId<T: Config> = StorageValue<_, T::RegistryId>;

    #[pallet::storage]
    #[pallet::getter(fn next_asset_id)]
    /// The next available asset index
    pub type NextAssetId<T: Config> = StorageValue<_, T::AssetId>;

    #[pallet::storage]
    #[pallet::getter(fn next_lease_id)]
    /// The next available lease index
    pub type NextLeaseId<T: Config> = StorageValue<_, T::LeaseId>;

    #[pallet::storage]
    #[pallet::getter(fn registries)]
    /// Permission options for a given asset.
    pub type Registries<T: Config> =
        StorageMap<_, Blake2_128Concat, Did, Vec<T::RegistryId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn assets)]
    /// Registry of assets
    pub type Assets<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::RegistryId,
        Blake2_128Concat,
        T::AssetId,
        Asset<T::Moment, T::Balance>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn allocations)]
    /// Lease allocations of assets
    pub type LeaseAllocations<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::RegistryId,
        Blake2_128Concat,
        T::AssetId,
        Vec<(T::LeaseId, u64, T::Moment)>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn leases)]
    /// Lease agreements by lessor
    pub type LeaseAgreements<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Did,
        Blake2_128Concat,
        T::LeaseId,
        LeaseAgreement<T::RegistryId, T::AssetId, T::Moment>,
        ValueQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new registry
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        fn create_registry(origin: OriginFor<T>, owner_did: Did) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_did_subject(sender, owner_did),
                Error::<T>::NotDidSubject
            );

            let registry_id = Self::next_registry_id().unwrap();
            let next_id = registry_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextRegistryId<T>>::put(next_id);

            <Registries<T>>::append(owner_did, &registry_id);

            Self::deposit_event(Event::RegistryCreated(owner_did, registry_id));
            Ok(().into())
        }

        /// Remove a registry
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Registry to be removed
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        fn delete_registry(
            origin: OriginFor<T>,
            owner_did: Did,
            registry_id: T::RegistryId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_did_subject(sender, owner_did),
                Error::<T>::NotDidSubject
            );

            <Registries<T>>::mutate(owner_did, |registries| {
                registries.retain(|rid| *rid != registry_id)
            });

            //TODO: is there any asset cleanup to do?

            Self::deposit_event(Event::RegistryDeleted(registry_id));
            Ok(().into())
        }

        /// Create a new asset within a given registry
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is created in this registry
        /// - `asset` instance to be added
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        fn create_asset(
            origin: OriginFor<T>,
            owner_did: Did,
            registry_id: T::RegistryId,
            asset: Asset<T::Moment, T::Balance>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_did_subject(sender, owner_did),
                Error::<T>::NotDidSubject
            );

            ensure!(
                Self::is_registry_owner(owner_did, registry_id),
                <Error<T>>::NotRegistryOwner
            );

            Self::create_registry_asset(registry_id, asset)?;
            Ok(().into())
        }

        /// Update asset
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is in this registry
        /// - `asset_id` ID of Asset
        /// - `asset` instance to be updated
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        fn update_asset(
            origin: OriginFor<T>,
            owner_did: Did,
            registry_id: T::RegistryId,
            asset_id: T::AssetId,
            asset: Asset<T::Moment, T::Balance>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                Self::is_did_subject(sender, owner_did),
                Error::<T>::NotDidSubject
            );

            //TODO: does this fail correctly if asset does not exist?

            //TODO: we are replacing everything - should we instead update in some way?

            <Assets<T>>::remove(&registry_id, &asset_id);

            <Assets<T>>::insert(&registry_id, &asset_id, asset);

            Self::deposit_event(Event::AssetUpdated(registry_id, asset_id));
            Ok(().into())
        }

        /// Delete asset. Asset can't be removed if it's part of an active lease.
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is created in this registry
        /// - `asset_id` Asset to be deleted
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        fn delete_asset(
            origin: OriginFor<T>,
            owner_did: Did,
            registry_id: T::RegistryId,
            asset_id: T::AssetId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                Self::is_did_subject(sender, owner_did),
                Error::<T>::NotDidSubject
            );

            <Assets<T>>::remove(&registry_id, &asset_id);

            Self::deposit_event(Event::AssetDeleted(registry_id, asset_id));

            Ok(().into())
        }

        /// Creates a new lease agreement.
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is created in this registry
        /// - `asset_id` Asset to be deleted
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        fn new_lease(
            origin: OriginFor<T>,
            lease: LeaseAgreement<T::RegistryId, T::AssetId, T::Moment>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                Self::is_did_subject(sender, lease.lessor),
                Error::<T>::NotDidSubject
            );

            Self::create_new_lease(lease)?;

            Ok(().into())
        }

        /// Void a lease agreement. Allocations are un-reserved.
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is created in this registry
        /// - `asset_id` Asset to be deleted
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        fn void_lease(
            origin: OriginFor<T>,
            lessor: Did,
            lease_id: T::LeaseId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                Self::is_did_subject(sender, lessor),
                Error::<T>::NotDidSubject
            );

            <LeaseAgreements<T>>::remove(&lessor, &lease_id);

            //TODO: clean up lease allocations on asset

            Self::deposit_event(Event::LeaseVoided(lease_id, lessor));

            Ok(().into())
        }
    }

    // public functions
    impl<T: Config> Module<T> {
        pub fn get_asset(
            _registry_id: T::RegistryId,
            _asset_id: T::AssetId,
        ) -> Option<Asset<T::Moment, T::Balance>> {
            None
        }
    }

    // private functions
    impl<T: Config> Module<T> {
        /// Create an asset and store it in the given registry
        fn create_registry_asset(
            registry_id: T::RegistryId,
            asset: Asset<T::Moment, T::Balance>,
        ) -> DispatchResult {
            let asset_id = Self::next_asset_id().unwrap();
            let next_id = asset_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextAssetId<T>>::put(next_id);

            <Assets<T>>::insert(&registry_id, &asset_id, asset);
            Self::deposit_event(Event::AssetCreated(registry_id, asset_id));

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

            let lease_id = Self::next_lease_id().unwrap();
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
                .for_each(|allocation| {
                    Self::make_allocation(lease_id, allocation, lease.expiry_ts)
                });

            <LeaseAgreements<T>>::insert(&lessor, &lease_id, lease);

            Self::deposit_event(Event::LeaseCreated(lease_id, lessor, lessee));

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
                let asset =
                    <Assets<T>>::get(asset_allocation.registry_id, asset_allocation.asset_id);
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
}
