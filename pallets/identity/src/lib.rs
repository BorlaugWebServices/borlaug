//! # DidCatalog Module
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
//!
//! #### For Controllers
//! * `register_did_for` - Registers a new DID for a subject and adds caller as a controller
//! * `add_did_properties` - Add properties to a DID Document
//! * `authorize_claim_consumer` - Grant permission to a claim consumer to add a claim
//! * `authorize_claim_verifier` - Grant permission to a claim verifier to attest a claim
//!
//! #### For Claim Consumers
//! * `set_fee` - Set the fee required to be paid for a judgement to be given by the registrar.
//! * `set_fields` - Set the fields that a registrar cares about in their judgements.
//! * `provide_judgement` - Provide a judgement to an identity.
//!
//! #### For Claim Verifiers
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

    use codec::Encode;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Randomness,
    };
    use frame_system::pallet_prelude::*;
    use primitives::{
        Attestation, Catalog, Claim, ClaimConsumer, ClaimIssuer, Did, DidDocument, DidProperty,
        Statement,
    };
    use sp_runtime::{
        traits::{AtLeast32Bit, CheckedAdd, One},
        DispatchResult,
    };
    use sp_std::prelude::*;

    /// A claim index.
    pub type ClaimIndex = u64;

    /// Short name associated with Did.
    pub type ShortName = Vec<u8>;

    /// Key used for DidProperty.
    pub type DidPropertyName = Vec<u8>;

    #[pallet::config]
    pub trait Config: frame_system::Config + timestamp::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type CatalogId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
    }

    #[pallet::event]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::Moment = "Moment",
        T::CatalogId = "CatalogId"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new DID was registered (Subject, Controller, DID)
        Registered(T::AccountId, T::AccountId, Did),
        /// Did updated (Controller, DID)
        DidUpdated(T::AccountId, Did),
        /// Did replaced (Controller, DID)
        DidReplaced(T::AccountId, Did),
        /// Claim consumers added
        ClaimConsumersAdded(Did, Vec<ClaimConsumer<T::Moment>>),
        /// Claim consumers removed
        ClaimConsumersRemoved(Did, Vec<Did>),
        /// Claim issuers added
        ClaimIssuersAdded(Did, Vec<ClaimIssuer<T::Moment>>),
        /// Claim issuers removed
        ClaimIssuersRemoved(Did, Vec<Did>),
        /// Claim was made against a DID (target DID, index of claim, claim proposer DID)
        ClaimMade(Did, ClaimIndex, Did),
        /// Claim was attested (target DID, index of claim, claim issuer DID)
        ClaimAttested(Did, ClaimIndex, Did),
        /// Claim attestation revoked (target DID, index of claim, claim issuer DID)
        ClaimAttestationRevoked(Did, ClaimIndex, Did),
        /// Catalog added (Owner Did, Catalog Id)
        CatalogCreated(Did, T::CatalogId),
        /// Catalog removed (Owner Did, Catalog Id)
        CatalogRemoved(Did, T::CatalogId),
        /// Dids added to catalog (Owner Did, Catalog Id)
        CatalogDidsAdded(Did, T::CatalogId),
        /// Dids removed from catalog (Owner Did, Catalog Id)
        CatalogDidsRemoved(Did, T::CatalogId),
        /// DID Controllers updated (Did, when Id)
        DidControllersUpdated(T::AccountId, Did),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value was None
        NoneValue,
        /// Value was not found
        NotFound,
        /// A non-controller account attempted to  modify a DID
        NotController,
        /// Not authorized to make a claim or attest a claim
        NotAuthorized,
        /// Id out of bounds
        NoIdAvailable,
    }
    //TODO: can we do these initializations in generate_store instead?
    #[pallet::type_value]
    pub fn UnitDefault<T: Config>() -> u64 {
        1u64
    }

    #[pallet::type_value]
    pub fn ZeroDefault<T: Config>() -> u64 {
        0u64
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

    /// An account can have multiple DIDs
    /// AccountId => Vec<Did>
    #[pallet::storage]
    #[pallet::getter(fn dids)]
    pub type DidCatalog<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Did>, OptionQuery>;

    /// A DID has a DID Document
    /// Did => DidDocument
    #[pallet::storage]
    #[pallet::getter(fn did_document)]
    pub type DidDocuments<T: Config> =
        StorageMap<_, Blake2_128Concat, Did, DidDocument<T::AccountId>, OptionQuery>;

    /// Controller for DIDs.
    /// Controller AccountId => Collection of DIDs
    #[pallet::storage]
    #[pallet::getter(fn controller)]
    pub type DidController<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Did>, OptionQuery>;

    /// The next available claim index, aka the number of claims started so far.
    #[pallet::storage]
    #[pallet::getter(fn claim_count)]
    pub type ClaimCount<T: Config> = StorageValue<_, ClaimIndex, ValueQuery, ZeroDefault<T>>;

    /// Claim consumers request a claim to offer protected services    
    /// Subject DID => DIDs of claim consumers
    #[pallet::storage]
    #[pallet::getter(fn claim_comsumers)]
    pub type ClaimConsumers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Did,
        Blake2_128Concat,
        Did,
        ClaimConsumer<T::Moment>,
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
        Did,
        ClaimIssuer<T::Moment>,
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
        ClaimIndex,
        Claim<T::Moment>,
        OptionQuery,
    >;

    /// The next available catalog index
    #[pallet::storage]
    #[pallet::getter(fn next_catalog_id)]
    pub type NextCatalogId<T: Config> =
        StorageValue<_, T::CatalogId, ValueQuery, CatalogIdDefault<T>>;

    /// Catalog ownership
    #[pallet::storage]
    #[pallet::getter(fn catalog_ownership)]
    pub type CatalogOwnership<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Did,
        Blake2_128Concat,
        T::CatalogId,
        Catalog,
        OptionQuery,
    >;

    /// Catalogs
    /// For each catalog index, we keep a mapping of `Did` an index name
    #[pallet::storage]
    #[pallet::getter(fn catalogs)]
    pub type Catalogs<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CatalogId,
        Blake2_128Concat,
        Did,
        Option<ShortName>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new DID for caller. Subject calls to create a new DID.
        ///
        /// # <weight>
        /// - O(1).
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_did(
            origin: OriginFor<T>,
            short_name: Option<Vec<u8>>,
            properties: Option<Vec<DidProperty>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            Self::mint_did(sender.clone(), sender, short_name, properties);
            Ok(().into())
        }

        /// Register a new DID for caller. Subject calls to create a new DID.
        ///
        /// # <weight>
        /// - O(1).
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_did_for(
            origin: OriginFor<T>,
            subject: T::AccountId,
            short_name: Option<Vec<u8>>,
            properties: Option<Vec<DidProperty>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            Self::mint_did(subject, sender, short_name, properties);
            Ok(().into())
        }

        /// Append given collection of Did properties provided the caller is a controller and/or update short_name
        ///
        /// Arguments:
        /// - `did` DID to which properties are to be added
        /// - `add_properties` DID properties to be added
        /// - `remove_keys` Keys of DID properties to be removed
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_did(
            origin: OriginFor<T>,
            did: Did,
            short_name: Option<Vec<u8>>,
            add_properties: Option<Vec<DidProperty>>,
            remove_keys: Option<Vec<DidPropertyName>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), did),
                Error::<T>::NotController
            );

            <DidDocuments<T>>::try_mutate_exists(&did, |maybe_did_doc| -> DispatchResult {
                let did_doc = maybe_did_doc.as_mut().ok_or(Error::<T>::NotFound)?;
                //TODO: we cannot delete short_name?
                if let Some(short_name) = short_name {
                    did_doc.short_name = Some(short_name);
                }
                if let Some(remove_keys) = remove_keys {
                    did_doc
                        .properties
                        .retain(|p| !remove_keys.contains(&p.name));
                }
                if let Some(mut add_properties) = add_properties {
                    did_doc.properties.append(&mut add_properties);
                }

                Ok(())
            })?;

            Self::deposit_event(Event::DidUpdated(sender, did));
            Ok(().into())
        }

        /// Replace given collection of Did properties provided the caller is a controller
        ///
        /// Arguments:
        /// - `did` DID to which properties are to be added
        /// - `properties` DID properties to be added
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn replace_did(
            origin: OriginFor<T>,
            did: Did,
            properties: Vec<DidProperty>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), did),
                Error::<T>::NotController
            );
            <DidDocuments<T>>::try_mutate_exists(&did, |maybe_did_doc| -> DispatchResult {
                let did_doc = maybe_did_doc.as_mut().ok_or(Error::<T>::NotFound)?;
                did_doc.properties = properties;
                Ok(())
            })?;

            Self::deposit_event(Event::DidReplaced(sender, did));
            Ok(().into())
        }

        /// Append given collection of Did properties provided the caller is a controller
        ///
        /// Arguments:
        /// - `did` subject
        /// - `add` DIDs to be added as controllers
        /// - `remove` DIDs to be removed as controllers
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn manage_controllers(
            origin: OriginFor<T>,
            did: Did,
            add: Option<Vec<T::AccountId>>,
            remove: Option<Vec<T::AccountId>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                Self::is_controller(sender.clone(), did),
                Error::<T>::NotController
            );
            <DidDocuments<T>>::try_mutate_exists(&did, |maybe_did_doc| -> DispatchResult {
                let did_doc = maybe_did_doc.as_mut().ok_or(Error::<T>::NotFound)?;
                if let Some(to_be_removed) = remove {
                    to_be_removed
                        .into_iter()
                        .map(|remove_account| {
                            <DidController<T>>::try_mutate_exists(
                                &remove_account,
                                |maybe_controlled| -> DispatchResult {
                                    let controlled =
                                        maybe_controlled.as_mut().ok_or(Error::<T>::NotFound)?;
                                    {
                                        controlled.retain(|c| *c != did);
                                        Ok(())
                                    }
                                },
                            )?;
                            did_doc.controllers.retain(|c| *c != remove_account);
                            Ok(())
                        })
                        .collect::<Result<Vec<_>, DispatchError>>()?;
                }

                if let Some(mut to_be_added) = add {
                    to_be_added
                        .iter()
                        .map(|add_account| {
                            <DidController<T>>::try_mutate_exists(
                                &add_account,
                                |maybe_controlled| -> DispatchResult {
                                    match maybe_controlled {
                                        Some(controlled) => controlled.push(did),
                                        None => <DidController<T>>::insert(&add_account, vec![did]),
                                    }
                                    Ok(())
                                },
                            )?;

                            Ok(())
                        })
                        .collect::<Result<Vec<_>, DispatchError>>()?;
                    did_doc.controllers.append(&mut to_be_added);
                    did_doc.controllers.dedup();
                }
                Ok(())
            })?;

            Self::deposit_event(Event::DidControllersUpdated(sender, did));
            Ok(().into())
        }

        /// Grants a claim consumer permission to write a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_consumers` DIDs of claim consumer
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn authorize_claim_consumers(
            origin: OriginFor<T>,
            target_did: Did,
            claim_consumers: Vec<ClaimConsumer<T::Moment>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender, target_did),
                Error::<T>::NotController
            );

            claim_consumers.iter().for_each(|claim_consumer| {
                if !<ClaimConsumers<T>>::contains_key(&target_did, &claim_consumer.consumer) {
                    <ClaimConsumers<T>>::insert(
                        &target_did,
                        &claim_consumer.consumer,
                        claim_consumer,
                    );
                }
            });

            Self::deposit_event(Event::ClaimConsumersAdded(target_did, claim_consumers));
            Ok(().into())
        }

        /// Revokes claim consumers permission to write a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_consumers` DIDs of claim consumers to be revoked
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn revoke_claim_consumers(
            origin: OriginFor<T>,
            target_did: Did,
            claim_consumers: Vec<Did>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender, target_did),
                Error::<T>::NotController
            );

            claim_consumers.iter().for_each(|claim_consumer| {
                <ClaimConsumers<T>>::remove(&target_did, claim_consumer);
            });

            Self::deposit_event(Event::ClaimConsumersRemoved(target_did, claim_consumers));
            Ok(().into())
        }

        /// Grants a claim attester permission to attest a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_issuers` DIDs of claim issuer
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn authorize_claim_issuers(
            origin: OriginFor<T>,
            target_did: Did,
            claim_issuers: Vec<ClaimIssuer<T::Moment>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender, target_did),
                Error::<T>::NotController
            );

            claim_issuers.iter().for_each(|claim_issuer| {
                if !<ClaimIssuers<T>>::contains_key(&target_did, &claim_issuer.issuer) {
                    <ClaimIssuers<T>>::insert(&target_did, &claim_issuer.issuer, claim_issuer);
                }
            });

            Self::deposit_event(Event::ClaimIssuersAdded(target_did, claim_issuers));
            Ok(().into())
        }

        /// Revokes claim issuers permission to attest a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_issuers` DIDs of claim issuers to be revoked
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn revoke_claim_issuers(
            origin: OriginFor<T>,
            target_did: Did,
            claim_issuers: Vec<Did>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender, target_did),
                Error::<T>::NotController
            );

            claim_issuers.iter().for_each(|claim_issuer| {
                <ClaimIssuers<T>>::remove(&target_did, claim_issuer);
            });

            Self::deposit_event(Event::ClaimIssuersRemoved(target_did, claim_issuers));
            Ok(().into())
        }

        /// Claim consumer calls this to make a new `claim` against `target_did`
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_consumer` DID of claim consumer
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn make_claim(
            origin: OriginFor<T>,
            claim_consumer: Did,
            target_did: Did,
            description: Vec<u8>,
            statements: Vec<Statement>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender, claim_consumer),
                Error::<T>::NotController
            );
            ensure!(
                Self::can_make_claim(target_did, claim_consumer),
                Error::<T>::NotAuthorized
            );

            let claim_index = Self::claim_count();
            <ClaimCount<T>>::put(claim_index + 1u64);

            let claim = Claim {
                description,
                statements,
                created_by: claim_consumer,
                attestation: None,
            };

            <Claims<T>>::insert(&target_did, claim_index, claim);

            Self::deposit_event(Event::ClaimMade(target_did, claim_index, claim_consumer));
            Ok(().into())
        }

        /// Claim issuer attests `claim_index` against `target_did` with given `statements`
        ///
        /// Arguments:
        /// - `claim_issuer` DID of claim issuer
        /// - `target_did` DID against which claims are to be attested
        /// - `claim_index` Claim to be attested
        /// - `statements` Claim issuer overwrites these statements
        /// - `valid_until` Attestation expires
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn attest_claim(
            origin: OriginFor<T>,
            claim_issuer: Did,
            target_did: Did,
            claim_index: ClaimIndex,
            statements: Vec<Statement>,
            valid_until: T::Moment,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender, claim_issuer),
                Error::<T>::NotController
            );
            ensure!(
                Self::can_attest_claim(target_did, claim_issuer),
                Error::<T>::NotAuthorized
            );

            <Claims<T>>::try_mutate_exists(
                &target_did,
                claim_index,
                |maybe_claim| -> DispatchResult {
                    let mut claim = maybe_claim.as_mut().ok_or(Error::<T>::NotFound)?;

                    let mut stmts = statements.clone();
                    let names = statements.into_iter().map(|s| s.name).collect::<Vec<_>>();

                    claim.statements.retain(|s| !names.contains(&s.name));
                    claim.statements.append(&mut stmts);
                    claim.attestation = Some(Attestation {
                        attested_by: claim_issuer,
                        valid_until,
                    });

                    Ok(())
                },
            )?;

            Self::deposit_event(Event::ClaimAttested(target_did, claim_index, claim_issuer));

            Ok(().into())
        }

        /// Claim issuer revokes `claim_index`
        ///
        /// Arguments:
        /// - `claim_issuer` DID of claim issuer
        /// - `target_did` DID against which claims are to be attested
        /// - `claim_index` Claim to be attested
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn revoke_attestation(
            origin: OriginFor<T>,
            claim_issuer: Did,
            target_did: Did,
            claim_index: ClaimIndex,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender, claim_issuer),
                Error::<T>::NotController
            );
            ensure!(
                Self::can_attest_claim(target_did, claim_issuer),
                Error::<T>::NotAuthorized
            );

            <Claims<T>>::try_mutate_exists(
                &target_did,
                claim_index,
                |maybe_claim| -> DispatchResult {
                    let mut claim = maybe_claim.as_mut().ok_or(Error::<T>::NotFound)?;
                    claim.attestation = None;
                    Ok(())
                },
            )?;

            Self::deposit_event(Event::ClaimAttestationRevoked(
                target_did,
                claim_index,
                claim_issuer,
            ));
            Ok(().into())
        }

        /// Add a new catalog
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_catalog(
            origin: OriginFor<T>,
            owner_did: Did,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender, owner_did),
                Error::<T>::NotController
            );

            let catalog_id = Self::next_catalog_id();
            let next_id = catalog_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextCatalogId<T>>::put(next_id);

            <CatalogOwnership<T>>::insert(owner_did, catalog_id, Catalog { name });

            Self::deposit_event(Event::CatalogCreated(owner_did, catalog_id));
            Ok(().into())
        }

        /// Remove a catalog
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `catalog_id` Catalog to be removed
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_catalog(
            origin: OriginFor<T>,
            owner_did: Did,
            catalog_id: T::CatalogId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender, owner_did),
                Error::<T>::NotController
            );

            <CatalogOwnership<T>>::remove(owner_did, catalog_id);
            <Catalogs<T>>::remove_prefix(&catalog_id);

            Self::deposit_event(Event::CatalogRemoved(owner_did, catalog_id));
            Ok(().into())
        }

        /// Add DIDs to a catalog
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `catalog_id` Catalog to which DID are to be added
        /// - `dids` DIDs are to be added
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn add_dids_to_catalog(
            origin: OriginFor<T>,
            owner_did: Did,
            catalog_id: T::CatalogId,
            dids: Vec<(Did, ShortName)>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender, owner_did),
                Error::<T>::NotController
            );

            for (did, short_name) in dids.into_iter() {
                <Catalogs<T>>::insert(catalog_id, did, Some(short_name));
            }

            Self::deposit_event(Event::CatalogDidsAdded(owner_did, catalog_id));
            Ok(().into())
        }

        /// Remove DIDs from a catalog
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `catalog_id` Catalog to which DID are to be removed
        /// - `dids` DIDs are to be removed
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_dids_from_catalog(
            origin: OriginFor<T>,
            owner_did: Did,
            catalog_id: T::CatalogId,
            dids: Vec<Did>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender, owner_did),
                Error::<T>::NotController
            );

            for did in dids.into_iter() {
                <Catalogs<T>>::remove(catalog_id, did);
            }

            Self::deposit_event(Event::CatalogDidsRemoved(owner_did, catalog_id));
            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --

        pub fn get_catalogs(owner_did: Did) -> Vec<(T::CatalogId, Catalog)> {
            let mut catalogs = Vec::new();
            <CatalogOwnership<T>>::iter_prefix(owner_did)
                .for_each(|(catalog_id, catalog)| catalogs.push((catalog_id, catalog)));
            catalogs
        }

        pub fn get_catalog(owner_did: Did, catalog_id: T::CatalogId) -> Option<Catalog> {
            <CatalogOwnership<T>>::get(owner_did, catalog_id)
        }

        pub fn get_dids_in_catalog(catalog_id: T::CatalogId) -> Vec<(Did, Option<Vec<u8>>)> {
            let mut dids = Vec::new();
            <Catalogs<T>>::iter_prefix(catalog_id).for_each(|(did, name)| dids.push((did, name)));
            dids
        }

        pub fn get_did_in_catalog(
            catalog_id: T::CatalogId,
            did: Did,
        ) -> Option<(Option<Vec<u8>>, DidDocument<T::AccountId>)> {
            <DidDocuments<T>>::get(did).map(|did_document| {
                let short_name = <Catalogs<T>>::get(catalog_id, did).flatten();
                (short_name, did_document)
            })
        }

        pub fn get_did(did: Did) -> Option<DidDocument<T::AccountId>> {
            <DidDocuments<T>>::get(did)
        }

        pub fn get_dids_by_subject(subject: T::AccountId) -> Vec<(Did, Option<Vec<u8>>)> {
            let dids = <DidCatalog<T>>::get(subject).unwrap_or_else(|| Vec::new());
            dids.into_iter()
                .map(|did| {
                    (
                        did,
                        <DidDocuments<T>>::get(did)
                            .map(|did_document| did_document.short_name)
                            .flatten(),
                    )
                })
                .collect()
        }

        pub fn get_dids_by_controller(controller: T::AccountId) -> Vec<(Did, Option<Vec<u8>>)> {
            let dids = <DidController<T>>::get(controller).unwrap_or_else(|| Vec::new());
            dids.into_iter()
                .map(|did| {
                    (
                        did,
                        <DidDocuments<T>>::get(did)
                            .map(|did_document| did_document.short_name)
                            .flatten(),
                    )
                })
                .collect()
        }

        // -- private functions --

        /// Returns true if a `account` can control `did`
        pub fn is_controller(account: T::AccountId, did: Did) -> bool {
            if <DidController<T>>::contains_key(&account) {
                let dids = <DidController<T>>::get(account).unwrap();
                dids.contains(&did)
            } else {
                false
            }
        }

        /// Returns true if a `claim_consumer` can make a claim against `target_did`
        pub fn can_make_claim(target_did: Did, claim_consumer: Did) -> bool {
            <ClaimConsumers<T>>::contains_key(target_did, claim_consumer)
        }

        /// Returns true if a `claim_issuer` can attest a claim against `target_did`
        pub fn can_attest_claim(target_did: Did, claim_issuer: Did) -> bool {
            <ClaimIssuers<T>>::contains_key(target_did, claim_issuer)
        }

        // -- private functions --

        fn next_nonce() -> u64 {
            let nonce = <Nonce<T>>::get();
            <Nonce<T>>::put(nonce + 1u64);
            nonce
        }

        /// Creates a Did with given properties
        fn mint_did(
            subject: T::AccountId,
            controller: T::AccountId,
            short_name: Option<Vec<u8>>,
            properties: Option<Vec<DidProperty>>,
        ) {
            let nonce = Self::next_nonce();
            let random = <randomness::Module<T>>::random(&b"mint_did context"[..]);
            let encoded = (random, subject.clone(), nonce).encode();
            let id = sp_io::hashing::blake2_256(&encoded);

            let did = Did { id };

            <DidCatalog<T>>::append(&subject, &did);
            <DidController<T>>::append(&controller, &did);
            let did_doc = DidDocument {
                short_name,
                subject: subject.clone(),
                controllers: vec![controller.clone()],
                properties: properties.unwrap_or_else(|| Vec::new()),
            };
            <DidDocuments<T>>::insert(&did, did_doc);

            Self::deposit_event(Event::Registered(subject, controller, did));
        }
    }
}
