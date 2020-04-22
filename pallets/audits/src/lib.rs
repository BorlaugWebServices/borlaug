//! # Audit Module
//!
//! ## Overview
//!
//! An audit
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For general users
//! * `create_audit` - Creates a new audit

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod tests;

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, weights::SimpleDispatchInfo,
    Parameter,
};
use frame_system::{self as system, ensure_signed};
use primitives::{ Evidence, Observation};
use sp_core::H256 as Hash;
use sp_runtime::{
    traits::{AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One},
};
use sp_std::prelude::*;

pub trait Trait: frame_system::Trait + timestamp::Trait {
    type AuditId: Parameter
        + Member
        + AtLeast32Bit
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + PartialEq;

    type ControlPointId: Parameter
        + Member
        + AtLeast32Bit
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + PartialEq;

    type EvidenceId: Parameter
        + Member
        + AtLeast32Bit
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + PartialEq;

    type ObservationId: Parameter
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
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::AuditId,
        <T as Trait>::ObservationId,
        <T as Trait>::ControlPointId,
        <T as Trait>::EvidenceId,

    {
        /// New registry created (owner, registry id)
        AuditCreated(AccountId, AuditId),
        /// New observation created (audit id, control point id, observation id)
        ObservationCreated(AuditId, ControlPointId, ObservationId),
        /// Evidence Attached (audit id, evidence id)
        EvidenceAttached(AuditId, EvidenceId),
        ///
        EvidenceLinked(AuditId, EvidenceId, ObservationId),
        ///
        EvidenceUnlinked(AuditId, EvidenceId, ObservationId),

    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Value was None
        NoneValue,
        NoIdAvailable
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Audits {

        /// Incrementing nonce
        pub Nonce get(fn nonce) build(|_| 1u64): u64;

        /// The next available audit index
        pub NextAuditId get(fn next_audit_id) config(): T::AuditId;
        /// The next available
        pub NextControlPointId get(fn next_control_point_id) config(): T::ControlPointId;
        /// The next available  index
        pub NextObservationId get(fn next_observation_id) config(): T::ObservationId;
        /// The next available  index
        pub NextEvidenceId get(fn next_evidence_id) config(): T::EvidenceId;

        /// Audits
        pub Audits get(fn audits):
            map hasher(blake2_128_concat) T::AccountId => Vec<T::AuditId>;

        ///Auditors
        pub Auditors get(fn auditors):
            map hasher(blake2_128_concat) T::AuditId => Vec<T::AccountId>;

        /// Check Points
        pub ControlPoints get(fn control_points):
            double_map hasher(blake2_128_concat) T::AuditId, hasher(blake2_128_concat) T::ControlPointId => Vec<Observation<T::ObservationId>>;

       /// Evidence
       pub Evidences get(fn evidences):
            double_map hasher(blake2_128_concat) T::AuditId, hasher(blake2_128_concat) T::EvidenceId => Evidence<T::EvidenceId>;

       /// Evidence Link
       pub EvidenceLinks get(fn evidence_links):
            double_map hasher(blake2_128_concat) T::ObservationId, hasher(blake2_128_concat) T::EvidenceId => T::EvidenceId;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Create a new registry
        ///
        /// Arguments: None

        #[weight = SimpleDispatchInfo::FixedNormal(100_000)]
        fn create_audit(origin) {
            let sender = ensure_signed(origin)?;

            let audit_id = Self::next_audit_id();
            let next_id = audit_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextAuditId<T>>::put(next_id);

            <Audits<T>>::append_or_insert(&sender, &[&audit_id][..]);

            Self::deposit_event(RawEvent::AuditCreated(sender, audit_id));
        }

        /// Create a new observation

        #[weight = SimpleDispatchInfo::FixedNormal(100_000)]
        fn create_observation(
            origin,
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
            observation: Observation<T::ObservationId>
          ){
                let sender = ensure_signed(origin)?;

                let observation_id = Self::next_observation_id();
                let next_id = observation_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoIdAvailable)?;
                <NextObservationId<T>>::put(next_id);

                let mut observation=observation;

                observation.observation_id=Some(observation_id);

                <ControlPoints<T>>::append_or_insert(&audit_id, &control_point_id, &[&observation][..]);

                Self::deposit_event(RawEvent::ObservationCreated(
                    audit_id,
                    control_point_id,
                    observation_id,
                ));
        }


        #[weight = SimpleDispatchInfo::FixedNormal(100_000)]
        fn create_evidence(
            origin,
            audit_id: T::AuditId,
            evidence: Evidence<T::EvidenceId>
        ){
            let sender = ensure_signed(origin)?;

            let evidence_id = Self::next_evidence_id();
            let next_id = evidence_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextEvidenceId<T>>::put(next_id);

            let mut evidence=evidence;

            evidence.evidence_id=Some(evidence_id);

            <Evidences<T>>::insert(&audit_id, &evidence_id, evidence);

            Self::deposit_event(RawEvent::EvidenceAttached(
                audit_id,
                evidence_id,
            ));
        }

        #[weight = SimpleDispatchInfo::FixedNormal(100_000)]
        fn link_evidence(
            origin,
            audit_id:T::AuditId,
            evidence_id:T::EvidenceId,
            observation_id: T::ObservationId,
        ){
            let sender = ensure_signed(origin)?;

            <EvidenceLinks<T>>::insert(&observation_id, &evidence_id, evidence_id);

            Self::deposit_event(RawEvent::EvidenceLinked(
                audit_id,
                evidence_id,
                observation_id
            ));
        }

        #[weight = SimpleDispatchInfo::FixedNormal(100_000)]
        fn unlink_evidence(
            origin,
            audit_id:T::AuditId,
            evidence_id:T::EvidenceId,
            observation_id:T::ObservationId,
        ){
        let sender = ensure_signed(origin)?;

            <EvidenceLinks<T>>::remove(&observation_id, &evidence_id);

            Self::deposit_event(RawEvent::EvidenceUnlinked(
                audit_id,
                evidence_id,
                observation_id
            ));
        }

    }
}

// private functions
impl<T: Trait> Module<T> {}
