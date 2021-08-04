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

mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    pub use super::weights::WeightInfo;
    use core::convert::TryInto;
    use extrinsic_extra::GetExtrinsicExtra;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use primitives::{bounded_vec::BoundedVec, *};
    use sp_runtime::{
        traits::{AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One},
        Either,
    };
    use sp_std::prelude::*;

    const MODULE_INDEX: u8 = 4;

    #[repr(u8)]
    pub enum ExtrinsicIndex {
        Registry = 41,
        Asset = 42,
        Lease = 43,
    }

    #[pallet::config]
    /// Configure the pallet by specifying the parameters and types on which it depends.
    pub trait Config:
        frame_system::Config + timestamp::Config + groups::Config + identity::Config
    {
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

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The maximum length of a name or symbol stored on-chain.
        type NameLimit: Get<u32>;

        /// The maximum length of a name or symbol stored on-chain.
        type FactStringLimit: Get<u32>;

        /// The maximum number of properties an asset can have.
        type AssetPropertyLimit: Get<u32>;
        /// The maximum number of assets a lease can have.
        type LeaseAssetLimit: Get<u32>;
    }

    #[pallet::event]
    #[pallet::metadata(
        T::RegistryId = "RegistryId",
        T::AssetId = "AssetId",
        T::LeaseId = "LeaseId"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New registry created (owner_did, registry_id)
        RegistryCreated(Did, T::RegistryId),
        /// A registry was renamed (owner_did, registry_id)
        RegistryRenamed(Did, T::RegistryId),
        /// A registry was renamed (owner_did, registry_id)
        RegistryDeleted(Did, T::RegistryId),
        /// New asset created in registry (registry_id, asset_id)
        AssetCreated(T::RegistryId, T::AssetId),
        /// Asset was updated in registry (registry_id, asset_id)
        AssetUpdated(Did, T::RegistryId, T::AssetId),
        /// Asset was deleted in registry (registry_id, asset_id)
        AssetDeleted(Did, T::RegistryId, T::AssetId),

        LeaseCreated(T::LeaseId, Did, Did),

        LeaseVoided(T::LeaseId, Did),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value was None
        NoneValue,
        /// A string exceeds the maximum allowed length
        BadString,
        /// the calling account is not the subject of owner_did
        NotDidSubject,
        /// A non-registry owner account attempted to  modify a registry or asset in the registry
        NotRegistryOwner,
        /// Delete all assets in registry before deleting registry     
        RegistryNotEmpty,
        /// Id out of bounds
        NoIdAvailable,
        /// Some assets could not be allocated
        AssetAllocationFailed,
    }

    #[pallet::type_value]
    pub fn UnitDefault<T: Config>() -> u64 {
        1u64
    }

    #[pallet::type_value]
    pub fn RegistryIdDefault<T: Config>() -> T::RegistryId {
        1u32.into()
    }
    #[pallet::type_value]
    pub fn AssetIdDefault<T: Config>() -> T::AssetId {
        1u32.into()
    }
    #[pallet::type_value]
    pub fn LeaseIdDefault<T: Config>() -> T::LeaseId {
        1u32.into()
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    /// Incrementing nonce
    pub type Nonce<T> = StorageValue<_, u64, ValueQuery, UnitDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_registry_id)]
    /// The next available registry index
    pub type NextRegistryId<T: Config> =
        StorageValue<_, T::RegistryId, ValueQuery, RegistryIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_asset_id)]
    /// The next available asset index
    pub type NextAssetId<T: Config> = StorageValue<_, T::AssetId, ValueQuery, AssetIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_lease_id)]
    /// The next available lease index
    pub type NextLeaseId<T: Config> = StorageValue<_, T::LeaseId, ValueQuery, LeaseIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn registries)]
    /// Permission options for a given asset.
    pub type Registries<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Did,
        Blake2_128Concat,
        T::RegistryId,
        Registry<BoundedVec<u8, <T as Config>::NameLimit>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn assets)]
    /// Registry of assets
    pub(super) type Assets<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::RegistryId,
        Blake2_128Concat,
        T::AssetId,
        Asset<
            T::Moment,
            T::Balance,
            BoundedVec<u8, <T as Config>::NameLimit>,
            BoundedVec<u8, <T as Config>::FactStringLimit>,
        >,
        OptionQuery,
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
        OptionQuery,
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
        LeaseAgreement<
            T::RegistryId,
            T::AssetId,
            T::Moment,
            BoundedVec<u8, <T as Config>::NameLimit>,
        >,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new registry
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `name` name of registry
        #[pallet::weight(<T as Config>::WeightInfo::create_registry(
            name.len() as u32
        ))]
        pub fn create_registry(
            origin: OriginFor<T>,
            owner_did: Did,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let bounded_name = enforce_limit!(name);

            ensure!(
                <identity::DidBySubject<T>>::contains_key(&sender, &owner_did),
                Error::<T>::NotDidSubject
            );

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Registry as u8),
                &sender,
            );

            let registry_id = next_id!(NextRegistryId<T>, T);

            <Registries<T>>::insert(&owner_did, &registry_id, Registry { name: bounded_name });

            Self::deposit_event(Event::RegistryCreated(owner_did, registry_id));
            Ok(().into())
        }

        /// Rename registry
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Registry
        /// - `name` new name of registry
        #[pallet::weight(<T as Config>::WeightInfo::update_registry(
            name.len() as u32
        ))]
        pub fn update_registry(
            origin: OriginFor<T>,
            owner_did: Did,
            registry_id: T::RegistryId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let bounded_name = enforce_limit!(name);

            ensure!(
                <identity::DidBySubject<T>>::contains_key(&sender, &owner_did),
                Error::<T>::NotDidSubject
            );
            ensure!(
                <Registries<T>>::contains_key(&owner_did, registry_id),
                <Error<T>>::NotRegistryOwner
            );

            <Registries<T>>::insert(&owner_did, &registry_id, Registry { name: bounded_name });

            Self::deposit_event(Event::RegistryRenamed(owner_did, registry_id));
            Ok(().into())
        }

        /// Remove a registry
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Registry to be removed
        #[pallet::weight(<T as Config>::WeightInfo::delete_registry())]
        pub fn delete_registry(
            origin: OriginFor<T>,
            owner_did: Did,
            registry_id: T::RegistryId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <identity::DidBySubject<T>>::contains_key(&sender, &owner_did),
                Error::<T>::NotDidSubject
            );
            ensure!(
                <Registries<T>>::contains_key(&owner_did, registry_id),
                <Error<T>>::NotRegistryOwner
            );

            ensure!(
                <Assets<T>>::iter_prefix(registry_id).next().is_none(),
                Error::<T>::RegistryNotEmpty
            );

            <Registries<T>>::remove(&owner_did, registry_id);

            Self::deposit_event(Event::RegistryDeleted(owner_did, registry_id));
            Ok(().into())
        }

        /// Create a new asset within a given registry
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is created in this registry
        /// - `asset` instance to be added
        #[pallet::weight(<T as Config>::WeightInfo::create_asset(
            asset.name.len() as u32,
            asset.asset_number.as_ref().map_or(0,|n|n.len()) as u32,
            asset.serial_number.as_ref().map_or(0,|n|n.len()) as u32,
            get_max_property_name_len(&asset.properties),
            get_max_property_fact_len(&asset.properties),
            asset.properties.len() as u32,
        ))]
        pub fn create_asset(
            origin: OriginFor<T>,
            owner_did: Did,
            registry_id: T::RegistryId,
            asset: Asset<T::Moment, T::Balance, Vec<u8>, Vec<u8>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <identity::DidBySubject<T>>::contains_key(&sender, &owner_did),
                Error::<T>::NotDidSubject
            );

            ensure!(
                <Registries<T>>::contains_key(&owner_did, registry_id),
                <Error<T>>::NotRegistryOwner
            );

            let asset = Asset {
                properties: asset
                    .properties
                    .into_iter()
                    .map(|property| {
                        Ok(AssetProperty {
                            name: enforce_limit!(property.name),
                            fact: enforce_limit_fact!(property.fact),
                        })
                    })
                    .collect::<Result<Vec<_>, Error<T>>>()?,
                name: enforce_limit!(asset.name),
                asset_number: enforce_limit_option!(asset.asset_number),
                status: asset.status,
                serial_number: enforce_limit_option!(asset.serial_number),
                total_shares: asset.total_shares,
                residual_value: asset.residual_value,
                purchase_value: asset.purchase_value,
                acquired_date: asset.acquired_date,
            };

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Asset as u8),
                &sender,
            );

            let asset_id = next_id!(NextAssetId<T>, T);

            <Assets<T>>::insert(&registry_id, &asset_id, asset);
            Self::deposit_event(Event::AssetCreated(registry_id, asset_id));

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
        pub fn update_asset(
            origin: OriginFor<T>,
            owner_did: Did,
            registry_id: T::RegistryId,
            asset_id: T::AssetId,
            asset: Asset<T::Moment, T::Balance, Vec<u8>, Vec<u8>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);
            ensure!(
                <identity::DidBySubject<T>>::contains_key(&sender, &owner_did),
                Error::<T>::NotDidSubject
            );

            let asset = Asset {
                properties: asset
                    .properties
                    .into_iter()
                    .map(|property| {
                        Ok(AssetProperty {
                            name: enforce_limit!(property.name),
                            fact: enforce_limit_fact!(property.fact),
                        })
                    })
                    .collect::<Result<Vec<_>, Error<T>>>()?,
                name: enforce_limit!(asset.name),
                asset_number: enforce_limit_option!(asset.asset_number),
                status: asset.status,
                serial_number: enforce_limit_option!(asset.serial_number),
                total_shares: asset.total_shares,
                residual_value: asset.residual_value,
                purchase_value: asset.purchase_value,
                acquired_date: asset.acquired_date,
            };

            <Assets<T>>::insert(&registry_id, &asset_id, asset);

            Self::deposit_event(Event::AssetUpdated(owner_did, registry_id, asset_id));
            Ok(().into())
        }

        /// Delete asset. Asset can't be removed if it's part of an active lease.
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is created in this registry
        /// - `asset_id` Asset to be deleted
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn delete_asset(
            origin: OriginFor<T>,
            owner_did: Did,
            registry_id: T::RegistryId,
            asset_id: T::AssetId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);
            ensure!(
                <identity::DidBySubject<T>>::contains_key(&sender, &owner_did),
                Error::<T>::NotDidSubject
            );

            <Assets<T>>::remove(&registry_id, &asset_id);

            Self::deposit_event(Event::AssetDeleted(owner_did, registry_id, asset_id));

            Ok(().into())
        }

        /// Creates a new lease agreement.
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `registry_id` Asset is created in this registry
        /// - `asset_id` Asset to be deleted
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn new_lease(
            origin: OriginFor<T>,
            lease: LeaseAgreement<T::RegistryId, T::AssetId, T::Moment, Vec<u8>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);
            ensure!(
                <identity::DidBySubject<T>>::contains_key(&sender, &lease.lessor),
                Error::<T>::NotDidSubject
            );

            let can_allocate = !lease.allocations.iter().any(Self::check_allocation);

            ensure!(can_allocate, Error::<T>::AssetAllocationFailed);

            let contract_number_limited = enforce_limit!(lease.contract_number.clone());

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Lease as u8),
                &sender,
            );

            let lessor = lease.lessor;
            let lessee = lease.lessee;

            let lease_id = next_id!(NextLeaseId<T>, T);

            lease.allocations.iter().for_each(|allocation| {
                Self::make_allocation(lease_id, allocation, lease.expiry_ts)
            });

            let lease = LeaseAgreement {
                contract_number: contract_number_limited,
                lessor: lease.lessor,
                lessee: lease.lessee,
                effective_ts: lease.effective_ts,
                expiry_ts: lease.expiry_ts,
                allocations: lease.allocations,
            };

            <LeaseAgreements<T>>::insert(&lessor, &lease_id, lease);

            Self::deposit_event(Event::LeaseCreated(lease_id, lessor, lessee));

            Ok(().into())
        }

        //TODO: should we keep some record of voided leases?

        /// Void a lease agreement. Allocations are un-reserved.
        ///
        /// Arguments:
        /// - `lessor` DID of caller        
        /// - `lease_id` Lease to be deleted
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn void_lease(
            origin: OriginFor<T>,
            lessor: Did,
            lease_id: T::LeaseId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);
            ensure!(
                <identity::DidBySubject<T>>::contains_key(&sender, &lessor),
                Error::<T>::NotDidSubject
            );

            let lease_agreement = <LeaseAgreements<T>>::take(&lessor, &lease_id);

            ensure!(lease_agreement.is_some(), Error::<T>::NotDidSubject);

            lease_agreement
                .unwrap()
                .allocations
                .into_iter()
                .for_each(|allocation| {
                    <LeaseAllocations<T>>::mutate_exists(
                        allocation.registry_id,
                        allocation.asset_id,
                        |maybe_lease_allocations| {
                            if let Some(ref mut lease_allocations) = maybe_lease_allocations {
                                lease_allocations.retain(|(l_id, _, _)| *l_id != lease_id);
                            }
                        },
                    );
                });

            Self::deposit_event(Event::LeaseVoided(lease_id, lessor));

            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --

        pub fn get_registries(
            did: Did,
        ) -> Vec<(
            T::RegistryId,
            Registry<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut registries = Vec::new();
            <Registries<T>>::iter_prefix(did)
                .for_each(|(registry_id, registry)| registries.push((registry_id, registry)));
            registries
        }

        pub fn get_registry(
            did: Did,
            registry_id: T::RegistryId,
        ) -> Option<Registry<BoundedVec<u8, <T as Config>::NameLimit>>> {
            <Registries<T>>::get(did, registry_id)
        }

        pub fn get_assets(
            registry_id: T::RegistryId,
        ) -> Vec<(
            T::AssetId,
            Asset<
                T::Moment,
                T::Balance,
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
        )> {
            let mut assets = Vec::new();
            <Assets<T>>::iter_prefix(registry_id)
                .for_each(|(asset_id, asset)| assets.push((asset_id, asset)));
            assets
        }

        pub fn get_asset(
            registry_id: T::RegistryId,
            asset_id: T::AssetId,
        ) -> Option<
            Asset<
                T::Moment,
                T::Balance,
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
        > {
            <Assets<T>>::get(registry_id, asset_id)
        }

        pub fn get_leases(
            lessor: Did,
        ) -> Vec<(
            T::LeaseId,
            LeaseAgreement<
                T::RegistryId,
                T::AssetId,
                T::Moment,
                BoundedVec<u8, <T as Config>::NameLimit>,
            >,
        )> {
            let mut leases = Vec::new();
            <LeaseAgreements<T>>::iter_prefix(lessor)
                .for_each(|(lease_id, lease)| leases.push((lease_id, lease)));
            leases
        }

        pub fn get_lease(
            lessor: Did,
            lease_id: T::LeaseId,
        ) -> Option<
            LeaseAgreement<
                T::RegistryId,
                T::AssetId,
                T::Moment,
                BoundedVec<u8, <T as Config>::NameLimit>,
            >,
        > {
            <LeaseAgreements<T>>::get(lessor, lease_id)
        }

        pub fn get_lease_allocations(
            registry_id: T::RegistryId,
            asset_id: T::AssetId,
        ) -> Option<Vec<(T::LeaseId, u64, T::Moment)>> {
            <LeaseAllocations<T>>::get(registry_id, asset_id)
        }

        // -- private functions --

        ///should return false if allocation is possible.
        fn check_allocation(asset_allocation: &AssetAllocation<T::RegistryId, T::AssetId>) -> bool {
            let asset = <Assets<T>>::get(asset_allocation.registry_id, asset_allocation.asset_id);
            if asset.is_none() {
                return true;
            }
            let asset = asset.unwrap();
            if asset_allocation.allocated_shares > asset.total_shares {
                return true;
            }
            let lease_allocations =
                <LeaseAllocations<T>>::get(asset_allocation.registry_id, asset_allocation.asset_id);
            if lease_allocations.is_none() {
                return false;
            }
            let now: T::Moment = <timestamp::Module<T>>::get();

            let lease_allocations = lease_allocations.unwrap();

            let lease_allocations_len = lease_allocations.len();

            let not_expired_allocations: Vec<(T::LeaseId, u64, T::Moment)> = lease_allocations
                .into_iter()
                .filter(|(_, _, expiry)| *expiry > now)
                .collect();

            if not_expired_allocations.len() < lease_allocations_len {
                <LeaseAllocations<T>>::insert(
                    &asset_allocation.registry_id,
                    &asset_allocation.asset_id,
                    &not_expired_allocations,
                );
            }
            let total_allocated: u64 = not_expired_allocations
                .into_iter()
                .map(|(_, allocation, _)| allocation)
                .sum();
            total_allocated + asset_allocation.allocated_shares > asset.total_shares
        }
        fn make_allocation(
            lease_id: T::LeaseId,
            asset_allocation: &AssetAllocation<T::RegistryId, T::AssetId>,
            lease_expiry: T::Moment,
        ) {
            <LeaseAllocations<T>>::append(
                asset_allocation.registry_id,
                asset_allocation.asset_id,
                &(lease_id, asset_allocation.allocated_shares, lease_expiry),
            )
        }
    }

    macro_rules! max_fact_len {
        ($fact:expr,$max_fact_len:ident) => {{
            let fact_len = match &$fact {
                Fact::Text(string) => string.len() as u32,
                _ => 10, //give minimum of 10 and don't bother checking for anything other than Text
            };
            if fact_len > $max_fact_len {
                $max_fact_len = fact_len;
            };
        }};
    }

    fn get_max_property_name_len(properties: &Vec<AssetProperty<Vec<u8>, Vec<u8>>>) -> u32 {
        let mut max_property_name_len = 0;
        properties.into_iter().for_each(|property| {
            if property.name.len() as u32 > max_property_name_len {
                max_property_name_len = property.name.len() as u32;
            };
        });
        max_property_name_len
    }

    fn get_max_property_fact_len(properties: &Vec<AssetProperty<Vec<u8>, Vec<u8>>>) -> u32 {
        let mut max_fact_len = 0;
        properties.into_iter().for_each(|property| {
            max_fact_len!(property.fact, max_fact_len);
        });
        max_fact_len
    }
}
