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
//! * `rename_catalog` - Rename a Catalog
//! * `remove_catalog` - Remove a Catalog
//! * `add_dids_to_catalog` - Add dids to a Catalog
//! * `rename_did_in_catalog` - Change the label of a DID in a catalog
//! * `remove_dids_from_catalog` - Remove a DID from in a catalog
//!
//! #### For Controllers
//! * `register_did_for` - Registers a new DID for a subject and adds caller as a controller
//! * `update_did` - Update properties of a DID Document
//! * `replace_did` - Replace all the properties of a DID Document
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
//! * `get_catalog` - Get a catalog name.
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

    const MODULE_INDEX: u8 = 2;

    #[repr(u8)]
    pub enum ExtrinsicIndex {
        Catalog = 21,
        Did = 22,
        Claim = 23,
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

        /// The maximum number of properties a DID may have
        #[pallet::constant]
        type PropertyLimit: Get<u32>;

        /// The maximum number of statements a Claim may have
        #[pallet::constant]
        type StatementLimit: Get<u32>;

        //TODO: if we add remove_did option then be sure to deal with cleanup of supporting storage carefully.
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
    }

    #[pallet::event]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::Moment = "Moment",
        T::CatalogId = "CatalogId",
        T::ClaimId = "ClaimId",      
        Vec<ClaimConsumer<T::AccountId, T::Moment>> = "ClaimConsumers",
        Vec<ClaimIssuer<T::AccountId, T::Moment>> = "ClaimIssuers",
        Vec<T::AccountId> = "AccountIds",
        Option<Vec<T::AccountId>> = "Option<AccountIds>"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new DID was registered (Subject, Controller, DID)
        Registered(T::AccountId, T::AccountId, Did),
        /// A DIDs were bulk registered (Controller, count)
        BulkRegistered(T::AccountId, u32),
        /// Did updated (Controller, DID)
        DidUpdated(T::AccountId, Did),
        /// Did replaced (Controller, DID)
        DidReplaced(T::AccountId, Did),
        /// Claim consumers added
        ClaimConsumersAdded(Did, Vec<ClaimConsumer<T::AccountId, T::Moment>>),
        /// Claim consumers removed
        ClaimConsumersRemoved(Did, Vec<T::AccountId>),
        /// Claim issuers added
        ClaimIssuersAdded(Did, Vec<ClaimIssuer<T::AccountId, T::Moment>>),
        /// Claim issuers removed
        ClaimIssuersRemoved(Did, Vec<T::AccountId>),
        /// Claim was made against a DID (target DID, index of claim, group_id)
        ClaimMade(Did, T::ClaimId, T::AccountId),
        /// Claim was attested (target DID, index of claim, group_id)
        ClaimAttested(Did, T::ClaimId, T::AccountId),
        /// Claim attestation revoked (target DID, index of claim, group_id)
        ClaimAttestationRevoked(Did, T::ClaimId, T::AccountId),
        /// Catalog added (account, Catalog Id)
        CatalogCreated(T::AccountId, T::CatalogId),
        /// Catalog removed (account, Catalog Id)
        CatalogRemoved(T::AccountId, T::CatalogId),
        /// Dids added to catalog (account, Catalog Id)
        CatalogDidsAdded(T::AccountId, T::CatalogId),
        /// Did short_name in catalog changed (account, catalog_id, did)
        CatalogDidRenamed(T::AccountId, T::CatalogId, Did),
        /// Dids removed from catalog (account, catalog_id)
        CatalogDidsRemoved(T::AccountId, T::CatalogId),
        /// DID Controller updated (account, target_did, added_controllers,removed_controllers)
        DidControllerUpdated(
            T::AccountId,
            Did,
            Option<Vec<T::AccountId>>,
            Option<Vec<T::AccountId>>,
        ),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value was None
        NoneValue,
        /// A string exceeds the maximum allowed length
        BadString,
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
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

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
    pub type DidDocuments<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Did,
        DidDocument<T::AccountId, BoundedVec<u8, <T as Config>::NameLimit>>,
        OptionQuery,
    >;

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

    /// Catalogs
    #[pallet::storage]
    #[pallet::getter(fn catalogs)]
    pub type Catalogs<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::CatalogId,
        Catalog<BoundedVec<u8, <T as Config>::NameLimit>>,
        OptionQuery,
    >;

    /// DidsByCatalog
    /// For each catalog index, we keep a mapping of `Did` an index name
    #[pallet::storage]
    #[pallet::getter(fn dids_by_catalog)]
    pub type DidsByCatalog<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CatalogId,
        Blake2_128Concat,
        Did,
        BoundedVec<u8, <T as Config>::NameLimit>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new DID for caller. Subject calls to create a new DID.
        ///   
        /// Arguments:
        /// - `short_name` an optional short name for the DID
        /// - `properties` initial DID properties to be added to the new DID
        #[pallet::weight(<T as Config>::WeightInfo::register_did(
            short_name.as_ref().map_or(0,|name|name.len()) as u32,
            get_max_property_name_len_option(properties),
            get_max_property_fact_len_option(properties),
            properties.as_ref().map_or(0,|properties|properties.len()) as u32,
        ))]
        pub fn register_did(
            origin: OriginFor<T>,
            short_name: Option<Vec<u8>>,
            properties: Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let bounded_name = enforce_limit_option!(short_name.clone());

            let property_count = properties.as_ref().map_or(0, |p| p.len());

            ensure!(
                property_count < T::PropertyLimit::get() as usize,
                Error::<T>::PropertyLimitExceeded
            );

            let properties = enforce_limit_did_properties_option!(properties);

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Did as u8),
                &sender,
            );

            Self::mint_did(sender.clone(), sender, bounded_name, properties);

            Ok(().into())
        }

        /// Register a new DID on behalf of another user (subject).
        ///        
        /// Arguments:
        /// - `subject` the Subject of the DID
        /// - `short_name` an optional short name for the DID
        /// - `properties` initial DID properties to be added to the new DID
        #[pallet::weight(<T as Config>::WeightInfo::register_did(
            short_name.as_ref().map_or(0,|name|name.len()) as u32,
            get_max_property_name_len_option(properties),
            get_max_property_fact_len_option(properties),
            properties.as_ref().map_or(0,|properties|properties.len()) as u32,
        ))]
        pub fn register_did_for(
            origin: OriginFor<T>,
            subject: T::AccountId,
            short_name: Option<Vec<u8>>,
            properties: Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let bounded_name = enforce_limit_option!(short_name);

            let property_count = properties.as_ref().map_or(0, |p| p.len());

            ensure!(
                property_count < T::PropertyLimit::get() as usize,
                Error::<T>::PropertyLimitExceeded
            );

            let properties = enforce_limit_did_properties_option!(properties);

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Did as u8),
                &sender,
            );

            Self::mint_did(subject, sender, bounded_name, properties);
            Ok(().into())
        }

        /// Register a number of new DIDs on behalf of users (subject). Note the client should be smart and avoid exceeding block weight and size limits.
        ///        
        /// Arguments:
        /// - `dids` the DIDs to be created
        #[pallet::weight({
            let (a,b,c,d)=get_did_for_bulk_lens::<T>(dids);
            <T as Config>::WeightInfo::register_did_for(a,b,c,d).saturating_mul(dids.len() as Weight)
        })]
        pub fn register_did_for_bulk(
            origin: OriginFor<T>,
            dids: Vec<(
                T::AccountId,
                Option<Vec<u8>>,
                Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
            )>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let did_count = dids.len();

            ensure!(
                did_count < T::BulkDidLimit::get() as usize,
                Error::<T>::BulkDidLimitExceeded
            );

            let dids = dids
                .into_iter()
                .map(|(subject, short_name, properties)| {
                    let bounded_name = enforce_limit_option!(short_name);
                    let property_count = properties.as_ref().map_or(0, |p| p.len());

                    ensure!(
                        property_count <= T::PropertyLimit::get() as usize,
                        Error::<T>::PropertyLimitExceeded
                    );

                    let properties = enforce_limit_did_properties_option!(properties);

                    Ok((subject, bounded_name, properties))
                })
                .collect::<Result<Vec<_>, Error<T>>>()?;

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(did_count as u8 * ExtrinsicIndex::Did as u8),
                &sender,
            );

            dids.into_iter()
                .for_each(|(subject, bounded_name, properties)| {
                    Self::mint_did(subject, sender.clone(), bounded_name, properties);
                });

            Self::deposit_event(Event::BulkRegistered(sender, did_count as u32));

            Ok(().into())
        }

        /// Append given collection of Did properties provided the caller is a controller and/or update short_name
        ///
        /// Arguments:
        /// - `did` DID to which properties are to be added
        /// - `add_properties` DID properties to be added
        /// - `remove_keys` Keys of DID properties to be removed
        #[pallet::weight(<T as Config>::WeightInfo::update_did(
            short_name.as_ref().map_or(0,|name|name.as_ref().map_or(0,|name|name.len())) as u32,
            get_max_property_name_len_option(add_properties),
            get_max_property_fact_len_option(add_properties),
            add_properties.as_ref().map_or(0,|properties|properties.len()) as u32,
            get_max_key_len(remove_keys),
            remove_keys.as_ref().map_or(0,|keys|keys.len()) as u32,
        ))]
        pub fn update_did(
            origin: OriginFor<T>,
            did: Did,
            short_name: Option<Option<Vec<u8>>>,
            add_properties: Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
            remove_keys: Option<Vec<Vec<u8>>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let bounded_short_name = match short_name {
                Some(short_name) => Some(enforce_limit_option!(short_name)),
                None => None,
            };

            let add_properties_count = add_properties.as_ref().map_or(0, |p| p.len());

            ensure!(
                add_properties_count < T::PropertyLimit::get() as usize,
                Error::<T>::PropertyLimitExceeded
            );

            let remove_keys_count = remove_keys.as_ref().map_or(0, |p| p.len());

            ensure!(
                remove_keys_count < T::PropertyLimit::get() as usize,
                Error::<T>::PropertyLimitExceeded
            );

            let add_properties = enforce_limit_did_properties_option!(add_properties);

            let remove_keys = remove_keys
                .map(|remove_keys| {
                    remove_keys
                        .into_iter()
                        .map(|remove_key| Ok(enforce_limit!(remove_key)))
                        .collect::<Result<Vec<_>, Error<T>>>()
                })
                .map_or(Ok(None), |r| r.map(Some))?;

            ensure!(
                <DidByController<T>>::contains_key(&sender, &did),
                Error::<T>::NotController
            );

            <DidDocuments<T>>::mutate_exists(&did, |maybe_did_doc| {
                if let Some(ref mut did_doc) = maybe_did_doc {
                    if let Some(bounded_short_name) = bounded_short_name {
                        did_doc.short_name = bounded_short_name;
                    }
                }
            });

            if let Some(remove_keys) = remove_keys {
                remove_keys.into_iter().for_each(|remove_key| {
                    let hash = T::Hashing::hash_of(&remove_key);
                    <DidDocumentProperties<T>>::remove(&did, &hash);
                });
            }
            if let Some(add_properties) = add_properties {
                add_properties.into_iter().for_each(|add_property| {
                    let hash = T::Hashing::hash_of(&add_property.name);
                    <DidDocumentProperties<T>>::insert(&did, &hash, add_property);
                });
            }

            Self::deposit_event(Event::DidUpdated(sender, did));
            Ok(().into())
        }

        /// Replace given collection of Did properties provided the caller is a controller
        ///
        /// Arguments:
        /// - `did` DID to which properties are to be added
        /// - `properties` DID properties to be added
        #[pallet::weight(<T as Config>::WeightInfo::replace_did(
            get_max_property_name_len(properties),
            get_max_property_fact_len(properties),
            properties.len() as u32,
            <T as Config>::PropertyLimit::get() //We don't know how many properties exist.
        ))]
        pub fn replace_did(
            origin: OriginFor<T>,
            did: Did,
            properties: Vec<DidProperty<Vec<u8>, Vec<u8>>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <DidByController<T>>::contains_key(&sender, &did),
                Error::<T>::NotController
            );

            ensure!(
                properties.len() < T::PropertyLimit::get() as usize,
                Error::<T>::PropertyLimitExceeded
            );

            let properties = enforce_limit_did_properties!(properties);

            <DidDocumentProperties<T>>::remove_prefix(&did);

            properties.into_iter().for_each(|property| {
                let hash = T::Hashing::hash_of(&property.name);
                <DidDocumentProperties<T>>::insert(&did, &hash, property);
            });

            Self::deposit_event(Event::DidReplaced(sender, did));
            //TODO: consider measuring how many properties were removed, and refund weight accordingly.
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
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <DidByController<T>>::contains_key(&sender, &target_did),
                Error::<T>::NotController
            );

            let add_count = add.as_ref().map_or(0, |a| a.len());
            let remove_count = remove.as_ref().map_or(0, |a| a.len());

            ensure!(
                add_count < T::ControllerLimit::get() as usize
                    && remove_count < T::ControllerLimit::get() as usize,
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
            Self::deposit_event(Event::DidControllerUpdated(sender, target_did, add, remove));
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
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <DidByController<T>>::contains_key(&sender, &target_did),
                Error::<T>::NotController
            );

            ensure!(
                claim_consumers.len() < T::ClaimConsumerLimit::get() as usize,
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

            Self::deposit_event(Event::ClaimConsumersAdded(target_did, claim_consumers));
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
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <DidByController<T>>::contains_key(&sender, &target_did),
                Error::<T>::NotController
            );

            ensure!(
                claim_consumers.len() < T::ClaimConsumerLimit::get() as usize,
                Error::<T>::ClaimConsumerLimitExceeded
            );

            claim_consumers.iter().for_each(|claim_consumer| {
                <ClaimConsumers<T>>::remove(&target_did, claim_consumer);
                <DidsByConsumer<T>>::remove(claim_consumer, &target_did);
            });

            Self::deposit_event(Event::ClaimConsumersRemoved(target_did, claim_consumers));
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
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <DidByController<T>>::contains_key(&sender, &target_did),
                Error::<T>::NotController
            );

            ensure!(
                claim_issuers.len() < T::ClaimIssuerLimit::get() as usize,
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

            Self::deposit_event(Event::ClaimIssuersAdded(target_did, claim_issuers));
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
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <DidByController<T>>::contains_key(&sender, &target_did),
                Error::<T>::NotController
            );

            ensure!(
                claim_issuers.len() < T::ClaimIssuerLimit::get() as usize,
                Error::<T>::ClaimIssuerLimitExceeded
            );

            claim_issuers.iter().for_each(|claim_issuer| {
                <ClaimIssuers<T>>::remove(&target_did, claim_issuer);
                <DidsByIssuer<T>>::remove(claim_issuer, &target_did);
            });

            Self::deposit_event(Event::ClaimIssuersRemoved(target_did, claim_issuers));
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
            let sender = ensure_account_or_group!(origin);

            ensure!(
                Self::is_valid_consumer(&target_did, &sender),
                Error::<T>::NotAuthorized
            );

            ensure!(
                statements.len() < T::StatementLimit::get() as usize,
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
                created_by: sender.clone(),
                attestation: None,
                threshold,
            };

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Claim as u8),
                &sender,
            );

            let claim_id = next_id!(NextClaimId<T>, T);

            <Claims<T>>::insert(&target_did, claim_id, claim);

            Self::deposit_event(Event::ClaimMade(target_did, claim_id, sender));
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
            let either = T::GroupsOriginAccountOrApproved::ensure_origin(origin)?;
            let (sender, yes_votes) = match either {
                Either::Left(account_id) => (account_id, None),
                Either::Right((_, _, yes_votes, _, group_account)) => (group_account, yes_votes),
            };

            ensure!(
                Self::is_valid_issuer(&target_did, &sender),
                Error::<T>::NotAuthorized
            );

            ensure!(
                statements.len() < T::StatementLimit::get() as usize,
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
                        attested_by: sender.clone(),
                        valid_until,
                    });
                }
            });

            Self::deposit_event(Event::ClaimAttested(target_did, claim_id, sender));

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
            let sender = ensure_account_or_group!(origin);

            ensure!(
                Self::is_valid_issuer(&target_did, &sender),
                Error::<T>::NotAuthorized
            );

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

            Self::deposit_event(Event::ClaimAttestationRevoked(target_did, claim_id, sender));
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
        #[pallet::weight(<T as Config>::WeightInfo::create_catalog(
            name.len() as u32
        ))]
        pub fn create_catalog(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let bounded_name = enforce_limit!(name);

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Catalog as u8),
                &sender,
            );

            let catalog_id = next_id!(NextCatalogId<T>, T);

            <Catalogs<T>>::insert(&sender, catalog_id, Catalog { name: bounded_name });

            Self::deposit_event(Event::CatalogCreated(sender, catalog_id));
            Ok(().into())
        }

        /// Rename a catalog
        ///
        /// Arguments:
        /// - `catalog_id` id of catalog
        /// - `name` new name of catalog
        #[pallet::weight(<T as Config>::WeightInfo::rename_catalog(
            name.len() as u32
        ))]
        pub fn rename_catalog(
            origin: OriginFor<T>,
            catalog_id: T::CatalogId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Catalogs<T>>::contains_key(&sender, catalog_id),
                Error::<T>::NotController
            );

            let bounded_name = enforce_limit!(name);

            <Catalogs<T>>::insert(&sender, catalog_id, Catalog { name: bounded_name });

            Self::deposit_event(Event::CatalogCreated(sender, catalog_id));
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
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Catalogs<T>>::contains_key(&sender, catalog_id),
                Error::<T>::NotController
            );

            <Catalogs<T>>::remove(&sender, catalog_id);
            //TODO: fix this for weights
            <DidsByCatalog<T>>::remove_prefix(&catalog_id);

            Self::deposit_event(Event::CatalogRemoved(sender, catalog_id));
            Ok(().into())
        }

        /// Add DIDs to a catalog
        ///
        /// Arguments:       
        /// - `catalog_id` Catalog to which DID are to be added
        /// - `dids` DIDs are to be added
        #[pallet::weight(<T as Config>::WeightInfo::add_dids_to_catalog(
            dids.len() as u32,
            get_max_did_short_name_len(dids)
        ))]
        pub fn add_dids_to_catalog(
            origin: OriginFor<T>,
            catalog_id: T::CatalogId,
            dids: Vec<(Did, Vec<u8>)>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Catalogs<T>>::contains_key(&sender, catalog_id),
                Error::<T>::NotController
            );

            ensure!(
                dids.len() < T::CatalogDidLimit::get() as usize,
                Error::<T>::CatalogDidLimitExceeded
            );

            let dids = dids
                .into_iter()
                .map(|(did, short_name)| Ok((did, enforce_limit!(short_name))))
                .collect::<Result<Vec<_>, Error<T>>>()?;

            for (did, short_name) in dids.into_iter() {
                <DidsByCatalog<T>>::insert(catalog_id, did, short_name);
            }

            Self::deposit_event(Event::CatalogDidsAdded(sender, catalog_id));
            Ok(().into())
        }

        /// Rename DID in catalog. Changes the short_name stored in the catalog.
        ///
        /// Arguments:       
        /// - `catalog_id` Catalog if DID
        /// - `target_did` DID to be renamed
        /// - `short_name` new short_name of did in catalog
        #[pallet::weight(<T as Config>::WeightInfo::rename_did_in_catalog(
            short_name.len() as u32,
        ))]
        pub fn rename_did_in_catalog(
            origin: OriginFor<T>,
            catalog_id: T::CatalogId,
            target_did: Did,
            short_name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Catalogs<T>>::contains_key(&sender, catalog_id),
                Error::<T>::NotController
            );

            let bounded_name = enforce_limit!(short_name);

            <DidsByCatalog<T>>::insert(&catalog_id, &target_did, bounded_name);

            Self::deposit_event(Event::CatalogDidRenamed(sender, catalog_id, target_did));
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
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Catalogs<T>>::contains_key(&sender, catalog_id),
                Error::<T>::NotController
            );

            ensure!(
                dids.len() < T::CatalogDidLimit::get() as usize,
                Error::<T>::CatalogDidLimitExceeded
            );

            for did in dids.into_iter() {
                <DidsByCatalog<T>>::remove(catalog_id, did);
            }

            Self::deposit_event(Event::CatalogDidsRemoved(sender, catalog_id));
            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --

        pub fn get_catalogs(
            account_id: T::AccountId,
        ) -> Vec<(
            T::CatalogId,
            Catalog<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut catalogs = Vec::new();
            <Catalogs<T>>::iter_prefix(account_id)
                .for_each(|(catalog_id, catalog)| catalogs.push((catalog_id, catalog)));
            catalogs
        }

        pub fn get_catalog(
            account_id: T::AccountId,
            catalog_id: T::CatalogId,
        ) -> Option<Catalog<BoundedVec<u8, <T as Config>::NameLimit>>> {
            <Catalogs<T>>::get(account_id, catalog_id)
        }

        pub fn get_dids_in_catalog(
            catalog_id: T::CatalogId,
        ) -> Vec<(Did, BoundedVec<u8, <T as Config>::NameLimit>)> {
            let mut dids = Vec::new();
            <DidsByCatalog<T>>::iter_prefix(catalog_id)
                .for_each(|(did, name)| dids.push((did, name.into())));
            dids
        }

        pub fn get_did_in_catalog(
            catalog_id: T::CatalogId,
            did: Did,
        ) -> Option<(
            BoundedVec<u8, <T as Config>::NameLimit>,
            DidDocument<T::AccountId, BoundedVec<u8, <T as Config>::NameLimit>>,
            Vec<
                DidProperty<
                    BoundedVec<u8, <T as Config>::NameLimit>,
                    BoundedVec<u8, <T as Config>::FactStringLimit>,
                >,
            >,
            Vec<T::AccountId>,
        )> {
            let short_name = <DidsByCatalog<T>>::get(catalog_id, did);
            short_name.and_then(|short_name| {
                <DidDocuments<T>>::get(did).map(|did_document| {
                    let mut properties = Vec::new();
                    <DidDocumentProperties<T>>::iter_prefix(&did)
                        .for_each(|(_hash, property)| properties.push(property));
                    let mut controllers = Vec::new();
                    <DidControllers<T>>::iter_prefix(&did)
                        .for_each(|(controller, _)| controllers.push(controller));
                    (short_name, did_document, properties, controllers)
                })
            })
        }

        pub fn get_did(
            did: Did,
        ) -> Option<(
            DidDocument<T::AccountId, BoundedVec<u8, <T as Config>::NameLimit>>,
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

        pub fn get_dids_by_subject(
            subject: T::AccountId,
        ) -> Vec<(Did, Option<BoundedVec<u8, <T as Config>::NameLimit>>)> {
            let mut did_documents = Vec::new();
            <DidBySubject<T>>::iter_prefix(subject).for_each(|(did, _)| {
                did_documents.push((
                    did,
                    <DidDocuments<T>>::get(&did)
                        .map(|did_document| did_document.short_name)
                        .flatten(),
                ))
            });
            did_documents
        }

        pub fn get_dids_by_controller(
            controller: T::AccountId,
        ) -> Vec<(Did, Option<BoundedVec<u8, <T as Config>::NameLimit>>)> {
            let mut did_documents = Vec::new();
            <DidByController<T>>::iter_prefix(controller).for_each(|(did, _)| {
                did_documents.push((
                    did,
                    <DidDocuments<T>>::get(did)
                        .map(|did_document| did_document.short_name)
                        .flatten(),
                ))
            });
            did_documents
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
            <DidsByConsumer<T>>::iter_prefix(account).for_each(|(did, expiry)| {
                if <Claims<T>>::iter_prefix(did).next().is_none() {
                    dids.push((did, expiry))
                }
            });
            dids
        }

        pub fn get_outstanding_attestations(account: T::AccountId) -> Vec<(Did, T::Moment)> {
            let mut dids = Vec::new();
            <DidsByIssuer<T>>::iter_prefix(account).for_each(|(did, expiry)| {
                if <Claims<T>>::iter_prefix(did)
                    .find(|(_, claim)| claim.attestation.is_some())
                    .is_none()
                {
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
            subject: T::AccountId,
            controller: T::AccountId,
            short_name: Option<BoundedVec<u8, <T as Config>::NameLimit>>,
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
            let encoded = (random, subject.clone(), nonce).encode();
            let id = sp_io::hashing::blake2_256(&encoded);

            let did = Did { id };

            <DidBySubject<T>>::insert(&subject, &did, ());
            <DidByController<T>>::insert(&controller, &did, ());
            <DidControllers<T>>::insert(&did, &controller, ());
            if subject != controller {
                <DidByController<T>>::insert(&subject, &did, ());
                <DidControllers<T>>::insert(&did, &subject, ());
            }

            let did_doc = DidDocument {
                short_name,
                subject: subject.clone(),
            };
            <DidDocuments<T>>::insert(&did, did_doc);

            if let Some(properties) = properties {
                properties.into_iter().for_each(|property| {
                    let hash = T::Hashing::hash_of(&property.name);
                    <DidDocumentProperties<T>>::insert(&did, &hash, property);
                });
            }

            Self::deposit_event(Event::Registered(subject, controller, did));
        }
    }
    // -- for use in weights --

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

    fn get_max_property_name_len_option(
        properties: &Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
    ) -> u32 {
        let mut max_property_name_len = 0;
        properties.as_ref().and_then(|properties| {
            Some({
                properties.into_iter().for_each(|property| {
                    if property.name.len() as u32 > max_property_name_len {
                        max_property_name_len = property.name.len() as u32;
                    };
                })
            })
        });
        max_property_name_len
    }

    fn get_max_property_fact_len_option(
        properties: &Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
    ) -> u32 {
        let mut max_fact_len = 0;
        properties.as_ref().and_then(|properties| {
            Some({
                properties.into_iter().for_each(|property| {
                    max_fact_len!(property.fact, max_fact_len);
                })
            })
        });
        max_fact_len
    }

    fn get_max_property_name_len(properties: &Vec<DidProperty<Vec<u8>, Vec<u8>>>) -> u32 {
        let mut max_property_name_len = 0;
        properties.into_iter().for_each(|property| {
            if property.name.len() as u32 > max_property_name_len {
                max_property_name_len = property.name.len() as u32;
            };
        });
        max_property_name_len
    }

    fn get_max_property_fact_len(properties: &Vec<DidProperty<Vec<u8>, Vec<u8>>>) -> u32 {
        let mut max_fact_len = 0;
        properties.into_iter().for_each(|property| {
            max_fact_len!(property.fact, max_fact_len);
        });
        max_fact_len
    }

    fn get_max_statement_name_len(statements: &Vec<Statement<Vec<u8>, Vec<u8>>>) -> u32 {
        let mut max_statement_name_len = 0;
        statements.into_iter().for_each(|statement| {
            if statement.name.len() as u32 > max_statement_name_len {
                max_statement_name_len = statement.name.len() as u32;
            };
        });
        max_statement_name_len
    }

    fn get_max_statement_fact_len(statements: &Vec<Statement<Vec<u8>, Vec<u8>>>) -> u32 {
        let mut max_fact_len = 0;
        statements.into_iter().for_each(|statement| {
            max_fact_len!(statement.fact, max_fact_len);
        });
        max_fact_len
    }

    fn get_max_statement_name_bounded_len<T: Config>(
        statements: &Vec<
            Statement<
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
        >,
    ) -> u32 {
        let mut max_statement_name_len = 0;
        statements.into_iter().for_each(|statement| {
            if statement.name.len() as u32 > max_statement_name_len {
                max_statement_name_len = statement.name.len() as u32;
            };
        });
        max_statement_name_len
    }

    fn get_max_statement_fact_bounded_len<T: Config>(
        statements: &Vec<
            Statement<
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
        >,
    ) -> u32 {
        let mut max_fact_len = 0;
        statements.into_iter().for_each(|statement| {
            max_fact_len!(statement.fact, max_fact_len);
        });
        max_fact_len
    }

    fn get_max_key_len(keys: &Option<Vec<Vec<u8>>>) -> u32 {
        let mut max_keys_len = 0;
        keys.as_ref().and_then(|keys| {
            Some({
                keys.into_iter().for_each(|key| {
                    if key.len() as u32 > max_keys_len {
                        max_keys_len = key.len() as u32;
                    };
                })
            })
        });
        max_keys_len
    }
    fn get_max_did_short_name_len(catalog_dids: &Vec<(Did, Vec<u8>)>) -> u32 {
        let mut max_did_short_name_len = 0;

        catalog_dids.into_iter().for_each(|(_, short_name)| {
            if short_name.len() as u32 > max_did_short_name_len {
                max_did_short_name_len = short_name.len() as u32;
            };
        });
        max_did_short_name_len
    }

    fn get_did_for_bulk_lens<T: Config>(
        dids: &Vec<(
            T::AccountId,
            Option<Vec<u8>>,
            Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
        )>,
    ) -> (u32, u32, u32, u32) {
        fn div_up(a: u32, b: u32) -> u32 {
            a / b + (a % b != 0) as u32
        }
        let mut short_name_tot = 0;
        let mut property_count_tot = 0;
        let mut property_name_tot = 0;
        let mut property_fact_tot = 0;
        let did_count = dids.len() as u32;
        dids.iter()
            .for_each(|(_, short_name_maybe, properties_maybe)| {
                short_name_tot =
                    short_name_tot + short_name_maybe.as_ref().map_or(0, |name| name.len()) as u32;

                if let Some(properties) = properties_maybe.as_ref() {
                    property_count_tot = property_count_tot + properties.len() as u32;

                    properties.into_iter().for_each(|property| {
                        property_name_tot = property_name_tot + property.name.len() as u32;
                        let fact_len = match &property.fact {
                            Fact::Text(string) => string.len() as u32,
                            _ => 10, //give minimum of 10 and don't bother checking for anything other than Text
                        };
                        property_fact_tot = property_fact_tot + fact_len;
                    })
                };
            });
        //avoid divide by zero errors
        if did_count == 0 {
            return (0, 0, 0, 0);
        }
        let short_name_avg = div_up(short_name_tot, did_count);
        let property_count_avg = div_up(property_count_tot, did_count);
        if property_count_tot == 0 {
            return (short_name_avg, 0, 0, property_count_avg);
        }
        let property_name_avg = div_up(property_name_tot, property_count_tot);
        let property_fact_avg = div_up(property_fact_tot, property_count_tot);

        (
            short_name_avg,
            property_name_avg,
            property_fact_avg,
            property_count_avg,
        )
    }
}
