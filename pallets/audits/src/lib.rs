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

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use primitives::{Audit, AuditStatus, Evidence, Observation};
    use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One};
    use sp_std::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

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

    #[pallet::event]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::AuditId = "AuditId",
        T::ObservationId = "ObservationId",
        T::ControlPointId = "ControlPointId",
        T::EvidenceId = "EvidenceId"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New registry created (owner, audit id)
        AuditCreated(T::AccountId, T::AuditId),
        /// Audit deleted (owner, audit id)
        AuditRemoved(T::AccountId, T::AuditId),
        ///
        AuditAccepted(T::AccountId, T::AuditId),
        ///
        AuditRejected(T::AccountId, T::AuditId),
        ///
        AuditCompleted(T::AccountId, T::AuditId),
        /// New observation created (audit id, control point id, observation id)
        ObservationCreated(T::AuditId, T::ControlPointId, T::ObservationId),
        /// Evidence Attached (audit id, evidence id)
        EvidenceAttached(T::AuditId, T::EvidenceId),
        /// Evidence Linked to Observation
        EvidenceLinked(T::AuditId, T::EvidenceId, T::ObservationId),
        /// Evidence Unlinked from Observation
        EvidenceUnlinked(T::AuditId, T::EvidenceId, T::ObservationId),
        /// Evidence Deleted from Audit
        EvidenceDeleted(T::AuditId, T::EvidenceId),
    }

    #[pallet::error]
    pub enum Error<T> {
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
    #[pallet::getter(fn next_audit_id)]
    /// The next available audit index
    pub type NextAuditId<T: Config> = StorageValue<_, T::AuditId>;

    #[pallet::storage]
    #[pallet::getter(fn next_observation_id)]
    /// The next available  index
    pub type NextObservationId<T: Config> = StorageValue<_, T::ObservationId>;

    #[pallet::storage]
    #[pallet::getter(fn next_evidence_id)]
    /// The next available  index
    pub type NextEvidenceId<T: Config> = StorageValue<_, T::EvidenceId>;

    #[pallet::storage]
    #[pallet::getter(fn audits)]
    /// Audits
    pub type Audits<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AuditId, Audit<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn observation_of)]
    /// Audit => (Control Point => Collection of Observation)
    pub type Observations<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::AuditId, T::ControlPointId),
        Blake2_128Concat,
        T::ObservationId,
        Observation,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn evidences)]
    /// Audit Id => (Evidence Id => Evidence(Name, Content-Type, URL, Hash))
    pub type Evidences<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AuditId,
        Blake2_128Concat,
        T::EvidenceId,
        Evidence,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn evidence_links)]
    /// Observation Id => (Evidence Id => Evidence Id)
    pub type EvidenceLinks<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::EvidenceId,
        Blake2_128Concat,
        T::ObservationId,
        T::EvidenceId,
        ValueQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new audit
        ///
        /// Arguments: None

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_audit(
            origin: OriginFor<T>,
            auditor: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let audit_id = Self::next_audit_id().unwrap();
            let next_id = audit_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextAuditId<T>>::put(next_id);

            let audit = Audit {
                status: AuditStatus::Requested,
                audit_creator: sender.clone(),
                auditor,
            };

            <Audits<T>>::insert(&audit_id, audit);

            Self::deposit_event(Event::AuditCreated(sender, audit_id));
            Ok(().into())
        }

        /// Delete Requested Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn delete_audit(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_audit_in_this_status(audit_id, AuditStatus::Requested),
                <Error<T>>::AuditIsNotRequested
            );

            ensure!(
                Self::is_audit_creator(audit_id, sender.clone()),
                <Error<T>>::AuditCreatorIsNotPresent
            );

            <Audits<T>>::remove(&audit_id);

            Self::deposit_event(Event::AuditRemoved(sender, audit_id));
            Ok(().into())
        }

        /// Auditor Accept Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn accept_audit(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_audit_in_this_status(audit_id, AuditStatus::Requested),
                <Error<T>>::AuditIsNotRequested
            );

            ensure!(
                Self::is_auditor_valid(audit_id, sender.clone()),
                <Error<T>>::AuditorIsNotValid
            );

            let mut audit = <Audits<T>>::get(audit_id);
            audit.status = AuditStatus::Accepted;

            <Audits<T>>::insert(&audit_id, audit);

            Self::deposit_event(Event::AuditAccepted(sender, audit_id));
            Ok(().into())
        }

        /// Auditor Reject Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn reject_audit(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_audit_in_this_status(audit_id, AuditStatus::Requested),
                <Error<T>>::AuditIsNotRequested
            );

            ensure!(
                Self::is_auditor_valid(audit_id, sender.clone()),
                <Error<T>>::AuditorIsNotValid
            );

            let mut audit = <Audits<T>>::get(audit_id);
            audit.status = AuditStatus::Rejected;

            <Audits<T>>::insert(&audit_id, audit);

            Self::deposit_event(Event::AuditRejected(sender, audit_id));
            Ok(().into())
        }

        /// Auditor Complete Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn complete_audit(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_audit_in_this_status(audit_id, AuditStatus::InProgress),
                <Error<T>>::AuditIsNotInProgress
            );

            ensure!(
                Self::is_auditor_valid(audit_id, sender.clone()),
                <Error<T>>::AuditorIsNotValid
            );

            let mut audit = <Audits<T>>::get(audit_id);
            audit.status = AuditStatus::Completed;

            <Audits<T>>::insert(&audit_id, audit);

            Self::deposit_event(Event::AuditCompleted(sender, audit_id));
            Ok(().into())
        }

        /// Create New Observation
        ///
        /// Arguments:
        /// - `audit_id` id created on chain of audit
        /// - `control_point_id` control point id of audit
        /// - `observation` (compliance, procedural notes)
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_observation(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
            observation: Observation,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_auditor_valid(audit_id, sender),
                <Error<T>>::AuditorIsNotValid
            );

            if Self::is_audit_in_this_status(audit_id, AuditStatus::Accepted) {
                let mut audit = <Audits<T>>::get(audit_id);
                audit.status = AuditStatus::InProgress;

                <Audits<T>>::insert(&audit_id, audit);
            }

            let observation_id = Self::next_observation_id().unwrap();
            let next_id = observation_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextObservationId<T>>::put(next_id);

            <Observations<T>>::insert((&audit_id, &control_point_id), &observation_id, observation);

            Self::deposit_event(Event::ObservationCreated(
                audit_id,
                control_point_id,
                observation_id,
            ));
            Ok(().into())
        }

        /// Attach New Evidence to Audit
        ///
        /// Arguments:
        /// - `audit_id` id of audit created on chain
        /// - `evidence` Body of evidence
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_evidence(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            evidence: Evidence,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_auditor_valid(audit_id, sender),
                <Error<T>>::AuditorIsNotValid
            );

            ensure!(
                Self::is_audit_inprogress_or_accepted(audit_id),
                <Error<T>>::AuditIsNotAcceptedOrInProgress
            );

            let evidence_id = Self::next_evidence_id().unwrap();
            let next_id = evidence_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextEvidenceId<T>>::put(next_id);

            <Evidences<T>>::insert(&audit_id, &evidence_id, evidence);

            Self::deposit_event(Event::EvidenceAttached(audit_id, evidence_id));
            Ok(().into())
        }

        /// Link Attached evidence to observation
        ///
        /// Arguments:
        /// - `audit_id` id of audit created on chain
        /// - `evidence_id` id of evidence created on chain
        /// - `observation_id` id of observation created on chain
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn link_evidence(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
            observation_id: T::ObservationId,
            evidence_id: T::EvidenceId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                Self::is_auditor_valid(audit_id, sender),
                <Error<T>>::AuditorIsNotValid
            );

            ensure!(
                Self::is_observation_exist(audit_id, control_point_id, observation_id),
                <Error<T>>::NoObservationAvailable
            );

            ensure!(
                Self::is_evidence_exist(audit_id, evidence_id),
                <Error<T>>::NoEvidenceAvailable
            );

            ensure!(
                Self::is_audit_in_this_status(audit_id, AuditStatus::InProgress),
                <Error<T>>::AuditIsNotInProgress
            );

            <EvidenceLinks<T>>::insert(&evidence_id, &observation_id, evidence_id);

            Self::deposit_event(Event::EvidenceLinked(audit_id, evidence_id, observation_id));
            Ok(().into())
        }

        /// Unlink Attached evidence from observation
        ///
        /// Arguments:
        /// - `audit_id` id of audit created on chain
        /// - `control_point_id` id of observation created on chain
        /// - `observation_id` id of observation created on chain
        /// - `evidence_id` id of evidence created on chain
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn unlink_evidence(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
            observation_id: T::ObservationId,
            evidence_id: T::EvidenceId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_auditor_valid(audit_id, sender),
                <Error<T>>::AuditorIsNotValid
            );

            ensure!(
                Self::is_observation_exist(audit_id, control_point_id, observation_id),
                <Error<T>>::NoObservationAvailable
            );

            ensure!(
                Self::is_evidence_exist(audit_id, evidence_id),
                <Error<T>>::NoEvidenceAvailable
            );

            ensure!(
                Self::is_audit_in_this_status(audit_id, AuditStatus::InProgress),
                <Error<T>>::AuditIsNotInProgress
            );

            <EvidenceLinks<T>>::remove(&evidence_id, &observation_id);

            Self::deposit_event(Event::EvidenceUnlinked(
                audit_id,
                evidence_id,
                observation_id,
            ));
            Ok(().into())
        }

        /// Delete Attached evidence from audit
        ///
        /// Arguments:
        /// - `audit_id` id of audit created on chain
        /// - `evidence_id` id of evidence created on chain
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn delete_evidence(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            evidence_id: T::EvidenceId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_auditor_valid(audit_id, sender),
                <Error<T>>::AuditorIsNotValid
            );

            ensure!(
                Self::is_evidence_exist(audit_id, evidence_id),
                <Error<T>>::NoEvidenceAvailable
            );

            ensure!(
                Self::is_audit_in_this_status(audit_id, AuditStatus::InProgress),
                <Error<T>>::AuditIsNotInProgress
            );

            <EvidenceLinks<T>>::remove_prefix(&evidence_id);

            <Evidences<T>>::remove(&audit_id, &evidence_id);

            Self::deposit_event(Event::EvidenceDeleted(audit_id, evidence_id));
            Ok(().into())
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
            if <Audits<T>>::contains_key(audit_id) {
                let audit = <Audits<T>>::get(audit_id);
                audit.audit_creator == audit_creator
            } else {
                false
            }
        }

        fn is_auditor_valid(audit_id: T::AuditId, auditor: T::AccountId) -> bool {
            if <Audits<T>>::contains_key(audit_id) {
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
            <Observations<T>>::contains_key((audit_id, control_point_id), observation_id)
        }

        fn is_evidence_exist(audit_id: T::AuditId, evidence_id: T::EvidenceId) -> bool {
            <Evidences<T>>::contains_key(audit_id, evidence_id)
        }
    }
}
