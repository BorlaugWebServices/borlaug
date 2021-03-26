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

use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, Parameter};
use frame_system::ensure_signed;
use primitives::{Audit, AuditStatus, Evidence, Observation};
use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One};
use sp_std::prelude::*;

pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

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
}

decl_event!(
    pub enum Event<T>
        where
        <T as frame_system::Config>::AccountId,
        <T as Config>::AuditId,
        <T as Config>::ObservationId,
        <T as Config>::ControlPointId,
        <T as Config>::EvidenceId,

    {
        /// New registry created (owner, audit id)
        AuditCreated(AccountId, AuditId),
        /// Audit deleted (owner, audit id)
        AuditRemoved(AccountId, AuditId),
        ///
        AuditAccepted(AccountId, AuditId),
        ///
        AuditRejected(AccountId, AuditId),
        ///
        AuditCompleted(AccountId, AuditId),
        /// New observation created (audit id, control point id, observation id)
        ObservationCreated(AuditId, ControlPointId, ObservationId),
        /// Evidence Attached (audit id, evidence id)
        EvidenceAttached(AuditId, EvidenceId),
        /// Evidence Linked to Observation
        EvidenceLinked(AuditId, EvidenceId, ObservationId),
        /// Evidence Unlinked from Observation
        EvidenceUnlinked(AuditId, EvidenceId, ObservationId),
        /// Evidence Deleted from Audit
        EvidenceDeleted(AuditId, EvidenceId),

    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Value was None
        NoneValue,
        NoIdAvailable,
        AuditCreatorIsNotPresent,
        AuditIsNotRequested,
        AuditIsNotInProgress,
        AuditIsNotAcceptedOrInProgress,
        AuditorIsNotValid,
        NoObservationAvailable,
        NoEvidenceAvailable,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as Audits {

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
            map hasher(blake2_128_concat) T::AuditId => Audit<T::AccountId>;

        /// Audit => (Control Point => Collection of Observation)
        pub Observations get(fn observation_of):
            double_map hasher(blake2_128_concat) (T::AuditId,T::ControlPointId), hasher(blake2_128_concat) T::ObservationId => Observation;

       /// Audit Id => (Evidence Id => Evidence(Name, Content-Type, URL, Hash))
       pub Evidences get(fn evidences):
            double_map hasher(blake2_128_concat) T::AuditId, hasher(blake2_128_concat) T::EvidenceId => Evidence;

       /// Observation Id => (Evidence Id => Evidence Id)
       pub EvidenceLinks get(fn evidence_links):
            double_map hasher(blake2_128_concat) T::EvidenceId , hasher(blake2_128_concat) T::ObservationId=> T::EvidenceId;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Create a new audit
        ///
        /// Arguments: None

        #[weight = 100_000]
        fn create_audit(
            origin            ,
            auditor: T::AccountId
        ) {
            let sender = ensure_signed(origin)?;

            let audit_id = Self::next_audit_id();
            let next_id = audit_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextAuditId<T>>::put(next_id);

            let audit = Audit {
                status: AuditStatus::Requested,
                audit_creator: sender.clone(),
                auditor: auditor
            };

            <Audits<T>>::insert(&audit_id, audit);

            Self::deposit_event(RawEvent::AuditCreated(sender, audit_id));
        }

        /// Delete Requested Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[weight = 100_000]
        fn delete_audit(origin,audit_id: T::AuditId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_audit_in_this_status(audit_id, AuditStatus::Requested),
            <Error<T>>::AuditIsNotRequested);

            ensure!(Self::is_audit_creator(audit_id, sender.clone()),
            <Error<T>>::AuditCreatorIsNotPresent);

            <Audits<T>>::remove(&audit_id);

            Self::deposit_event(RawEvent::AuditRemoved(sender, audit_id));
        }


        /// Auditor Accept Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[weight = 100_000]
        fn accept_audit(origin, audit_id: T::AuditId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_audit_in_this_status(audit_id, AuditStatus::Requested),
            <Error<T>>::AuditIsNotRequested);

            ensure!(Self::is_auditor_valid(audit_id, sender.clone()),
            <Error<T>>::AuditorIsNotValid);

            let mut audit = <Audits<T>>::get(audit_id);
            audit.status = AuditStatus::Accepted;

            <Audits<T>>::insert(&audit_id, audit);

            Self::deposit_event(RawEvent::AuditAccepted(sender, audit_id));
        }


        /// Auditor Reject Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[weight = 100_000]
        fn reject_audit(origin, audit_id: T::AuditId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_audit_in_this_status(audit_id, AuditStatus::Requested),
            <Error<T>>::AuditIsNotRequested);

            ensure!(Self::is_auditor_valid(audit_id, sender.clone()),
            <Error<T>>::AuditorIsNotValid);

            let mut audit = <Audits<T>>::get(audit_id);
            audit.status = AuditStatus::Rejected;

            <Audits<T>>::insert(&audit_id, audit);

            Self::deposit_event(RawEvent::AuditRejected(sender, audit_id));
        }

        /// Auditor Complete Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[weight = 100_000]
        fn complete_audit(origin, audit_id: T::AuditId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_audit_in_this_status(audit_id, AuditStatus::InProgress ),
            <Error<T>>::AuditIsNotInProgress);

            ensure!(Self::is_auditor_valid(audit_id, sender.clone()),
            <Error<T>>::AuditorIsNotValid);

            let mut audit = <Audits<T>>::get(audit_id);
            audit.status = AuditStatus::Completed;

            <Audits<T>>::insert(&audit_id, audit);

            Self::deposit_event(RawEvent::AuditCompleted(sender, audit_id));
        }


        /// Create New Observation
        ///
        /// Arguments:
        /// - `audit_id` id created on chain of audit
        /// - `control_point_id` control point id of audit
        /// - `observation` (compliance, procedural notes)
        #[weight = 100_000]
        fn create_observation(
            origin,
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
            observation: Observation,
        ) {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_auditor_valid(audit_id, sender.clone()),
                <Error<T>>::AuditorIsNotValid
            );

            if Self::is_audit_in_this_status(audit_id, AuditStatus::Accepted) {
                let mut audit = <Audits<T>>::get(audit_id);
                audit.status = AuditStatus::InProgress;

                <Audits<T>>::insert(&audit_id, audit);
            }

            let observation_id = Self::next_observation_id();
            let next_id = observation_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextObservationId<T>>::put(next_id);

            <Observations<T>>::insert((&audit_id, &control_point_id), &observation_id, observation);

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
        #[weight = 100_000]
        fn create_evidence(
            origin,
            audit_id: T::AuditId,
            evidence: Evidence
        ){
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_auditor_valid(audit_id, sender),
            <Error<T>>::AuditorIsNotValid);

            ensure!(Self::is_audit_inprogress_or_accepted(audit_id),
            <Error<T>>::AuditIsNotAcceptedOrInProgress);

            let evidence_id = Self::next_evidence_id();
            let next_id = evidence_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextEvidenceId<T>>::put(next_id);

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
        #[weight = 100_000]
        fn link_evidence(
            origin,
            audit_id:T::AuditId,
            control_point_id: T::ControlPointId,
            observation_id: T::ObservationId,
            evidence_id:T::EvidenceId,
        ){
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_auditor_valid(audit_id, sender),
            <Error<T>>::AuditorIsNotValid);

            ensure!(Self::is_observation_exist(audit_id, control_point_id, observation_id),
            <Error<T>>::NoObservationAvailable);

            ensure!(Self::is_evidence_exist(audit_id, evidence_id),
            <Error<T>>::NoEvidenceAvailable);

            ensure!(Self::is_audit_in_this_status(audit_id, AuditStatus::InProgress ),
            <Error<T>>::AuditIsNotInProgress);

            <EvidenceLinks<T>>::insert(&evidence_id,&observation_id,  evidence_id);

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
        /// - `control_point_id` id of observation created on chain
        /// - `observation_id` id of observation created on chain
        /// - `evidence_id` id of evidence created on chain
        #[weight = 100_000]
        fn unlink_evidence(
            origin,
            audit_id:T::AuditId,
            control_point_id: T::ControlPointId,
            observation_id:T::ObservationId,
            evidence_id:T::EvidenceId,
        ){
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_auditor_valid(audit_id, sender),
            <Error<T>>::AuditorIsNotValid);

            ensure!(Self::is_observation_exist(audit_id, control_point_id, observation_id),
            <Error<T>>::NoObservationAvailable);

            ensure!(Self::is_evidence_exist(audit_id, evidence_id),
            <Error<T>>::NoEvidenceAvailable);

            ensure!(Self::is_audit_in_this_status(audit_id, AuditStatus::InProgress ),
            <Error<T>>::AuditIsNotInProgress);

            <EvidenceLinks<T>>::remove(&evidence_id,&observation_id);

            Self::deposit_event(RawEvent::EvidenceUnlinked(
                audit_id,
                evidence_id,
                observation_id
            ));
        }


        /// Delete Attached evidence from audit
        ///
        /// Arguments:
        /// - `audit_id` id of audit created on chain
        /// - `evidence_id` id of evidence created on chain
        #[weight = 100_000]
        fn delete_evidence(
            origin,
            audit_id:T::AuditId,
            evidence_id:T::EvidenceId,
        ){
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_auditor_valid(audit_id, sender),
            <Error<T>>::AuditorIsNotValid);

            ensure!(Self::is_evidence_exist(audit_id, evidence_id),
            <Error<T>>::NoEvidenceAvailable);

            ensure!(Self::is_audit_in_this_status(audit_id, AuditStatus::InProgress ),
            <Error<T>>::AuditIsNotInProgress);

            <EvidenceLinks<T>>::remove_prefix(&evidence_id);

            <Evidences<T>>::remove(&audit_id,&evidence_id);

            Self::deposit_event(RawEvent::EvidenceDeleted(
                audit_id,
                evidence_id
            ));
        }
    }
}

