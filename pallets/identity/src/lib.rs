//! # DidRegistry Module
//!
//! ## Overview
//!
//! A DID registry is a data registry that mediates the creation, verification, updating, and
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

mod mock;
mod tests;

use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure, traits::Randomness, Parameter,
    StorageMap,
};
use frame_system::{self as system, ensure_signed};
use primitives::{
    attestation::Attestation,
    claim::{Claim, ClaimConsumer, ClaimIssuer, Statement},
    did::Did,
    did_document::DidDocument,
    did_property::DidProperty,
};
#[cfg(not(feature = "std"))]
use sp_io::hashing::blake2_256;
use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, One};
use sp_std::prelude::*;

/// A claim index.
pub type ClaimIndex = u64;

/// Short name associated with Did.
pub type ShortName = Vec<u8>;

/// Key used for DidProperty.
pub type DidPropertyName = Vec<u8>;

pub trait Trait: frame_system::Trait + timestamp::Trait {
    type CatalogId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;

    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        <T as frame_system::Trait>::AccountId,
        <T as timestamp::Trait>::Moment,
        <T as Trait>::CatalogId,
    {
     /// A new DID was registered (Subject, Controller, DID)
     Registered(AccountId, AccountId, Did),
     /// Did updated (Controller, DID)
     DidUpdated(AccountId, Did),
     /// Did replaced (Controller, DID)
     DidReplaced(AccountId, Did),
     /// Claim consumers added
     ClaimConsumersAdded(Did, Vec<ClaimConsumer<Moment>>),
     /// Claim consumer removed
     ClaimConsumersRemoved(Did, Vec<Did>),
     /// Claim issuer added
     ClaimIssuersAdded(Did, Vec<ClaimIssuer<Moment>>),
     /// Claim issuerz removed
     ClaimIssuersRemoved(Did, Vec<Did>),
     /// Claim was made against a DID (target DID, index of claim, claim proposer DID)
     ClaimMade(Did, ClaimIndex, Did),
     /// Claim was attested (target DID, index of claim, claim issuer DID)
     ClaimAttested(Did, ClaimIndex, Did),
     /// Claim attestation revoked (target DID, index of claim, claim issuer DID)
     ClaimAttestationRevoked(Did, ClaimIndex, Did),
     /// Catalog added (Owner Did, Catalog Id)
     CatalogCreated(Did, CatalogId),
     /// Catalog removed (Owner Did, Catalog Id)
     CatalogRemoved(Did, CatalogId),
     /// Dids added to catalog (Owner Did, Catalog Id)
     CatalogDidsAdded(Did, CatalogId),
     /// Dids removed from catalog (Owner Did, Catalog Id)
     CatalogDidsRemoved(Did, CatalogId),
     /// DID Controllers updated (Did, when Id)
     DidControllersUpdated(AccountId, Did),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Value was None
        NoneValue,
        /// A non-controller account attempted to  modify a DID
        NotController,
        /// Not authorized to make a claim or attest a claim
        NotAuthorized,
        /// Id out of bounds
        NoIdAvailable
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Identity {

        /// Incrementing nonce
        pub Nonce get(fn nonce) build(|_| 1u64): u64;

        /// An account can have multiple DIDs
        /// AccountId => Vec<Did>
        pub DidRegistry get(fn dids): map hasher(blake2_128_concat) T::AccountId => Vec<Did>;

        /// A DID has a DID Document
        /// Did => DidDocument
        pub DidInfo get(fn did_document): map hasher(blake2_128_concat) Did => DidDocument;

        /// Controller for DIDs.
        /// Controller AccountId => Collection of DIDs
        pub DidController get(fn controller): map hasher(blake2_128_concat) T::AccountId => Vec<Did>;

        /// The next available claim index, aka the number of claims started so far.
        pub ClaimCount get(fn claim_count) build(|_| 0 as ClaimIndex): ClaimIndex;

        /// Claim consumers request a claim to offer protected services
        /// Subject DID => DIDs of claim consumers
        pub ClaimConsumers get(fn claim_comsumers):
            map hasher(blake2_128_concat) Did => Vec<ClaimConsumer<T::Moment>>;

        /// Claim issuers provide verifiable claims
        /// Subject DID => (DIDs of claim issuers, Expiration time)
        pub ClaimIssuers get(fn claim_issuers):
            map hasher(blake2_128_concat) Did => Vec<ClaimIssuer<T::Moment>>;

        /// Claims associated with a DID
        /// Subject DID => collection of ClaimIndex
        pub ClaimsOf get(fn claims_of): map hasher(blake2_128_concat) Did => Vec<ClaimIndex>;

        /// Claims associated with a DID
        /// Subject DID => (Claim ID => Claim)
        pub Claims get(fn claims):
            double_map hasher(blake2_128_concat) Did, hasher(twox_64_concat) ClaimIndex => Claim<T::Moment>;

        /// The next available catalog index
        pub NextCatalogId get(fn next_catalog_id) config(): T::CatalogId;

        /// Catalog ownership
        pub CatalogOwnership get(fn catalog_ownership):
            map hasher(blake2_128_concat) Did => Vec<T::CatalogId>;

        /// Catalogs
        /// For each catalog index, we keep a mapping of `Did` an index name
        pub Catalogs get(fn catalogs):
            double_map hasher(twox_64_concat) T::CatalogId, hasher(blake2_128_concat) Did => Option<ShortName>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Register a new DID for caller. Subject calls to create a new DID.
        ///
        /// # <weight>
        /// - O(1).
        /// # </weight>
        #[weight = 100_000]
        pub fn register_did(origin, properties: Option<Vec<DidProperty>>) {
            let sender = ensure_signed(origin)?;

            Self::mint_did(sender.clone(), sender, properties);
        }

        /// Register a new DID for caller. Subject calls to create a new DID.
        ///
        /// # <weight>
        /// - O(1).
        /// # </weight>
        #[weight = 100_000]
        pub fn register_did_for(origin, subject: T::AccountId, properties: Option<Vec<DidProperty>>) {
            let sender = ensure_signed(origin)?;

            Self::mint_did(subject, sender, properties);
        }

        /// Append given collection of Did properties provided the caller is a controller
        ///
        /// Arguments:
        /// - `did` DID to which properties are to be added
        /// - `add_properties` DID properties to be added
        /// - `remove_keys` Keys of DID properties to be removed
        #[weight = 100_000]
        pub fn update_did(
            origin,
            did: Did,
            add_properties: Option<Vec<DidProperty>>,
            remove_keys: Option<Vec<DidPropertyName>>,
        ) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), did.clone()),
                Error::<T>::NotController
            );

            //TODO: does this fail correctly if did does not exist?

            let mut did_doc = <DidInfo>::take(&did);
            remove_keys.and_then(|remove_keys| {
                Some(did_doc.properties.retain(|p| !remove_keys.contains(&p.name)))
            });

            add_properties
                .and_then(|mut add_properties| Some(did_doc.properties.append(&mut add_properties)));
            <DidInfo>::insert(&did, did_doc);


            Self::deposit_event(RawEvent::DidUpdated(sender, did));
        }

        /// Append given collection of Did properties provided the caller is a controller
        ///
        /// Arguments:
        /// - `did` DID to which properties are to be added
        /// - `properties` DID properties to be added
        #[weight = 100_000]
        pub fn replace_did(origin, did: Did, properties: Vec<DidProperty>) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), did.clone()),
                Error::<T>::NotController
            );

            <DidInfo>::remove(&did);
            <DidInfo>::insert(&did, DidDocument { properties });


            Self::deposit_event(RawEvent::DidReplaced(sender, did));
        }

        /// Append given collection of Did properties provided the caller is a controller
        ///
        /// Arguments:
        /// - `did` subject
        /// - `add` DIDs to be added as controllers
        /// - `remove` DIDs to be removed as controllers
        #[weight = 100_000]
        pub fn manage_controllers(
            origin,
            did: Did,
            add: Option<Vec<T::AccountId>>,
            remove: Option<Vec<T::AccountId>>,
        ) {
            let sender = ensure_signed(origin)?;
            ensure!(
                Self::is_controller(sender.clone(), did.clone()),
                Error::<T>::NotController
            );
            if let Some(to_be_removed) = remove {
                to_be_removed.iter().for_each(|remove_account|{
                    <DidController<T>>::mutate(&remove_account, |controlled|
                        controlled.retain(|c| *c!=did)
                    );
                })
            }

            if let Some(to_be_added) = add {
                to_be_added.iter().for_each(|add_account|{

                    <DidController<T>>::mutate(&add_account, |controlled|
                        controlled.append( &mut vec![did])
                    );
                })
            }

            Self::deposit_event(RawEvent::DidControllersUpdated(
                sender,
                did,

            ));
        }

        /// Grants a claim consumer permission to write a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_consumers` DIDs of claim consumer
        #[weight = 100_000]
        pub fn authorize_claim_consumers(
            origin,
            target_did: Did,
            claim_consumers: Vec<ClaimConsumer<T::Moment>>,
        ) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), target_did.clone()),
                Error::<T>::NotController
            );

            let mut claim_consumers = claim_consumers;
            <ClaimConsumers<T>>::mutate(&target_did, |consumers| {
                consumers.append(&mut claim_consumers)
            });


            Self::deposit_event(RawEvent::ClaimConsumersAdded(
                target_did,
                claim_consumers,

            ));
        }

        /// Revokes claim consumers permission to write a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_consumers` DIDs of claim consumers to be revoked
        #[weight = 100_000]
        pub fn revoke_claim_consumers(origin, target_did: Did, claim_consumers: Vec<Did>) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), target_did.clone()),
                Error::<T>::NotController
            );

            <ClaimConsumers<T>>::mutate(&target_did, |consumers| {
                consumers.retain(|c| !claim_consumers.iter().any(|&a| c.consumer == a))
            });


            Self::deposit_event(RawEvent::ClaimConsumersRemoved(
                target_did,
                claim_consumers,

            ));
        }

        /// Grants a claim attester permission to attest a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_issuers` DIDs of claim issuer
        #[weight = 100_000]
        pub fn authorize_claim_issuers(origin, target_did: Did, claim_issuers: Vec<ClaimIssuer<T::Moment>>) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), target_did.clone()),
                Error::<T>::NotController
            );

            let mut claim_issuers = claim_issuers;
            <ClaimIssuers<T>>::mutate(&target_did, |issuers| issuers.append(&mut claim_issuers));


            Self::deposit_event(RawEvent::ClaimIssuersAdded(target_did, claim_issuers));
        }

        /// Revokes claim issuers permission to attest a claim against a DID
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_issuers` DIDs of claim issuers to be revoked
        #[weight = 100_000]
        pub fn revoke_claim_issuers(origin, target_did: Did, claim_issuers: Vec<Did>) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), target_did.clone()),
                Error::<T>::NotController
            );

            <ClaimIssuers<T>>::mutate(&target_did, |issuers| {
                issuers.retain(|i| !claim_issuers.iter().any(|&a| i.issuer == a))
            });


            Self::deposit_event(RawEvent::ClaimIssuersRemoved(
                target_did,
                claim_issuers,

            ));
        }

        /// Claim consumer calls this to make a new `claim` against `target_did`
        ///
        /// Arguments:
        /// - `target_did` DID to which claims are to be added
        /// - `claim_consumer` DID of claim consumer
        #[weight = 100_000]
        pub fn make_claim(
            origin,
            claim_consumer: Did,
            target_did: Did,
            description: Vec<u8>,
            statements: Vec<Statement>,
        ) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), claim_consumer),
                Error::<T>::NotController
            );
            ensure!(
                Self::can_make_claim(target_did.clone(), claim_consumer),
                Error::<T>::NotAuthorized
            );

            let claim_index = Self::claim_count();
            ClaimCount::put(claim_index + 1u64);

            let claim = Claim {                
                description,
                statements,
                created_by: claim_consumer,
                attestation: None,
            };
            <ClaimsOf>::mutate(&target_did, |indexes| indexes.push(claim_index));
            <Claims<T>>::insert(&target_did, claim_index, claim.clone());


            Self::deposit_event(RawEvent::ClaimMade(
                target_did,
                claim_index,
                claim_consumer,

            ));
        }

        /// Claim issuer attests `claim_index` against `target_did` with given `statements`
        ///
        /// Arguments:
        /// - `claim_issuer` DID of claim issuer
        /// - `target_did` DID against which claims are to be attested
        /// - `claim_index` Claim to be attested
        /// - `statements` Claim issuer overwrites these statements
        /// - `valid_until` Attestation expires
        #[weight = 100_000]
        pub fn attest_claim(
            origin,
            claim_issuer: Did,
            target_did: Did,
            claim_index: ClaimIndex,
            statements: Vec<Statement>,
            valid_until: T::Moment,
        ) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), claim_issuer.clone()),
                Error::<T>::NotController
            );
            ensure!(
                Self::can_attest_claim(target_did.clone(), claim_issuer.clone()),
                Error::<T>::NotAuthorized
            );

            if <Claims<T>>::contains_key(&target_did, claim_index) {
                let mut stmts = statements.clone();
                let names = statements.into_iter().map(|s| s.name).collect::<Vec<_>>();
                let mut claim = <Claims<T>>::take(&target_did, claim_index);
                claim.statements.retain(|s| !names.contains(&s.name));
                claim.statements.append(&mut stmts);
                claim.attestation = Some(Attestation {
                    attested_by: claim_issuer,
                    valid_until,
                });
                <Claims<T>>::insert(&target_did, claim_index, claim);


                Self::deposit_event(RawEvent::ClaimAttested(
                    target_did,
                    claim_index,
                    claim_issuer,

                ));
            }
        }

        /// Claim issuer revokes `claim_index`
        ///
        /// Arguments:
        /// - `claim_issuer` DID of claim issuer
        /// - `target_did` DID against which claims are to be attested
        /// - `claim_index` Claim to be attested
        #[weight = 100_000]
        pub fn revoke_attestation(origin, claim_issuer: Did, target_did: Did, claim_index: ClaimIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), claim_issuer.clone()),
                Error::<T>::NotController
            );
            ensure!(
                Self::can_attest_claim(target_did.clone(), claim_issuer.clone()),
                Error::<T>::NotAuthorized
            );

            <Claims<T>>::mutate(&target_did, claim_index, |claim| claim.attestation = None);

            Self::deposit_event(RawEvent::ClaimAttestationRevoked(
                target_did,
                claim_index,
                claim_issuer,

            ));
        }

        /// Add a new catalog
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        #[weight = 100_000]
        pub fn create_catalog(origin, owner_did: Did) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), owner_did.clone()),
                Error::<T>::NotController
            );

            let catalog_id = Self::next_catalog_id();
            let next_id = catalog_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextCatalogId<T>>::put(next_id);

            <CatalogOwnership<T>>::append(owner_did, &catalog_id);


            Self::deposit_event(RawEvent::CatalogCreated(owner_did, catalog_id));
        }

        /// Remove a catalog
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `catalog_id` Catalog to be removed
        #[weight = 100_000]
        pub fn remove_catalog(origin, owner_did: Did, catalog_id: T::CatalogId) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), owner_did.clone()),
                Error::<T>::NotController
            );

            <CatalogOwnership<T>>::mutate(owner_did, |catalogs| {
                catalogs.retain(|cid| *cid != catalog_id)
            });
            <Catalogs<T>>::remove_prefix(&catalog_id);


            Self::deposit_event(RawEvent::CatalogRemoved(owner_did, catalog_id));
        }

        /// Add DIDs to a catalog
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `catalog_id` Catalog to which DID are to be added
        /// - `dids` DIDs are to be added
        #[weight = 100_000]
        pub fn add_dids_to_catalog(
            origin,
            owner_did: Did,
            catalog_id: T::CatalogId,
            dids: Vec<(Did, ShortName)>,
        ) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), owner_did.clone()),
                Error::<T>::NotController
            );

            for (did, short_name) in dids.into_iter() {
                <Catalogs<T>>::insert(catalog_id, did, short_name);
            }


            Self::deposit_event(RawEvent::CatalogDidsAdded(owner_did, catalog_id));
        }


        /// Remove DIDs from a catalog
        ///
        /// Arguments:
        /// - `owner_did` DID of caller
        /// - `catalog_id` Catalog to which DID are to be removed
        /// - `dids` DIDs are to be removed
        #[weight = 100_000]
        pub fn remove_dids_from_catalog(
            origin,
            owner_did: Did,
            catalog_id: T::CatalogId,
            dids: Vec<Did>,
        ) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_controller(sender.clone(), owner_did.clone()),
                Error::<T>::NotController
            );

            for did in dids.into_iter() {
                <Catalogs<T>>::remove(catalog_id, did);
            }


            Self::deposit_event(RawEvent::CatalogDidsRemoved(owner_did, catalog_id));
        }
    }
}

impl<T: Trait> Module<T> {
    /// Returns true if a `account` can control `did`
    pub fn is_controller(account: T::AccountId, did: Did) -> bool {
        if <DidController<T>>::contains_key(account.clone()) {
            let dids = <DidController<T>>::get(account);
            dids.contains(&did)
        } else {
            false
        }
    }

    /// Returns true if a `claim_consumer` can make a claim against `target_did`
    pub fn can_make_claim(target_did: Did, claim_consumer: Did) -> bool {
        if <ClaimConsumers<T>>::contains_key(target_did) {
            let consumers = <ClaimConsumers<T>>::get(&target_did);
            consumers.iter().any(|&a| a.consumer == claim_consumer)
        } else {
            false
        }
    }

    /// Returns true if a `claim_issuer` can attest a claim against `target_did`
    pub fn can_attest_claim(target_did: Did, claim_issuer: Did) -> bool {
        if <ClaimIssuers<T>>::contains_key(target_did) {
            let issuers = <ClaimIssuers<T>>::get(&target_did);
            issuers.iter().any(|&a| a.issuer == claim_issuer)
        } else {
            false
        }
    }

    // -- private functions --

    fn next_nonce() -> u64 {
        let nonce = <Nonce>::get();
        <Nonce>::mutate(|n| *n += 1u64);
        nonce
    }

    /// Creates a Did with given properties
    fn mint_did(
        subject: T::AccountId,
        controller: T::AccountId,
        properties: Option<Vec<DidProperty>>,
    ) {
        let nonce = Self::next_nonce();
        let random_seed = <randomness::Module<T>>::random_seed();
        let encoded = (random_seed, subject.clone(), nonce).encode();
        let id = sp_io::hashing::blake2_256(&encoded);

        let did = Did { id };
        let did_doc = if let Some(props) = properties {
            DidDocument { properties: props }
        } else {
            DidDocument {
                properties: Vec::new(),
            }
        };

        <DidRegistry<T>>::append(&subject, &did);
        <DidInfo>::insert(&did, did_doc);
        <DidController<T>>::append(controller.clone(), &did);

        Self::deposit_event(RawEvent::Registered(subject, controller, did));
    }
}
