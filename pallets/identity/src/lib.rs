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
    use core::convert::TryInto;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Randomness,
    };
    use frame_system::pallet_prelude::*;
    use primitives::{bounded_vec::BoundedVec, *};
    use sp_runtime::{
        traits::{AtLeast32Bit, CheckedAdd, Hash, One},
        DispatchResult, Either,
    };
    use sp_std::prelude::*;
    #[pallet::config]
    pub trait Config: frame_system::Config + timestamp::Config + groups::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type CatalogId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;

        type ClaimId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;

        /// The maximum length of a name or symbol stored on-chain.
        type NameLimit: Get<u32>;

        /// The maximum length of a name or symbol stored on-chain.
        type FactStringLimit: Get<u32>;

        /// The maximum number of properties a DID may have
        type PropertyLimit: Get<u32>;

        /// The maximum number of statements a Claim may have
        type StatementLimit: Get<u32>;
    }

    #[pallet::event]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::Moment = "Moment",
        T::CatalogId = "CatalogId",
        T::ClaimId = "ClaimId",
        T::GroupId = "GroupId",
        Vec<T::GroupId> = "GroupIds",
        Vec<ClaimConsumer<T::GroupId, T::Moment>> = "ClaimConsumers",
        Vec<ClaimIssuer<T::GroupId, T::Moment>> = "ClaimIssuers",
        Option<Vec<T::AccountId>> = "Option<AccountIds>"
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
        ClaimConsumersAdded(Did, Vec<ClaimConsumer<T::GroupId, T::Moment>>),
        /// Claim consumers removed
        ClaimConsumersRemoved(Did, Vec<T::GroupId>),
        /// Claim issuers added
        ClaimIssuersAdded(Did, Vec<ClaimIssuer<T::GroupId, T::Moment>>),
        /// Claim issuers removed
        ClaimIssuersRemoved(Did, Vec<T::GroupId>),
        /// Claim was made against a DID (target DID, index of claim, group_id)
        ClaimMade(Did, T::ClaimId, T::GroupId),
        /// Claim was attested (target DID, index of claim, group_id)
        ClaimAttested(Did, T::ClaimId, T::GroupId),
        /// Claim attestation revoked (target DID, index of claim, group_id)
        ClaimAttestationRevoked(Did, T::ClaimId, T::GroupId),
        /// Catalog added (Owner Did, Catalog Id)
        CatalogCreated(T::GroupId, T::CatalogId),
        /// Catalog removed (Owner Did, Catalog Id)
        CatalogRemoved(T::GroupId, T::CatalogId),
        /// Dids added to catalog (Owner Did, Catalog Id)
        CatalogDidsAdded(T::GroupId, T::CatalogId),
        /// Dids removed from catalog (group_id, catalog_id)
        CatalogDidsRemoved(T::GroupId, T::CatalogId),
        /// DID Controller updated (from_group_id, to_group_id, did)
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
    #[pallet::getter(fn did_document)]
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
        T::GroupId,
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
        T::GroupId,
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
            T::GroupId,
            T::Moment,
            BoundedVec<u8, <T as Config>::NameLimit>,
            BoundedVec<u8, <T as Config>::FactStringLimit>,
        >,
        OptionQuery,
    >;

    /// Catalog ownership
    #[pallet::storage]
    #[pallet::getter(fn catalog_ownership)]
    pub type CatalogOwnership<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::GroupId,
        Blake2_128Concat,
        T::CatalogId,
        Catalog<BoundedVec<u8, <T as Config>::NameLimit>>,
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
        BoundedVec<u8, <T as Config>::NameLimit>,
        OptionQuery,
    >;

    macro_rules! next_id {
        ($id:ty,$t:ty) => {{
            let current_id = <$id>::get();
            let next_id = current_id
                .checked_add(&One::one())
                .ok_or(Error::<$t>::NoIdAvailable)?;
            <$id>::put(next_id);
            current_id
        }};
    }

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
            properties: Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let bounded_name = enforce_limit_option!(short_name);

            let properties = enforce_limit_did_properties_option!(properties);

            Self::mint_did(sender.clone(), sender, bounded_name, properties);
            Ok(().into())
        }

        /// Register a new DID for caller. A group calls to create a new DID.
        ///
        /// # <weight>
        /// - O(1).
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_did_for(
            origin: OriginFor<T>,
            subject: T::AccountId,
            short_name: Option<Vec<u8>>,
            properties: Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let bounded_name = enforce_limit_option!(short_name);

            let properties = enforce_limit_did_properties_option!(properties);

            Self::mint_did(subject, sender, bounded_name, properties);
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
            add_properties: Option<Vec<DidProperty<Vec<u8>, Vec<u8>>>>,
            remove_keys: Option<Vec<Vec<u8>>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let bounded_short_name = enforce_limit_option!(short_name);

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

            <DidDocuments<T>>::try_mutate_exists(&did, |maybe_did_doc| -> DispatchResult {
                let did_doc = maybe_did_doc.as_mut().ok_or(Error::<T>::NotFound)?;
                //TODO: we cannot delete short_name?
                if let Some(short_name) = bounded_short_name {
                    did_doc.short_name = Some(short_name);
                }
                Ok(())
            })?;

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
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
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

            let properties = enforce_limit_did_properties!(properties);

            <DidDocumentProperties<T>>::remove_prefix(&did);

            properties.into_iter().for_each(|property| {
                let hash = T::Hashing::hash_of(&property.name);
                <DidDocumentProperties<T>>::insert(&did, &hash, property);
            });

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
            target_did: Did,
            add: Option<Vec<T::AccountId>>,
            remove: Option<Vec<T::AccountId>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <DidByController<T>>::contains_key(&sender, &target_did),
                Error::<T>::NotController
            );
            if let Some(remove) = remove.clone() {
                remove.iter().for_each(|remove| {
                    <DidByController<T>>::remove(&remove, &target_did);
                    <DidControllers<T>>::remove(&target_did, &remove);
                });
            }
            if let Some(add) = add.clone() {
                add.iter().for_each(|add| {
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
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn authorize_claim_consumers(
            origin: OriginFor<T>,
            target_did: Did,
            claim_consumers: Vec<ClaimConsumer<T::GroupId, T::Moment>>,
        ) -> DispatchResultWithPostInfo {
            let (_group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            // ensure!(
            //     <DidByController<T>>::contains_key(&group_id, &target_did),
            //     Error::<T>::NotController
            // );

            claim_consumers.iter().for_each(|claim_consumer| {
                <ClaimConsumers<T>>::insert(
                    &target_did,
                    &claim_consumer.group_id,
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
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn revoke_claim_consumers(
            origin: OriginFor<T>,
            target_did: Did,
            claim_consumers: Vec<T::GroupId>,
        ) -> DispatchResultWithPostInfo {
            let (_group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            // ensure!(
            //     <DidByController<T>>::contains_key(&group_id, &target_did),
            //     Error::<T>::NotController
            // );

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
            claim_issuers: Vec<ClaimIssuer<T::GroupId, T::Moment>>,
        ) -> DispatchResultWithPostInfo {
            let (_group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            // ensure!(
            //     <DidByController<T>>::contains_key(&group_id, &target_did),
            //     Error::<T>::NotController
            // );

            claim_issuers.iter().for_each(|claim_issuer| {
                <ClaimIssuers<T>>::insert(
                    &target_did,
                    &claim_issuer.group_id,
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
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn revoke_claim_issuers(
            origin: OriginFor<T>,
            target_did: Did,
            claim_issuers: Vec<T::GroupId>,
        ) -> DispatchResultWithPostInfo {
            let (_group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            // ensure!(
            //     <DidByController<T>>::contains_key(&group_id, &target_did),
            //     Error::<T>::NotController
            // );

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
            target_did: Did,
            description: Vec<u8>,
            statements: Vec<Statement<Vec<u8>, Vec<u8>>>,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            // ensure!(
            //     <DidByController<T>>::contains_key(&group_id, &target_did),
            //     Error::<T>::NotController
            // );

            ensure!(
                <ClaimConsumers<T>>::contains_key(target_did, group_id),
                Error::<T>::NotAuthorized
            );

            let claim = Claim {
                description: enforce_limit!(description),
                statements: statements
                    .into_iter()
                    .map(|statement| {
                        Ok(Statement {
                            name: enforce_limit!(statement.name),
                            fact: enforce_limit_fact!(statement.fact),
                            for_issuer: statement.for_issuer,
                        })
                    })
                    .collect::<Result<Vec<_>, Error<T>>>()?,
                created_by: group_id,
                attestation: None,
            };

            let claim_index = next_id!(NextClaimId<T>, T);

            <Claims<T>>::insert(&target_did, claim_index, claim);

            Self::deposit_event(Event::ClaimMade(target_did, claim_index, group_id));
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
            target_did: Did,
            claim_index: T::ClaimId,
            statements: Vec<Statement<Vec<u8>, Vec<u8>>>,
            valid_until: T::Moment,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                Self::can_attest_claim(target_did, group_id),
                Error::<T>::NotAuthorized
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
                        attested_by: group_id,
                        valid_until,
                    });

                    Ok(())
                },
            )?;

            Self::deposit_event(Event::ClaimAttested(target_did, claim_index, group_id));

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
            target_did: Did,
            claim_index: T::ClaimId,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                Self::can_attest_claim(target_did, group_id),
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
                group_id,
            ));
            Ok(().into())
        }

        /// Add a new catalog
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_catalog(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            let bounded_name = enforce_limit!(name);

            let catalog_id = next_id!(NextCatalogId<T>, T);

            <CatalogOwnership<T>>::insert(group_id, catalog_id, Catalog { name: bounded_name });

            Self::deposit_event(Event::CatalogCreated(group_id, catalog_id));
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
            catalog_id: T::CatalogId,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                <CatalogOwnership<T>>::contains_key(group_id, catalog_id),
                Error::<T>::NotController
            );

            <CatalogOwnership<T>>::remove(group_id, catalog_id);
            <Catalogs<T>>::remove_prefix(&catalog_id);

            Self::deposit_event(Event::CatalogRemoved(group_id, catalog_id));
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
            catalog_id: T::CatalogId,
            dids: Vec<(Did, Vec<u8>)>,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                <CatalogOwnership<T>>::contains_key(group_id, catalog_id),
                Error::<T>::NotController
            );

            //TODO: check name lengths first before mutating any storage

            for (did, short_name) in dids.into_iter() {
                let bounded_name = enforce_limit!(short_name);

                <Catalogs<T>>::insert(catalog_id, did, bounded_name);
            }

            Self::deposit_event(Event::CatalogDidsAdded(group_id, catalog_id));
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
            catalog_id: T::CatalogId,
            dids: Vec<Did>,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                <CatalogOwnership<T>>::contains_key(group_id, catalog_id),
                Error::<T>::NotController
            );

            for did in dids.into_iter() {
                <Catalogs<T>>::remove(catalog_id, did);
            }

            Self::deposit_event(Event::CatalogDidsRemoved(group_id, catalog_id));
            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --

        pub fn get_catalogs(
            group_id: T::GroupId,
        ) -> Vec<(
            T::CatalogId,
            Catalog<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut catalogs = Vec::new();
            <CatalogOwnership<T>>::iter_prefix(group_id)
                .for_each(|(catalog_id, catalog)| catalogs.push((catalog_id, catalog)));
            catalogs
        }

        pub fn get_catalog(
            group_id: T::GroupId,
            catalog_id: T::CatalogId,
        ) -> Option<Catalog<BoundedVec<u8, <T as Config>::NameLimit>>> {
            <CatalogOwnership<T>>::get(group_id, catalog_id)
        }

        pub fn get_dids_in_catalog(
            catalog_id: T::CatalogId,
        ) -> Vec<(Did, BoundedVec<u8, <T as Config>::NameLimit>)> {
            let mut dids = Vec::new();
            <Catalogs<T>>::iter_prefix(catalog_id)
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
            let short_name = <Catalogs<T>>::get(catalog_id, did);
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
                T::GroupId,
                <T as timestamp::Config>::Moment,
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
        )> {
            let mut claims = Vec::new();
            <Claims<T>>::iter_prefix(did)
                .for_each(|(claim_index, claim)| claims.push((claim_index, claim)));
            claims
        }

        // -- private functions --

        /// Returns true if a `claim_issuer` can attest a claim against `target_did`
        pub fn can_attest_claim(target_did: Did, claim_issuer: T::GroupId) -> bool {
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
}