// private functions
impl<T: Config> Module<T> {
    fn is_audit_in_this_status(audit_id: T::AuditId, status: AuditStatus) -> bool {
        if <Audits<T>>::contains_key(audit_id) {
            let audit = <Audits<T>>::get(audit_id);
            audit.status == status
        } else {
            false
        }
    }

    fn is_audit_inprogress_or_accepted(audit_id: T::AuditId) -> bool {
        if <Audits<T>>::contains_key(audit_id) {
            let audit = <Audits<T>>::get(audit_id);
            audit.status == AuditStatus::Accepted || audit.status == AuditStatus::InProgress
        } else {
            false
        }
    }

    fn is_audit_creator(audit_id: T::AuditId, audit_creator: T::AccountId) -> bool {
        if <Audits<T>>::contains_key(audit_id.clone()) {
            let audit = <Audits<T>>::get(audit_id);
            audit.audit_creator == audit_creator
        } else {
            false
        }
    }

    fn is_auditor_valid(audit_id: T::AuditId, auditor: T::AccountId) -> bool {
        if <Audits<T>>::contains_key(audit_id.clone()) {
            let audit = <Audits<T>>::get(audit_id);
            audit.auditor == auditor
        } else {
            false
        }
    }

    fn is_observation_exist(
        audit_id: T::AuditId,
        control_point_id: T::ControlPointId,
        observation_id: T::ObservationId,
    ) -> bool {
        if <Observations<T>>::contains_key(
            (audit_id.clone(), control_point_id.clone()),
            observation_id,
        ) {
            true
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
