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
    decl_error, decl_event, decl_module, decl_storage, ensure, weights::SimpleDispatchInfo,
    Parameter,
};
use frame_system::{self as system, ensure_signed};
use primitives::{Audit, AuditStatus, Evidence, Observation};
use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One};
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
        /// Evidence Linked to Observation
        EvidenceLinked(AuditId, EvidenceId, ObservationId),
        /// Evidence Unlinked from Observation
        EvidenceUnlinked(AuditId, EvidenceId, ObservationId),

    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Value was None
        NoneValue,
        NoIdAvailable,
        NoObservationAvailable,
        NoEvidenceAvailable,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Audits {

        /// Incrementing nonce
        pub Nonce get(fn nonce) build(|_| 1u64): u64;

        /// The next available audit index
        pub NextAuditId get(fn next_audit_id) config(): T::AuditId;

        /// The next available  index
        pub NextObservationId get(fn next_observation_id) config(): T::ObservationId;

        /// The next available  index
        pub NextEvidenceId get(fn next_evidence_id) config(): T::EvidenceId;

        /// Audits
        pub Audits get(fn audits):
            double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::AuditId => Audit<T::AuditId>;

        /// Audit => (Control Point => Collection of Observation)
        pub ObservationOf get(fn observation_of):
            double_map hasher(blake2_128_concat) T::AuditId, hasher(blake2_128_concat) T::ControlPointId => Vec<T::ObservationId>;

        /// Observation Id => Observation(Compliance, Procedural Notes)
        pub Observations get(fn observations):
            map hasher(blake2_128_concat) T::ObservationId => Observation<T::ObservationId>;

       /// Audit Id => (Evidence Id => Evidence(Name, Content-Type, URL, Hash))
       pub Evidences get(fn evidences):
            double_map hasher(blake2_128_concat) T::AuditId, hasher(blake2_128_concat) T::EvidenceId => Evidence<T::EvidenceId>;

       /// Observation Id => (Evidence Id => Evidence Id)
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
        fn create_audit(origin,auditor:Vec<u8>) {
            let sender = ensure_signed(origin)?;

            let audit_id = Self::next_audit_id();
            let next_id = audit_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextAuditId<T>>::put(next_id);

            let audit=Audit{
                audit_id:audit_id,
                auditor:auditor,
                status:AuditStatus::Requested
            };

            <Audits<T>>::insert(&sender, &audit_id,audit);

            Self::deposit_event(RawEvent::AuditCreated(sender, audit_id));
        }

        /// Create New Observation
        ///
        /// Arguments:
        /// - `audit_id` id created on chain of audit
        /// - `control_point_id` control point id of audit
        /// - `observation` (compliance, procedural notes)
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

                <ObservationOf<T>>::append_or_insert(&audit_id, &control_point_id, &[&observation_id][..]);

                <Observations<T>>::insert(&observation_id, observation);

                Self::deposit_event(RawEvent::ObservationCreated(
                    audit_id,
                    control_point_id,
                    observation_id,
                ));
        }

        /// Attach New Evidence to Audit
        ///
        /// Arguments:
        /// - `audit_id` id of audit created on chain
        /// - `evidence` Body of evidence
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

        /// Link Attached evidence to observation
        ///
        /// Arguments:
        /// - `audit_id` id of audit created on chain
        /// - `evidence_id` id of evidence created on chain
        /// - `observation_id` id of observation created on chain
        #[weight = SimpleDispatchInfo::FixedNormal(100_000)]
        fn link_evidence(
            origin,
            audit_id:T::AuditId,
            control_point_id: T::ControlPointId,
            observation_id: T::ObservationId,
            evidence_id:T::EvidenceId,
        ){
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_observation_exist(audit_id, control_point_id, observation_id),
            <Error<T>>::NoObservationAvailable);

            ensure!(Self::is_evidence_exist(audit_id, evidence_id),
            <Error<T>>::NoEvidenceAvailable);

            <EvidenceLinks<T>>::insert(&observation_id, &evidence_id, evidence_id);

            Self::deposit_event(RawEvent::EvidenceLinked(
                audit_id,
                evidence_id,
                observation_id
            ));
        }

        /// Unlink Attached evidence from observation
        ///
        /// Arguments:
        /// - `audit_id` id of audit created on chain
        /// - `evidence_id` id of evidence created on chain
        /// - `observation_id` id of observation created on chain
        #[weight = SimpleDispatchInfo::FixedNormal(100_000)]
        fn unlink_evidence(
            origin,
            audit_id:T::AuditId,
            control_point_id: T::ControlPointId,
            observation_id:T::ObservationId,
            evidence_id:T::EvidenceId,
        ){
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_observation_exist(audit_id, control_point_id, observation_id),
            <Error<T>>::NoObservationAvailable);

            ensure!(Self::is_evidence_exist(audit_id, evidence_id),
            <Error<T>>::NoEvidenceAvailable);

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
impl<T: Trait> Module<T> {
    fn is_observation_exist(
        audit_id: T::AuditId,
        control_point_id: T::ControlPointId,
        observation_id: T::ObservationId,
    ) -> bool {
        if <ObservationOf<T>>::contains_key(audit_id.clone(), control_point_id.clone()) {
            let observation_ids = <ObservationOf<T>>::get(audit_id, control_point_id);
            observation_ids.contains(&observation_id)
        } else {
            false
        }
    }

    fn is_evidence_exist(audit_id: T::AuditId, evidence_id: T::EvidenceId) -> bool {
        if <Evidences<T>>::contains_key(audit_id.clone(), evidence_id.clone()) {
            true
        } else {
            false
        }
    }
}
