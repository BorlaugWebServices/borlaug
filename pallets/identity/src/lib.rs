//! # Identity Module
//!
//! ## Overview
//!
//! A DID catalog is a data catalog that mediates the creation, verification, updating, and
//! deactivation of decentralized identifiers (DIDs). Any account holder can create a DID. A DID
//! and DID document do not inherently carry any PII (personally-identifiable information).
//!
//! DID Subject is the entity identified by the DID and described by the DID document.
//!
//! DID Controller is the entity, or a group of entities, in control of a DID or DID document.
//! Note that the DID controller might include the DID subject.
//!
//! DIDs are organized by catalogs. An entity might use catalogs for KYC providers, vendors,
//! suppliers and so on.
//!
//! DIDs can have claims associated with them. A claim is a cryptographically non-repudiable set of
//! statements made by an entity about another entity.
//! * Consumers request verifiable claims from people and organizations in order to give them access
//!   to protected resources.
//! * Issuers provide verifiable claims to people and organizations
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For general users
//! * `register_did` - Creates a new DID and registers it for the caller
//! * `create_catalog` - Creates a new Catalog for organizing DIDs into collections
//! * `remove_catalog` - Remove a Catalog
//! * `add_dids_to_catalog` - Add dids to a Catalog
//! * `remove_dids_from_catalog` - Remove a DID from in a catalog
//!
//! #### For Controllers
//! * `register_did_for` - Registers a new DID for a subject and adds caller as a controller
//! * `register_did_for_bulk` - Registers a collection of DIDs for subjects and adds caller as a controller
//! * `add_did_properties` - Add properties to a DID Document
//! * `remove_did_properties` - Remove properties from a DID Document
//! * `manage_controllers` - Add or remove controllers for the did. Subject cannot be removed.
//! * `authorize_claim_consumers` - Grant permission to claim consumers to add claims to a DID
//! * `revoke_claim_consumers` - Remove permission from claim consumers to add claims to a DID
//! * `authorize_claim_issuers` - Grant permission to claim issuers to attest claims to a DID
//! * `revoke_claim_issuers` - Remove permission from claim issuers to attest claims to a DID
//!
//! #### For Claim Consumers
//! * `make_claim` - Claim consumer makes a claim against a DID.
//!
//! #### For Claim Verifiers
//! * `attest_claim` - Claim consumer makes a claim against a DID.
//! * `revoke_attestation` - Claim consumer makes a claim against a DID.
//!
//! ### RPC Methods
//! * `get_catalogs` - Get the collection of catalogs owned by the caller.
//! * `is_catalog_owner` - Get a catalog name.
//! * `get_dids_in_catalog` - Get the collection of DIDs in a catalog.
//! * `get_did_in_catalog` - Get a DID with its catalog label and its DID Document.
//! * `get_did` - Get a DID with its short name and its DID Document.
//! * `get_dids_by_subject` - Get the collection of DIDs with the specified subject.
//! * `get_dids_by_controller` - Get the collection of DIDs with the specified controller.
//! * `get_claims` - Get the collection of claims against a DID.
//! * `get_claim_consumers` - Get the list of claim consumers for a DID.
//! * `get_claim_issuers` - Get the list of claim issuers for a DID.
//! * `get_dids_by_consumer` - Get the list DIDs by claim consumer.
//! * `get_dids_by_issuer` - Get the list DIDs by claim issuer.
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

pub mod migration;

#[frame_support::pallet]
pub mod pallet {
    pub use super::weights::WeightInfo;
    use codec::Encode;
    use core::convert::TryInto;
    use extrinsic_extra::GetExtrinsicExtra;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Randomness,
    };
    use frame_system::pallet_prelude::*;
    use primitives::{bounded_vec::BoundedVec, *};
    use sp_runtime::{
        traits::{AtLeast32Bit, CheckedAdd, Hash, One, Saturating, UniqueSaturatedFrom},
        Either,
    };
    use sp_std::prelude::*;
    use utils::like::ILike;

    const MODULE_INDEX: u8 = 2;

    #[repr(u8)]
    pub enum ExtrinsicIndex {
        Catalog = 21,
        Did = 22,
        Claim = 23,
    }

    #[derive(Encode, Decode, Clone, frame_support::RuntimeDebug, PartialEq)]
    pub enum Releases {
        V1,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + timestamp::Config + groups::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type CatalogId: Parameter + AtLeast32Bit + Copy + PartialEq;

        type ClaimId: Parameter + AtLeast32Bit + Copy + PartialEq;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The maximum length of a name or symbol stored on-chain.
        type NameLimit: Get<u32>;

        /// The maximum length of a name or symbol stored on-chain.
        type FactStringLimit: Get<u32>;

        /// The maximum number of properties you can add to a DID at one time (does not limit total)
        #[pallet::constant]
        type PropertyLimit: Get<u32>;

        /// The maximum number of statements a Claim may have
        #[pallet::constant]
        type StatementLimit: Get<u32>;

        /// The maximum number of controllers you can add/remove at one time (does not limit total)
        #[pallet::constant]
        type ControllerLimit: Get<u32>;
        /// The maximum number of claim consumers you can add/remove at one time (does not limit total)
        #[pallet::constant]
        type ClaimConsumerLimit: Get<u32>;
        /// The maximum number of claim issuers you can add/remove at one time (does not limit total)
        #[pallet::constant]
        type ClaimIssuerLimit: Get<u32>;
        /// The maximum number of dids you can add/remove to a catalog at one time (does not limit total)
        #[pallet::constant]
        type CatalogDidLimit: Get<u32>;
        /// The maximum number of dids you can register in bulk at one time. Watch out for weight and block size limits too.
        #[pallet::constant]
        type BulkDidLimit: Get<u32>;
        /// The maximum number of properties you can add to a DID using bulk did route
        #[pallet::constant]
        type BulkDidPropertyLimit: Get<u32>;
    }

    #[pallet::event]
    // #[pallet::metadata(
    //     T::AccountId = "AccountId",
    //     T::Moment = "Moment",
    //     T::CatalogId = "CatalogId",
    //     T::ClaimId = "ClaimId",
    //     Vec<ClaimConsumer<T::AccountId, T::Moment>> = "ClaimConsumers",
    //     Vec<ClaimIssuer<T::AccountId, T::Moment>> = "ClaimIssuers",
    //     Vec<T::AccountId> = "AccountIds",
    //     Option<Vec<T::AccountId>> = "Option<AccountIds>"
    // )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new DID was registered
        /// (caller, subject, controller, did)
        Registered(T::AccountId, T::AccountId, T::AccountId, Did),
        /// DIDs were bulk registered
        /// (caller, controller, count)
        BulkRegistered(T::AccountId, T::AccountId, u32),
        /// Did properties added
        /// (caller, controller, did)
        DidPropertiesAdded(T::AccountId, T::AccountId, Did),
        /// Did properties removed
        /// (caller, controller, did)
        DidPropertiesRemoved(T::AccountId, T::AccountId, Did),
        /// DID Controller updated
        /// (caller, controller, target_did, added_controllers, removed_controllers)
        DidControllerUpdated(
            T::AccountId,
            T::AccountId,
            Did,
            Option<Vec<T::AccountId>>,
            Option<Vec<T::AccountId>>,
        ),
        /// Claim consumers added
        /// (caller, controller, did, claim_consumers)
        ClaimConsumersAuthorized(
            T::AccountId,
            T::AccountId,
            Did,
            Vec<ClaimConsumer<T::AccountId, T::Moment>>,
        ),
        /// Claim consumers removed
        /// (caller, controller, did, claim_consumers)
        ClaimConsumersRevoked(T::AccountId, T::AccountId, Did, Vec<T::AccountId>),
        /// Claim issuers added
        /// (caller, controller, did, claim_consumers)
        ClaimIssuersAuthorized(
            T::AccountId,
            T::AccountId,
            Did,
            Vec<ClaimIssuer<T::AccountId, T::Moment>>,
        ),
        /// Claim issuers removed
        /// (caller, controller, did, claim_consumers)
        ClaimIssuersRevoked(T::AccountId, T::AccountId, Did, Vec<T::AccountId>),
        /// Claim was made against a DID
        /// (caller, issuer, target_did, claim_id)
        ClaimMade(T::AccountId, T::AccountId, Did, T::ClaimId),
        /// Claim was attested
        /// ( attester, target_did, claim_id)
        ClaimAttested(T::AccountId, Did, T::ClaimId),
        /// Claim attestation revoked
        /// (attester, target_did, claim_id)
        ClaimAttestationRevoked(T::AccountId, Did, T::ClaimId),
        /// Catalog added
        /// (caller, controller, catalog_id)
        CatalogCreated(T::AccountId, T::AccountId, T::CatalogId),
        /// Catalog removed
        /// (caller, controller, catalog_id)
        CatalogRemoved(T::AccountId, T::AccountId, T::CatalogId),
        /// Dids added to catalog
        /// (caller, controller, catalog_id)
        CatalogDidsAdded(T::AccountId, T::AccountId, T::CatalogId),
        /// Dids removed from catalog
        /// (caller, controller, catalog_id)
        CatalogDidsRemoved(T::AccountId, T::AccountId, T::CatalogId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value was None
        NoneValue,
        /// A string exceeds the maximum allowed length
        StringLengthLimitExceeded,
        /// Value was not found
        NotFound,
        /// Too many properties
        PropertyLimitExceeded,
        /// Controller limit exceeded. Call extrinsic multiple times to add/remove more.
        ControllerLimitExceeded,
        /// Claim Consumer limit exceeded. Call extrinsic multiple times to add/remove more.
        ClaimConsumerLimitExceeded,
        /// Claim Issuer limit exceeded. Call extrinsic multiple times to add/remove more.
        ClaimIssuerLimitExceeded,
        /// Too many statements
        StatementLimitExceeded,
        /// Catalog Did limit exceeded. Call extrinsic multiple times to add/remove more.
        CatalogDidLimitExceeded,
        /// Bulk Did limit exceeded.
        BulkDidLimitExceeded,
        /// A non-controller account attempted to  modify a DID
        NotController,
        /// A catalog must be empty to be removed
        CatalogNotEmpty,
        /// The requested DID Document does not exist
        DidDocumentNotFound,
        /// Not authorized to make a claim or attest a claim
        NotAuthorized,
        /// The required threshold of votes to attest a claim was not met
        ThresholdNotMet,
        /// Id out of bounds
        NoIdAvailable,
    }
    //TODO: can we do these initializations in generate_store instead?
    #[pallet::type_value]
    pub fn UnitDefault<T: Config>() -> u64 {
        1u64
    }

    #[pallet::type_value]
    pub fn ClaimIdDefault<T: Config>() -> T::ClaimId {
        1u32.into()
    }

    #[pallet::type_value]
    pub fn CatalogIdDefault<T: Config>() -> T::CatalogId {
        1u32.into()
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // fn on_runtime_upgrade() -> frame_support::weights::Weight {
        //     let mut weight: Weight = 0;
        //     // weight += super::migration::migrate_to_v2::<T>();
        //     // weight += super::migration::migrate_to_v3::<T>();
        //     weight
        // }
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        phantom: PhantomData<T>,
    }
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                phantom: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <StorageVersion<T>>::put(Releases::V1);
        }
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    /// Storage version of the pallet.
    ///
    /// V2 - added DidCatalogs
    /// V3 - added issued to attestation
    pub type StorageVersion<T> = StorageValue<_, Releases, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    /// Incrementing nonce
    pub type Nonce<T> = StorageValue<_, u64, ValueQuery, UnitDefault<T>>;

    /// The next available claim index, aka the number of claims started so far.
    #[pallet::storage]
    #[pallet::getter(fn claim_count)]
    pub type NextClaimId<T: Config> = StorageValue<_, T::ClaimId, ValueQuery, ClaimIdDefault<T>>;

    /// The next available catalog index
    #[pallet::storage]
    #[pallet::getter(fn next_catalog_id)]
    pub type NextCatalogId<T: Config> =
        StorageValue<_, T::CatalogId, ValueQuery, CatalogIdDefault<T>>;

    /// An account can have multiple DIDs
    /// AccountId , Did => ()
    #[pallet::storage]
    #[pallet::getter(fn dids_by_subject)]
    pub type DidBySubject<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, Did, (), OptionQuery>;

    /// An account can control multiple DIDs.
    /// Controller AccountId , Did => ()
    #[pallet::storage]
    #[pallet::getter(fn dids_by_controller)]
    pub type DidByController<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, Did, (), OptionQuery>;

    /// Controllers of a DID.
    /// Controller Did, AccountId => ()
    #[pallet::storage]
    #[pallet::getter(fn controllers)]
    pub type DidControllers<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, Did, Blake2_128Concat, T::AccountId, (), OptionQuery>;

    /// A DID has a DID Document
    /// Did => DidDocument
    #[pallet::storage]
    #[pallet::getter(fn did_documents)]
    pub type DidDocuments<T: Config> =
        StorageMap<_, Blake2_128Concat, Did, DidDocument<T::AccountId>, OptionQuery>;

    //TODO: when Full BoundedVec support released, use BoundedVec not hash for storage key and remove name from DidProperty and use Fact here.

    /// A DidDocument has properties
    /// Did => DidDocumentProperty
    #[pallet::storage]
    #[pallet::getter(fn did_document_properties)]
    pub type DidDocumentProperties<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Did,
        Blake2_128Concat,
        T::Hash,
        DidProperty<
            BoundedVec<u8, <T as Config>::NameLimit>,
            BoundedVec<u8, <T as Config>::FactStringLimit>,
        >,
        OptionQuery,
    >;

    /// Claim consumers request a claim to offer protected services
    /// Subject DID => DIDs of claim consumers
    #[pallet::storage]
    #[pallet::getter(fn claim_comsumers)]
    pub type ClaimConsumers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Did,
        Blake2_128Concat,
        T::AccountId,
        T::Moment,
        OptionQuery,
    >;

    /// Claim consumers request a claim to offer protected services
    /// Subject DID => DIDs of claim consumers
    #[pallet::storage]
    #[pallet::getter(fn dids_by_consumer)]
    pub type DidsByConsumer<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        Did,
        T::Moment,
        OptionQuery,
    >;

    /// Claim issuers provide verifiable claims
    /// Subject DID => (DIDs of claim issuers, Expiration time)
    #[pallet::storage]
    #[pallet::getter(fn claim_issuers)]
    pub type ClaimIssuers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Did,
        Blake2_128Concat,
        T::AccountId,
        T::Moment,
        OptionQuery,
    >;
    /// Claim issuers provide verifiable claims
    /// Subject DID => (DIDs of claim issuers, Expiration time)
    #[pallet::storage]
    #[pallet::getter(fn dids_by_issuer)]
    pub type DidsByIssuer<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        Did,
        T::Moment,
        OptionQuery,
    >;

    /// Claims associated with a DID
    /// Subject DID => (Claim ID => Claim)
    #[pallet::storage]
    #[pallet::getter(fn claims)]
    pub type Claims<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Did,
        Blake2_128Concat,
        T::ClaimId,
        Claim<
            T::AccountId,
            T::MemberCount,
            T::Moment,
            BoundedVec<u8, <T as Config>::NameLimit>,
            BoundedVec<u8, <T as Config>::FactStringLimit>,
        >,
        OptionQuery,
    >;

    /// Dids may be organized into catalogs
    #[pallet::storage]
    #[pallet::getter(fn catalogs)]
    pub type Catalogs<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::CatalogId,
        (),
        OptionQuery,
    >;

    /// For each catalog index, we keep a mapping of `Did` an index name
    #[pallet::storage]
    #[pallet::getter(fn dids_by_catalog)]
    pub type DidsByCatalog<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::CatalogId, Blake2_128Concat, Did, (), OptionQuery>;

    /// For each did we keep a record or which catalogs they are in
    #[pallet::storage]
    #[pallet::getter(fn did_catalogs)]
    pub type DidCatalogs<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, Did, Blake2_128Concat, T::CatalogId, (), OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new DID for caller. Subject calls to create a new DID.
        ///
        /// Arguments:
        /// - `properties` initial DID properties to be added to the new DID
        #[pallet::weight(<T as Config>::WeightInfo::register_did(
            get_max_property_name_len_option(properties),
            get_max_property_fact_len_option(properties),
            properties.as_ref().map_or(0,|properties|properties.len()) as u32,
        ))]
        pub fn register_did(
            origin: OriginFor<T>,
            properties: Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            let property_count = properties.as_ref().map_or(0, |p| p.len());

            ensure!(
                property_count <= T::PropertyLimit::get() as usize,
                Error::<T>::PropertyLimitExceeded
            );

            let properties = enforce_limit_did_properties_option!(properties);

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Did as u8),
                &group_account,
            );

            Self::mint_did(&account_id, &group_account, &group_account, properties);

            Ok(().into())
        }

        /// Register a new DID on behalf of another user (subject).
        ///
        /// Arguments:
        /// - `subject` the Subject of the DID
        /// - `properties` initial DID properties to be added to the new DID
        #[pallet::weight(<T as Config>::WeightInfo::register_did(
            get_max_property_name_len_option(properties),
            get_max_property_fact_len_option(properties),
            properties.as_ref().map_or(0,|properties|properties.len()) as u32,
        ))]
        pub fn register_did_for(
            origin: OriginFor<T>,
            subject: T::AccountId,
            properties: Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            let property_count = properties.as_ref().map_or(0, |p| p.len());

            ensure!(
                property_count <= T::PropertyLimit::get() as usize,
                Error::<T>::PropertyLimitExceeded
            );

            let properties = enforce_limit_did_properties_option!(properties);

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Did as u8),
                &group_account,
            );

            Self::mint_did(&account_id, &subject, &group_account, properties);
            Ok(().into())
        }

        /// Register a number of new DIDs on behalf of users (subject).
        ///
        /// Arguments:
        /// - `dids` the DIDs to be created
        #[pallet::weight({
            let (a,b,c)=get_did_for_bulk_lens::<T>(dids);
            <T as Config>::WeightInfo::register_did_for(a,b,c).saturating_mul(dids.len() as Weight)

        })]
        pub fn register_did_for_bulk(
            origin: OriginFor<T>,
            dids: Vec<(T::AccountId, Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>)>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            let did_count = dids.len();

            ensure!(
                did_count < T::BulkDidLimit::get() as usize,
                Error::<T>::BulkDidLimitExceeded
            );

            let dids = dids
                .into_iter()
                .map(|(subject, properties)| {
                    let property_count = properties.as_ref().map_or(0, |p| p.len());

                    ensure!(
                        property_count <= (T::BulkDidPropertyLimit::get()) as usize,
                        Error::<T>::PropertyLimitExceeded
                    );

                    let properties = enforce_limit_did_properties_option!(properties);

                    Ok((subject, properties))
                })
                .collect::<Result<Vec<_>, Error<T>>>()?;

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(did_count as u8 * ExtrinsicIndex::Did as u8),
                &group_account,
            );

            dids.into_iter().for_each(|(subject, properties)| {
                Self::mint_did(&account_id, &subject, &group_account, properties);
            });

            Self::deposit_event(Event::BulkRegistered(
                account_id,
                group_account,
                did_count as u32,
            ));

            Ok(().into())
        }

        /// Append given collection of Did properties provided the caller is a controller
        ///
        /// Arguments:
        /// - `did` DID to which properties are to be added
        /// - `properties` DID properties to be added
        #[pallet::weight(<T as Config>::WeightInfo::add_did_properties(
            get_max_property_name_len(properties),
            get_max_property_fact_len(properties),
            properties.len() as u32,
        ))]
        pub fn add_did_properties(
            origin: OriginFor<T>,
            did: Did,
            properties: Vec<DidProperty<Vec<u8>, Vec<u8>>>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                properties.len() <= T::PropertyLimit::get() as usize,
                Error::<T>::PropertyLimitExceeded
            );

            let properties = enforce_limit_did_properties!(properties);

            ensure!(
                <DidByController<T>>::contains_key(&group_account, &did),
                Error::<T>::NotController
            );

            properties.into_iter().for_each(|add_property| {
                let hash = T::Hashing::hash_of(&add_property.name);
                <DidDocumentProperties<T>>::insert(&did, &hash, add_property);
            });

            Self::deposit_event(Event::DidPropertiesAdded(account_id, group_account, did));
            Ok(().into())
        }

        /// Append given collection of Did properties provided the caller is a controller
        ///
        /// Arguments:
        /// - `did` DID to which properties are to be added
        /// - `keys` Keys of DID properties to be removed
        #[pallet::weight(<T as Config>::WeightInfo::remove_did_properties(
            get_max_key_len(keys),
            keys.len() as u32,
        ))]
        pub fn remove_did_properties(
            origin: OriginFor<T>,
            did: Did,
            keys: Vec<Vec<u8>>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                keys.len() <= T::PropertyLimit::get() as usize,
                Error::<T>::PropertyLimitExceeded
            );

            let keys = keys
                .into_iter()
                .map(|key| Ok(enforce_limit!(key)))
                .collect::<Result<Vec<_>, Error<T>>>()?;

            ensure!(
                <DidByController<T>>::contains_key(&group_account, &did),
                Error::<T>::NotController
            );

            keys.into_iter().for_each(|key| {
                let hash = T::Hashing::hash_of(&key);
                <DidDocumentProperties<T>>::remove(&did, &hash);
            });

            Self::deposit_event(Event::DidPropertiesRemoved(account_id, group_account, did));
            Ok(().into())
        }

        /// Add or remove DID controllers for a DID. Subject cannot be removed.
        ///
        /// Arguments:
        /// - `did` subject
        /// - `add` DIDs to be added as controllers
        /// - `remove` DIDs to be removed as controllers
        #[pallet::weight(<T as Config>::WeightInfo::manage_controllers(
            add.as_ref().map_or(0,|a|a.len()) as u32,
            remove.as_ref().map_or(0,|a|a.len()) as u32,
        ))]
        pub fn manage_controllers(
            origin: OriginFor<T>,
            target_did: Did,
            add: Option<Vec<T::AccountId>>,
            remove: Option<Vec<T::AccountId>>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);
            ensure!(
                <DidByController<T>>::contains_key(&group_account, &target_did),
                Error::<T>::NotController
            );

            let add_count = add.as_ref().map_or(0, |a| a.len());
            let remove_count = remove.as_ref().map_or(0, |a| a.len());

            ensure!(
                add_count <= T::ControllerLimit::get() as usize
                    && remove_count <= T::ControllerLimit::get() as usize,
                Error::<T>::ControllerLimitExceeded
            );

            let did_document = <DidDocuments<T>>::try_get(&target_did)
                .map_err(|_| Error::<T>::DidDocumentNotFound)?;

            if let Some(remove) = remove.clone() {
                remove.into_iter().for_each(|remove| {
                    if did_document.subject != remove {
                        <DidByController<T>>::remove(&remove, &target_did);
                        <DidControllers<T>>::remove(&target_did, &remove);
                    }
                });
            }
            if let Some(add) = add.clone() {
                add.into_iter().for_each(|add| {
                    <DidByController<T>>::insert(&add, &target_did, ());
                    <DidControllers<T>>::insert(&target_did, &add, ());
                });
            }
            Self::deposit_event(Event::DidControllerUpdated(
                account_id,
                group_account,
                target_did,
                add,
                remove,
            ));
            Ok(().into())
        }

        /// Grants a claim consumer permission to write a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_consumers` DIDs of claim consumer
        #[pallet::weight(<T as Config>::WeightInfo::authorize_claim_consumers(
            claim_consumers.len() as u32
        ))]
        pub fn authorize_claim_consumers(
            origin: OriginFor<T>,
            target_did: Did,
            claim_consumers: Vec<ClaimConsumer<T::AccountId, T::Moment>>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <DidByController<T>>::contains_key(&group_account, &target_did),
                Error::<T>::NotController
            );

            ensure!(
                claim_consumers.len() <= T::ClaimConsumerLimit::get() as usize,
                Error::<T>::ClaimConsumerLimitExceeded
            );

            claim_consumers.iter().for_each(|claim_consumer| {
                <ClaimConsumers<T>>::insert(
                    &target_did,
                    &claim_consumer.consumer,
                    claim_consumer.expiration,
                );
                <DidsByConsumer<T>>::insert(
                    &claim_consumer.consumer,
                    &target_did,
                    claim_consumer.expiration,
                );
            });

            Self::deposit_event(Event::ClaimConsumersAuthorized(
                account_id,
                group_account,
                target_did,
                claim_consumers,
            ));
            Ok(().into())
        }

        /// Revokes claim consumers permission to write a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_consumers` DIDs of claim consumers to be revoked
        #[pallet::weight(<T as Config>::WeightInfo::revoke_claim_consumers(
            claim_consumers.len() as u32
        ))]
        pub fn revoke_claim_consumers(
            origin: OriginFor<T>,
            target_did: Did,
            claim_consumers: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <DidByController<T>>::contains_key(&group_account, &target_did),
                Error::<T>::NotController
            );

            ensure!(
                claim_consumers.len() <= T::ClaimConsumerLimit::get() as usize,
                Error::<T>::ClaimConsumerLimitExceeded
            );

            claim_consumers.iter().for_each(|claim_consumer| {
                <ClaimConsumers<T>>::remove(&target_did, claim_consumer);
                <DidsByConsumer<T>>::remove(claim_consumer, &target_did);
            });

            Self::deposit_event(Event::ClaimConsumersRevoked(
                account_id,
                group_account,
                target_did,
                claim_consumers,
            ));
            Ok(().into())
        }

        /// Grants a claim attestor permission to attest a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_issuers` DIDs of claim issuer
        #[pallet::weight(<T as Config>::WeightInfo::authorize_claim_issuers(
            claim_issuers.len() as u32
        ))]
        pub fn authorize_claim_issuers(
            origin: OriginFor<T>,
            target_did: Did,
            claim_issuers: Vec<ClaimIssuer<T::AccountId, T::Moment>>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <DidByController<T>>::contains_key(&group_account, &target_did),
                Error::<T>::NotController
            );

            ensure!(
                claim_issuers.len() <= T::ClaimIssuerLimit::get() as usize,
                Error::<T>::ClaimIssuerLimitExceeded
            );

            claim_issuers.iter().for_each(|claim_issuer| {
                <ClaimIssuers<T>>::insert(
                    &target_did,
                    &claim_issuer.issuer,
                    claim_issuer.expiration,
                );
                <DidsByIssuer<T>>::insert(
                    &claim_issuer.issuer,
                    &target_did,
                    claim_issuer.expiration,
                );
            });

            Self::deposit_event(Event::ClaimIssuersAuthorized(
                account_id,
                group_account,
                target_did,
                claim_issuers,
            ));
            Ok(().into())
        }

        /// Revokes claim issuers permission to attest a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_issuers` DIDs of claim issuers to be revoked
        #[pallet::weight(<T as Config>::WeightInfo::revoke_claim_issuers(
            claim_issuers.len() as u32
        ))]
        pub fn revoke_claim_issuers(
            origin: OriginFor<T>,
            target_did: Did,
            claim_issuers: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <DidByController<T>>::contains_key(&group_account, &target_did),
                Error::<T>::NotController
            );

            ensure!(
                claim_issuers.len() <= T::ClaimIssuerLimit::get() as usize,
                Error::<T>::ClaimIssuerLimitExceeded
            );

            claim_issuers.iter().for_each(|claim_issuer| {
                <ClaimIssuers<T>>::remove(&target_did, claim_issuer);
                <DidsByIssuer<T>>::remove(claim_issuer, &target_did);
            });

            Self::deposit_event(Event::ClaimIssuersRevoked(
                account_id,
                group_account,
                target_did,
                claim_issuers,
            ));
            Ok(().into())
        }

        /// Claim consumer calls this to make a new `claim` against `target_did`
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `description` description of claim
        /// - `statements` statements of claim
        /// - `threshold` threshold required to attest claim if group makes attestation
        #[pallet::weight(<T as Config>::WeightInfo::make_claim(
            description.len() as u32,
            statements.len() as u32,
            get_max_statement_name_len(statements),
            get_max_statement_fact_len(statements),

        ))]
        pub fn make_claim(
            origin: OriginFor<T>,
            target_did: Did,
            description: Vec<u8>,
            statements: Vec<Statement<Vec<u8>, Vec<u8>>>,
            threshold: T::MemberCount,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                Self::is_valid_consumer(&target_did, &group_account),
                Error::<T>::NotAuthorized
            );

            ensure!(
                statements.len() <= T::StatementLimit::get() as usize,
                Error::<T>::StatementLimitExceeded
            );

            let statements = statements
                .into_iter()
                .map(|statement| {
                    Ok(Statement {
                        name: enforce_limit!(statement.name),
                        fact: enforce_limit_fact!(statement.fact),
                        for_issuer: statement.for_issuer,
                    })
                })
                .collect::<Result<Vec<_>, Error<T>>>()?;

            let claim = Claim {
                description: enforce_limit!(description),
                statements,
                created_by: group_account.clone(),
                attestation: None,
                threshold,
            };

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Claim as u8),
                &group_account,
            );

            let claim_id = next_id!(NextClaimId<T>, T);

            <Claims<T>>::insert(&target_did, claim_id, claim);

            Self::deposit_event(Event::ClaimMade(
                account_id,
                group_account,
                target_did,
                claim_id,
            ));
            Ok(().into())
        }

        /// Claim issuer attests `claim_id` against `target_did` with given `statements`
        ///
        /// Arguments:
        /// - `target_did` DID against which claims are to be attested
        /// - `claim_id` Claim to be attested
        /// - `statements` Claim issuer overwrites these statements
        /// - `valid_until` Attestation expires
        #[pallet::weight(<T as Config>::WeightInfo::attest_claim(
            statements.len() as u32,
            <T as Config>::StatementLimit::get() as u32,
            get_max_statement_name_len(statements),
            get_max_statement_fact_len(statements),

        ))]
        pub fn attest_claim(
            origin: OriginFor<T>,
            target_did: Did,
            claim_id: T::ClaimId,
            statements: Vec<Statement<Vec<u8>, Vec<u8>>>,
            valid_until: T::Moment,
        ) -> DispatchResultWithPostInfo {
            //TODO: use macro
            let either = T::GroupsOriginAccountOrApproved::ensure_origin(origin)?;
            let (account_id, yes_votes) = match either {
                Either::Left(account_id) => (account_id, None),
                Either::Right((_, _, yes_votes, _, group_account)) => (group_account, yes_votes),
            };

            ensure!(
                Self::is_valid_issuer(&target_did, &account_id),
                Error::<T>::NotAuthorized
            );

            ensure!(
                statements.len() <= T::StatementLimit::get() as usize,
                Error::<T>::StatementLimitExceeded
            );

            let statements_len = statements.len();

            let max_statement_name_len = get_max_statement_name_len(&statements);
            let max_statement_fact_len = get_max_statement_fact_len(&statements);

            let bounded_statements = statements
                .into_iter()
                .map(|statement| {
                    Ok(Statement {
                        name: enforce_limit!(statement.name),
                        fact: enforce_limit_fact!(statement.fact),
                        for_issuer: statement.for_issuer,
                    })
                })
                .collect::<Result<Vec<_>, Error<T>>>()?;

            if yes_votes.is_some() {
                let claim = <Claims<T>>::get(&target_did, claim_id);
                ensure!(claim.is_some(), Error::<T>::NotFound);
                ensure!(
                    yes_votes.unwrap() >= claim.unwrap().threshold,
                    Error::<T>::ThresholdNotMet
                );
            }

            let mut existing_statements_len = 0;

            <Claims<T>>::mutate_exists(&target_did, claim_id, |maybe_claim| {
                if let Some(ref mut claim) = maybe_claim {
                    existing_statements_len = claim.statements.len();

                    let mut stmts = bounded_statements.clone();

                    let names = bounded_statements
                        .into_iter()
                        .map(|s| s.name)
                        .collect::<Vec<_>>();

                    claim.statements.retain(|s| !names.contains(&s.name));
                    claim.statements.append(&mut stmts);
                    claim.attestation = Some(Attestation {
                        attested_by: account_id.clone(),
                        issued: <timestamp::Module<T>>::get(),
                        valid_until,
                    });
                }
            });

            Self::deposit_event(Event::ClaimAttested(account_id, target_did, claim_id));

            Ok(Some(<T as Config>::WeightInfo::attest_claim(
                statements_len as u32,
                existing_statements_len as u32,
                max_statement_name_len,
                max_statement_fact_len,
            ))
            .into())
        }

        /// Claim issuer revokes `claim_id`
        ///
        /// Arguments:
        /// - `target_did` DID against which claims are to be attested
        /// - `claim_id` Claim to be attested
        #[pallet::weight(<T as Config>::WeightInfo::revoke_attestation(
            <T as Config>::StatementLimit::get() as u32,
            <T as Config>::NameLimit::get() as u32,
            <T as Config>::FactStringLimit::get() as u32,
        ))]
        pub fn revoke_attestation(
            origin: OriginFor<T>,
            target_did: Did,
            claim_id: T::ClaimId,
        ) -> DispatchResultWithPostInfo {
            let either = T::GroupsOriginAccountOrApproved::ensure_origin(origin)?;
            let (account_id, yes_votes) = match either {
                Either::Left(account_id) => (account_id, None),
                Either::Right((_, _, yes_votes, _, group_account)) => (group_account, yes_votes),
            };

            ensure!(
                Self::is_valid_issuer(&target_did, &account_id),
                Error::<T>::NotAuthorized
            );

            if yes_votes.is_some() {
                let claim = <Claims<T>>::get(&target_did, claim_id);
                ensure!(claim.is_some(), Error::<T>::NotFound);
                ensure!(
                    yes_votes.unwrap() >= claim.unwrap().threshold,
                    Error::<T>::ThresholdNotMet
                );
            }

            let mut existing_statements_len = 0;
            let mut max_statement_name_len = 0;
            let mut max_statement_fact_len = 0;

            <Claims<T>>::mutate_exists(&target_did, claim_id, |maybe_claim| {
                if let Some(ref mut claim) = maybe_claim {
                    //for correct weight calculation
                    existing_statements_len = claim.statements.len();
                    max_statement_name_len =
                        get_max_statement_name_bounded_len::<T>(&claim.statements);
                    max_statement_fact_len =
                        get_max_statement_fact_bounded_len::<T>(&claim.statements);

                    claim.attestation = None;
                }
            });

            Self::deposit_event(Event::ClaimAttestationRevoked(
                account_id, target_did, claim_id,
            ));
            Ok(Some(<T as Config>::WeightInfo::revoke_attestation(
                existing_statements_len as u32,
                max_statement_name_len as u32,
                max_statement_fact_len as u32,
            ))
            .into())
        }

        /// Add a new catalog
        ///
        /// Arguments:
        /// - `name` name of the catalog
        #[pallet::weight(<T as Config>::WeightInfo::create_catalog())]
        pub fn create_catalog(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Catalog as u8),
                &group_account,
            );

            let catalog_id = next_id!(NextCatalogId<T>, T);

            <Catalogs<T>>::insert(&group_account, catalog_id, ());

            Self::deposit_event(Event::CatalogCreated(account_id, group_account, catalog_id));
            Ok(().into())
        }

        /// Remove a catalog
        ///
        /// Arguments:
        /// - `catalog_id` Catalog to be removed
        #[pallet::weight(<T as Config>::WeightInfo::remove_catalog())]
        pub fn remove_catalog(
            origin: OriginFor<T>,
            catalog_id: T::CatalogId,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Catalogs<T>>::contains_key(&group_account, catalog_id),
                Error::<T>::NotController
            );

            ensure!(
                <DidsByCatalog<T>>::iter_prefix(catalog_id).next().is_none(),
                Error::<T>::CatalogNotEmpty
            );

            <Catalogs<T>>::remove(&group_account, catalog_id);

            Self::deposit_event(Event::CatalogRemoved(account_id, group_account, catalog_id));

            Ok(().into())
        }

        /// Add DIDs to a catalog
        ///
        /// Arguments:
        /// - `catalog_id` Catalog to which DID are to be added
        /// - `dids` DIDs are to be added
        #[pallet::weight(<T as Config>::WeightInfo::add_dids_to_catalog(
            dids.len() as u32,
        ))]
        pub fn add_dids_to_catalog(
            origin: OriginFor<T>,
            catalog_id: T::CatalogId,
            dids: Vec<Did>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Catalogs<T>>::contains_key(&group_account, catalog_id),
                Error::<T>::NotController
            );

            ensure!(
                dids.len() < T::CatalogDidLimit::get() as usize,
                Error::<T>::CatalogDidLimitExceeded
            );

            for did in dids.into_iter() {
                ensure!(<DidDocuments<T>>::contains_key(&did), Error::<T>::NotFound);

                <DidsByCatalog<T>>::insert(catalog_id, &did, ());
                <DidCatalogs<T>>::insert(&did, catalog_id, ());
            }

            Self::deposit_event(Event::CatalogDidsAdded(
                account_id,
                group_account,
                catalog_id,
            ));
            Ok(().into())
        }

        /// Remove DIDs from a catalog
        ///
        /// Arguments:
        /// - `catalog_id` Catalog to which DID are to be removed
        /// - `dids` DIDs are to be removed
        #[pallet::weight(<T as Config>::WeightInfo::remove_dids_from_catalog(
            dids.len() as u32,
        ))]
        pub fn remove_dids_from_catalog(
            origin: OriginFor<T>,
            catalog_id: T::CatalogId,
            dids: Vec<Did>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Catalogs<T>>::contains_key(&group_account, catalog_id),
                Error::<T>::NotController
            );

            ensure!(
                dids.len() < T::CatalogDidLimit::get() as usize,
                Error::<T>::CatalogDidLimitExceeded
            );

            for did in dids.into_iter() {
                <DidsByCatalog<T>>::remove(catalog_id, did);
                <DidCatalogs<T>>::remove(&did, catalog_id);
            }

            Self::deposit_event(Event::CatalogDidsRemoved(
                account_id,
                group_account,
                catalog_id,
            ));
            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --

        pub fn is_catalog_owner(account_id: T::AccountId, catalog_id: T::CatalogId) -> bool {
            <Catalogs<T>>::contains_key(account_id, catalog_id)
        }

        pub fn get_catalogs(account_id: T::AccountId) -> Vec<T::CatalogId> {
            let mut catalogs = Vec::new();
            <Catalogs<T>>::iter_prefix(account_id)
                .for_each(|(catalog_id, _)| catalogs.push(catalog_id));
            catalogs
        }

        pub fn get_dids_in_catalog(catalog_id: T::CatalogId) -> Vec<Did> {
            let mut dids = Vec::new();
            <DidsByCatalog<T>>::iter_prefix(catalog_id).for_each(|(did, _)| dids.push(did));
            dids
        }

        pub fn get_catalogs_by_did(did: Did) -> Vec<T::CatalogId> {
            let mut catalogs = Vec::new();
            <DidCatalogs<T>>::iter_prefix(did)
                .for_each(|(catalog_id, _)| catalogs.push(catalog_id));
            catalogs
        }

        pub fn get_did_in_catalog(
            catalog_id: T::CatalogId,
            did: Did,
        ) -> Option<(
            DidDocument<T::AccountId>,
            Vec<
                DidProperty<
                    BoundedVec<u8, <T as Config>::NameLimit>,
                    BoundedVec<u8, <T as Config>::FactStringLimit>,
                >,
            >,
            Vec<T::AccountId>,
        )> {
            let exists = <DidsByCatalog<T>>::contains_key(catalog_id, did);
            exists
                .then(|| {
                    <DidDocuments<T>>::get(did).map(|did_document| {
                        let mut properties = Vec::new();
                        <DidDocumentProperties<T>>::iter_prefix(&did)
                            .for_each(|(_hash, property)| properties.push(property));
                        let mut controllers = Vec::new();
                        <DidControllers<T>>::iter_prefix(&did)
                            .for_each(|(controller, _)| controllers.push(controller));
                        (did_document, properties, controllers)
                    })
                })
                .flatten()
        }

        pub fn is_controller(account_id: T::AccountId, did: Did) -> bool {
            <DidControllers<T>>::contains_key(&did, &account_id)
        }

        pub fn get_did(
            did: Did,
        ) -> Option<(
            DidDocument<T::AccountId>,
            Vec<
                DidProperty<
                    BoundedVec<u8, <T as Config>::NameLimit>,
                    BoundedVec<u8, <T as Config>::FactStringLimit>,
                >,
            >,
            Vec<T::AccountId>,
        )> {
            <DidDocuments<T>>::get(did).map(|did_document| {
                let mut properties = Vec::new();
                <DidDocumentProperties<T>>::iter_prefix(&did)
                    .for_each(|(_hash, property)| properties.push(property));
                let mut controllers = Vec::new();
                <DidControllers<T>>::iter_prefix(&did)
                    .for_each(|(controller, _)| controllers.push(controller));
                (did_document, properties, controllers)
            })
        }

        pub fn get_dids_by_subject(subject: T::AccountId) -> Vec<Did> {
            let mut did_documents = Vec::new();
            <DidBySubject<T>>::iter_prefix(subject).for_each(|(did, _)| did_documents.push(did));
            did_documents
        }

        pub fn get_dids_by_controller(controller: T::AccountId) -> Vec<Did> {
            let mut did_documents = Vec::new();
            <DidByController<T>>::iter_prefix(controller)
                .for_each(|(did, _)| did_documents.push(did));
            did_documents
        }

        pub fn find_did_by_text_or_did_property(
            catalog_id: T::CatalogId,
            name: Vec<u8>,
            filter: Vec<u8>,
        ) -> Vec<Did> {
            let mut matching_dids = vec![];
            let hash = T::Hashing::hash_of(&name);
            let filter: &[u8] = &filter;
            <DidsByCatalog<T>>::iter_prefix(catalog_id).for_each(|(did, _)| {
                let property_maybe = <DidDocumentProperties<T>>::get(&did, hash);
                if let Some(property) = property_maybe {
                    let matching = match property.fact {
                        Fact::Text(value) => {
                            let value: Vec<u8> = value.into();
                            let value: &[u8] = &value;
                            ILike::<false>::ilike(value, filter).unwrap_or(false)
                        }
                        Fact::Did(value) => {
                            ILike::<false>::ilike(&value.id[..], filter).unwrap_or(false)
                        }
                        _ => false,
                    };
                    if matching {
                        matching_dids.push(did);
                    }
                };
            });
            matching_dids
        }

        pub fn find_did_by_integer_property(
            catalog_id: T::CatalogId,
            name: Vec<u8>,
            min: Option<u128>,
            max: Option<u128>,
        ) -> Vec<Did> {
            let mut matching_dids = vec![];
            let hash = T::Hashing::hash_of(&name);
            <DidsByCatalog<T>>::iter_prefix(catalog_id).for_each(|(did, _)| {
                let property_maybe = <DidDocumentProperties<T>>::get(&did, hash);
                if let Some(property) = property_maybe {
                    let matching = match property.fact {
                        Fact::U8(value) => min_max_check(value, min, max),
                        Fact::U16(value) => min_max_check(value, min, max),
                        Fact::U32(value) => min_max_check(value, min, max),
                        Fact::U128(value) => min_max_check(value, min, max),
                        _ => false,
                    };
                    if matching {
                        matching_dids.push(did);
                    }
                };
            });
            matching_dids
        }
        pub fn find_did_by_float_property(
            catalog_id: T::CatalogId,
            name: Vec<u8>,
            min: Option<[u8; 8]>,
            max: Option<[u8; 8]>,
        ) -> Vec<Did> {
            let mut matching_dids = vec![];
            let hash = T::Hashing::hash_of(&name);
            <DidsByCatalog<T>>::iter_prefix(catalog_id).for_each(|(did, _)| {
                let property_maybe = <DidDocumentProperties<T>>::get(&did, hash);
                if let Some(property) = property_maybe {
                    let matching = match property.fact {
                        Fact::Float(value) => {
                            let min_check = if let Some(min) = min {
                                f64::from_le_bytes(value) >= f64::from_le_bytes(min)
                            } else {
                                true
                            };
                            let max_check = if let Some(max) = max {
                                f64::from_le_bytes(value) <= f64::from_le_bytes(max)
                            } else {
                                true
                            };
                            min_check && max_check
                        }
                        _ => false,
                    };
                    if matching {
                        matching_dids.push(did);
                    }
                };
            });
            matching_dids
        }

        pub fn find_did_by_date_property(
            catalog_id: T::CatalogId,
            name: Vec<u8>,
            min: Option<(u16, u8, u8)>,
            max: Option<(u16, u8, u8)>,
        ) -> Vec<Did> {
            let mut matching_dids = vec![];
            let hash = T::Hashing::hash_of(&name);
            <DidsByCatalog<T>>::iter_prefix(catalog_id).for_each(|(did, _)| {
                let property_maybe = <DidDocumentProperties<T>>::get(&did, hash);
                if let Some(property) = property_maybe {
                    let matching = match property.fact {
                        Fact::Date(year, month, day) => {
                            let min_check = if let Some((min_year, min_month, min_day)) = min {
                                year > min_year
                                    || (year == min_year
                                        && (month > min_month
                                            || (month == min_month && day >= min_day)))
                            } else {
                                true
                            };
                            let max_check = if let Some((max_year, max_month, max_day)) = max {
                                year < max_year
                                    || (year == max_year
                                        && (month < max_month
                                            || (month == max_month && day <= max_day)))
                            } else {
                                true
                            };
                            min_check && max_check
                        }
                        _ => false,
                    };
                    if matching {
                        matching_dids.push(did);
                    }
                };
            });
            matching_dids
        }

        pub fn find_did_by_iso8601_property(
            catalog_id: T::CatalogId,
            name: Vec<u8>,
            min: Option<(u16, u8, u8, u8, u8, u8, Vec<u8>)>,
            max: Option<(u16, u8, u8, u8, u8, u8, Vec<u8>)>,
        ) -> Vec<Did> {
            let mut matching_dids = vec![];
            let hash = T::Hashing::hash_of(&name);
            <DidsByCatalog<T>>::iter_prefix(catalog_id).for_each(|(did, _)| {
                let property_maybe = <DidDocumentProperties<T>>::get(&did, hash);
                if let Some(property) = property_maybe {
                    //TODO: take care of time zones
                    let matching = match property.fact {
                        Fact::Iso8601(year, month, day, hour, minute, second, _timezone) => {
                            let min_check = if let Some((
                                min_year,
                                min_month,
                                min_day,
                                min_hour,
                                min_minute,
                                min_second,
                                _min_timezone,
                            )) = &min
                            {
                                year > *min_year
                                    || (year == *min_year
                                        && (month > *min_month
                                            || (month == *min_month
                                                && (day > *min_day
                                                    || (day == *min_day
                                                        && (hour > *min_hour
                                                            || (hour == *min_hour
                                                                && (minute > *min_minute
                                                                    || (minute
                                                                        == *min_minute
                                                                        && (second
                                                                            >= *min_second))))))))))
                            } else {
                                true
                            };
                            let max_check = if let Some((
                                max_year,
                                max_month,
                                max_day,
                                max_hour,
                                max_minute,
                                max_second,
                                _max_timezone,
                            )) = &max
                            {
                                year < *max_year
                                    || (year == *max_year
                                        && (month < *max_month
                                            || (month == *max_month
                                                && (day < *max_day
                                                    || (day == *max_day
                                                        && (hour < *max_hour
                                                            || (hour == *max_hour
                                                                && (minute < *max_minute
                                                                    || (minute
                                                                        == *max_minute
                                                                        && (second
                                                                            <= *max_second))))))))))
                            } else {
                                true
                            };
                            min_check && max_check
                        }
                        _ => false,
                    };
                    if matching {
                        matching_dids.push(did);
                    }
                };
            });
            matching_dids
        }

        pub fn get_claims(
            did: Did,
        ) -> Vec<(
            T::ClaimId,
            Claim<
                T::AccountId,
                T::MemberCount,
                <T as timestamp::Config>::Moment,
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
        )> {
            let mut claims = Vec::new();
            <Claims<T>>::iter_prefix(did)
                .for_each(|(claim_id, claim)| claims.push((claim_id, claim)));
            claims
        }

        pub fn get_claim(
            did: Did,
            claim_id: T::ClaimId,
        ) -> Option<
            Claim<
                T::AccountId,
                T::MemberCount,
                T::Moment,
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
        > {
            <Claims<T>>::get(did, claim_id)
        }

        pub fn get_claim_consumers(did: Did) -> Vec<(T::AccountId, T::Moment)> {
            let mut claim_consumers = Vec::new();
            <ClaimConsumers<T>>::iter_prefix(did)
                .for_each(|(account_id, expiry)| claim_consumers.push((account_id, expiry)));
            claim_consumers
        }

        pub fn get_claim_issuers(did: Did) -> Vec<(T::AccountId, T::Moment)> {
            let mut claim_issuers = Vec::new();
            <ClaimIssuers<T>>::iter_prefix(&did)
                .for_each(|(account_id, expiry)| claim_issuers.push((account_id, expiry)));
            claim_issuers
        }

        pub fn get_dids_by_consumer(account: T::AccountId) -> Vec<(Did, T::Moment)> {
            let mut dids = Vec::new();
            <DidsByConsumer<T>>::iter_prefix(account)
                .for_each(|(did, expiry)| dids.push((did, expiry)));
            dids
        }

        pub fn get_dids_by_issuer(account: T::AccountId) -> Vec<(Did, T::Moment)> {
            let mut dids = Vec::new();
            <DidsByIssuer<T>>::iter_prefix(account)
                .for_each(|(did, expiry)| dids.push((did, expiry)));
            dids
        }

        pub fn get_outstanding_claims(account: T::AccountId) -> Vec<(Did, T::Moment)> {
            let mut dids = Vec::new();
            <DidsByConsumer<T>>::iter_prefix(&account).for_each(|(did, expiry)| {
                if !<Claims<T>>::iter_prefix(did).any(|(_, claim)| claim.created_by == account) {
                    dids.push((did, expiry))
                }
            });
            dids
        }

        pub fn get_outstanding_attestations(account: T::AccountId) -> Vec<(Did, T::Moment)> {
            let mut dids = Vec::new();
            <DidsByIssuer<T>>::iter_prefix(account).for_each(|(did, expiry)| {
                if !<Claims<T>>::iter_prefix(did).any(|(_, claim)| claim.attestation.is_some()) {
                    dids.push((did, expiry))
                }
            });
            dids
        }

        // -- private functions --

        /// Returns true if a `account` is a consumer and expiry has not yet passed
        pub fn is_valid_consumer(target_did: &Did, account: &T::AccountId) -> bool {
            <ClaimConsumers<T>>::contains_key(target_did, account) && {
                let expiry = <ClaimConsumers<T>>::get(target_did, account).unwrap();
                let now = <timestamp::Module<T>>::get();
                expiry.saturating_mul(T::Moment::unique_saturated_from(1_000u32)) > now
            }
        }

        /// Returns true if a `account` an issuer and expiry has not yet passed
        pub fn is_valid_issuer(target_did: &Did, account: &T::AccountId) -> bool {
            <ClaimIssuers<T>>::contains_key(target_did, account) && {
                let expiry = <ClaimIssuers<T>>::get(target_did, account).unwrap();
                let now = <timestamp::Module<T>>::get();
                expiry.saturating_mul(T::Moment::unique_saturated_from(1_000u32)) > now
            }
        }

        fn next_nonce() -> u64 {
            let nonce = <Nonce<T>>::get();
            <Nonce<T>>::put(nonce + 1u64);
            nonce
        }

        /// Creates a Did with given properties
        fn mint_did(
            caller: &T::AccountId,
            subject: &T::AccountId,
            controller: &T::AccountId,
            properties: Option<
                Vec<
                    DidProperty<
                        BoundedVec<u8, <T as Config>::NameLimit>,
                        BoundedVec<u8, <T as Config>::FactStringLimit>,
                    >,
                >,
            >,
        ) {
            let nonce = Self::next_nonce();
            let random = <randomness::Module<T>>::random(&b"mint_did context"[..]);
            let encoded = (random, subject, nonce).encode();
            let id = sp_io::hashing::blake2_256(&encoded);

            let did = Did { id };

            <DidBySubject<T>>::insert(subject, &did, ());
            <DidByController<T>>::insert(controller, &did, ());
            <DidControllers<T>>::insert(&did, controller, ());
            if *subject != *controller {
                <DidByController<T>>::insert(subject, &did, ());
                <DidControllers<T>>::insert(&did, subject, ());
            }

            let did_doc = DidDocument {
                subject: subject.clone(),
            };
            <DidDocuments<T>>::insert(&did, did_doc);

            if let Some(properties) = properties {
                properties.into_iter().for_each(|property| {
                    let hash = T::Hashing::hash_of(&property.name);
                    <DidDocumentProperties<T>>::insert(&did, &hash, property);
                });
            }

            Self::deposit_event(Event::Registered(
                caller.clone(),
                subject.clone(),
                controller.clone(),
                did,
            ));
        }
    }
    // -- for use in weights --

    macro_rules! max_fact_len {
        ($fact:expr,$max_fact_len:ident) => {{
            let fact_len = match &$fact {
                Fact::Bool(..) => 1u32,
                Fact::Text(string) => string.len() as u32,
                Fact::Attachment(_hash, filename) => 32u32 + (filename.len() as u32),
                Fact::Location(..) => 2u32,
                Fact::Did(..) => 32u32,
                Fact::Float(..) => 8u32,
                Fact::U8(..) => 1u32,
                Fact::U16(..) => 2u32,
                Fact::U32(..) => 4u32,
                Fact::U128(..) => 16u32,
                Fact::Date(..) => 4u32,
                Fact::Iso8601(..) => 17u32, //Timezone should be max 10 ?
            };
            if fact_len > $max_fact_len {
                $max_fact_len = fact_len;
            };
        }};
    }

    fn get_max_property_name_len_option(
        properties: &Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
    ) -> u32 {
        let mut max_property_name_len = 0;
        if let Some(properties) = properties.as_ref() {
            properties.iter().for_each(|property| {
                if property.name.len() as u32 > max_property_name_len {
                    max_property_name_len = property.name.len() as u32;
                };
            })
        }
        max_property_name_len
    }

    fn get_max_property_fact_len_option(
        properties: &Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
    ) -> u32 {
        let mut max_fact_len = 0;
        if let Some(properties) = properties.as_ref() {
            properties.iter().for_each(|property| {
                max_fact_len!(property.fact, max_fact_len);
            })
        }
        max_fact_len
    }

    fn get_max_property_name_len(properties: &[DidProperty<Vec<u8>, Vec<u8>>]) -> u32 {
        let mut max_property_name_len = 0;
        properties.iter().for_each(|property| {
            if property.name.len() as u32 > max_property_name_len {
                max_property_name_len = property.name.len() as u32;
            };
        });
        max_property_name_len
    }

    fn get_max_property_fact_len(properties: &[DidProperty<Vec<u8>, Vec<u8>>]) -> u32 {
        let mut max_fact_len = 0;
        properties.iter().for_each(|property| {
            max_fact_len!(property.fact, max_fact_len);
        });
        max_fact_len
    }

    fn get_max_statement_name_len(statements: &[Statement<Vec<u8>, Vec<u8>>]) -> u32 {
        let mut max_statement_name_len = 0;
        statements.iter().for_each(|statement| {
            if statement.name.len() as u32 > max_statement_name_len {
                max_statement_name_len = statement.name.len() as u32;
            };
        });
        max_statement_name_len
    }

    fn get_max_statement_fact_len(statements: &[Statement<Vec<u8>, Vec<u8>>]) -> u32 {
        let mut max_fact_len = 0;
        statements.iter().for_each(|statement| {
            max_fact_len!(statement.fact, max_fact_len);
        });
        max_fact_len
    }

    fn get_max_statement_name_bounded_len<T: Config>(
        statements: &[Statement<
            BoundedVec<u8, <T as Config>::NameLimit>,
            BoundedVec<u8, <T as Config>::FactStringLimit>,
        >],
    ) -> u32 {
        let mut max_statement_name_len = 0;
        statements.iter().for_each(|statement| {
            if statement.name.len() as u32 > max_statement_name_len {
                max_statement_name_len = statement.name.len() as u32;
            };
        });
        max_statement_name_len
    }

    fn get_max_statement_fact_bounded_len<T: Config>(
        statements: &[Statement<
            BoundedVec<u8, <T as Config>::NameLimit>,
            BoundedVec<u8, <T as Config>::FactStringLimit>,
        >],
    ) -> u32 {
        let mut max_fact_len = 0;
        statements.iter().for_each(|statement| {
            max_fact_len!(statement.fact, max_fact_len);
        });
        max_fact_len
    }

    fn get_max_key_len(keys: &[Vec<u8>]) -> u32 {
        let mut max_keys_len = 0;
        keys.iter().for_each(|key| {
            if key.len() as u32 > max_keys_len {
                max_keys_len = key.len() as u32;
            };
        });
        max_keys_len
    }

    fn get_did_for_bulk_lens<T: Config>(
        dids: &[(T::AccountId, Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>)],
    ) -> (u32, u32, u32) {
        fn div_up(a: u32, b: u32) -> u32 {
            a / b + (a % b != 0) as u32
        }

        let mut property_count_tot = 0;
        let mut property_name_tot = 0;
        let mut property_fact_tot = 0;
        let did_count = dids.len() as u32;
        dids.iter().for_each(|(_, properties_maybe)| {
            if let Some(properties) = properties_maybe.as_ref() {
                property_count_tot += properties.len() as u32;

                properties.iter().for_each(|property| {
                    property_name_tot += property.name.len() as u32;
                    let fact_len = match &property.fact {
                        Fact::Text(string) => string.len() as u32,
                        _ => 10, //give minimum of 10 and don't bother checking for anything other than Text
                    };
                    property_fact_tot += fact_len;
                })
            };
        });
        //avoid divide by zero errors
        if did_count == 0 {
            return (0, 0, 0);
        }

        let property_count_avg = div_up(property_count_tot, did_count);
        if property_count_tot == 0 {
            return (0, 0, property_count_avg);
        }

        let property_name_avg = div_up(property_name_tot, property_count_tot);
        let property_fact_avg = div_up(property_fact_tot, property_count_tot);

        (property_name_avg, property_fact_avg, property_count_avg)
    }
    fn min_max_check<V>(value: V, min: Option<u128>, max: Option<u128>) -> bool
    where
        V: Copy + Into<u128>,
    {
        let min_check = if let Some(min) = min {
            value.into() >= min
        } else {
            true
        };
        let max_check = if let Some(max) = max {
            value.into() <= max
        } else {
            true
        };
        min_check && max_check
    }
}
