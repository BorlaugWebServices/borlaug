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
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use primitives::{bounded_vec::BoundedVec, *};
    use sp_runtime::{
        traits::{AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One},
        Either,
    };
    use sp_std::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + groups::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type AuditId: Parameter
            + Member
            + AtLeast32Bit
            + Copy
            + MaybeSerializeDeserialize
            + PartialEq;

        type ControlPointId: Parameter
            + Member
            + AtLeast32Bit
            + Copy
            + MaybeSerializeDeserialize
            + PartialEq;

        type EvidenceId: Parameter
            + Member
            + AtLeast32Bit
            + Copy
            + MaybeSerializeDeserialize
            + PartialEq;

        type ObservationId: Parameter
            + Member
            + AtLeast32Bit
            + Copy
            + MaybeSerializeDeserialize
            + PartialEq;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The maximum length of a name or symbol stored on-chain.
        type NameLimit: Get<u32>;
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
        AuditStarted(T::AccountId, T::AuditId),
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
        /// A string exceeds the maximum allowed length
        BadString,
        NoIdAvailable,
        AuditNotFound,
        NotAuthorized,
        AuditIsNotRequested,
        AuditIsNotInProgress,
        AuditIsNotAcceptedOrInProgress,
        AuditorIsNotValid,
        NoObservationAvailable,
        NoEvidenceAvailable,
    }

    #[pallet::type_value]
    pub fn UnitDefault<T: Config>() -> u64 {
        1u64
    }

    #[pallet::type_value]
    pub fn AuditIdDefault<T: Config>() -> T::AuditId {
        1u32.into()
    }
    #[pallet::type_value]
    pub fn ObservationIdDefault<T: Config>() -> T::ObservationId {
        1u32.into()
    }
    #[pallet::type_value]
    pub fn EvidenceIdDefault<T: Config>() -> T::EvidenceId {
        1u32.into()
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
    pub type Nonce<T> = StorageValue<_, u64, ValueQuery, UnitDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_audit_id)]
    /// The next available audit index
    pub type NextAuditId<T: Config> = StorageValue<_, T::AuditId, ValueQuery, AuditIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_observation_id)]
    /// The next available  index
    pub type NextObservationId<T: Config> =
        StorageValue<_, T::ObservationId, ValueQuery, ObservationIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_evidence_id)]
    /// The next available  index
    pub type NextEvidenceId<T: Config> =
        StorageValue<_, T::EvidenceId, ValueQuery, EvidenceIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn audits)]
    /// Audits
    pub type Audits<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AuditId, Audit<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn audits_by_creator)]
    pub type AuditsByCreator<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AuditId,
        (),
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn audits_by_auditor)]
    pub type AuditsByAuditor<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AuditId,
        (),
        OptionQuery,
    >;

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
        OptionQuery,
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
        Evidence<BoundedVec<u8, <T as Config>::NameLimit>>,
        OptionQuery,
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
        (),
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
        /// Create a new audit
        ///
        /// Arguments: None
        #[pallet::weight(<T as Config>::WeightInfo::create_audit())]
        pub fn create_audit(
            origin: OriginFor<T>,
            auditor: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let audit_id = next_id!(NextAuditId<T>, T);

            let audit = Audit {
                status: AuditStatus::Requested,
                audit_creator: sender.clone(),
                auditor: auditor.clone(),
            };

            <Audits<T>>::insert(&audit_id, audit);
            <AuditsByCreator<T>>::insert(&sender, &audit_id, ());
            <AuditsByAuditor<T>>::insert(&auditor, &audit_id, ());

            Self::deposit_event(Event::AuditCreated(sender, audit_id));
            Ok(().into())
        }

        /// Delete Requested Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[pallet::weight(<T as Config>::WeightInfo::delete_audit())]
        pub fn delete_audit(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Requested,
                <Error<T>>::AuditIsNotRequested
            );
            ensure!(
                audit.audit_creator == sender.clone(),
                <Error<T>>::NotAuthorized
            );

            <Audits<T>>::remove(&audit_id);
            <AuditsByCreator<T>>::remove(&sender, &audit_id);
            <AuditsByAuditor<T>>::remove(&audit.auditor, &audit_id);

            Self::deposit_event(Event::AuditRemoved(sender, audit_id));
            Ok(().into())
        }

        /// Auditor Accept Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[pallet::weight(<T as Config>::WeightInfo::accept_audit())]
        pub fn accept_audit(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Requested,
                <Error<T>>::AuditIsNotRequested
            );
            ensure!(
                audit.auditor == sender.clone(),
                <Error<T>>::AuditorIsNotValid
            );
            audit.status = AuditStatus::Accepted;

            <Audits<T>>::insert(audit_id, audit);

            Self::deposit_event(Event::AuditAccepted(sender, audit_id));
            Ok(().into())
        }

        /// Auditor Rejects Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[pallet::weight(<T as Config>::WeightInfo::reject_audit())]
        pub fn reject_audit(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Requested,
                <Error<T>>::AuditIsNotRequested
            );
            ensure!(
                audit.auditor == sender.clone(),
                <Error<T>>::AuditorIsNotValid
            );

            audit.status = AuditStatus::Rejected;

            <Audits<T>>::insert(audit_id, audit);

            Self::deposit_event(Event::AuditRejected(sender, audit_id));
            Ok(().into())
        }

        /// Auditor Complete Audit
        ///
        /// Arguments:
        /// - `audit_id`
        #[pallet::weight(<T as Config>::WeightInfo::complete_audit())]
        pub fn complete_audit(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(
                audit.auditor == sender.clone(),
                <Error<T>>::AuditorIsNotValid
            );

            audit.status = AuditStatus::Completed;

            <Audits<T>>::insert(audit_id, audit);

            Self::deposit_event(Event::AuditCompleted(sender, audit_id));
            Ok(().into())
        }

        /// Create New Observation
        ///
        /// Arguments:
        /// - `audit_id` id created on chain of audit
        /// - `control_point_id` control point id of audit
        /// - `observation` (compliance, procedural notes)
        #[pallet::weight(<T as Config>::WeightInfo::create_observation())]
        pub fn create_observation(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
            observation: Observation,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Accepted || audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotAcceptedOrInProgress
            );
            ensure!(
                audit.auditor == sender.clone(),
                <Error<T>>::AuditorIsNotValid
            );

            if audit.status == AuditStatus::Accepted {
                audit.status = AuditStatus::InProgress;
                <Audits<T>>::insert(audit_id, audit);
            }

            let observation_id = next_id!(NextObservationId<T>, T);

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
        #[pallet::weight(<T as Config>::WeightInfo::create_evidence(
            evidence.name.len() as u32,
            evidence.content_type.len() as u32,
            evidence.url.as_ref().map_or(0,|url|url.len()) as u32,
            evidence.hash.len() as u32,

        ))]
        pub fn create_evidence(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            evidence: Evidence<Vec<u8>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Accepted || audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotAcceptedOrInProgress
            );
            ensure!(
                audit.auditor == sender.clone(),
                <Error<T>>::AuditorIsNotValid
            );

            let bounded_content_type = enforce_limit!(evidence.content_type);
            let bounded_hash = enforce_limit!(evidence.hash);
            let bounded_name = enforce_limit!(evidence.name);
            let bounded_url = enforce_limit_option!(evidence.url);

            let evidence_id = next_id!(NextEvidenceId<T>, T);

            let evidence = Evidence {
                content_type: bounded_content_type,
                hash: bounded_hash,
                name: bounded_name,
                url: bounded_url,
            };

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
        #[pallet::weight(<T as Config>::WeightInfo::link_evidence())]
        pub fn link_evidence(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
            observation_id: T::ObservationId,
            evidence_id: T::EvidenceId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(
                audit.auditor == sender.clone(),
                <Error<T>>::AuditorIsNotValid
            );
            ensure!(
                <Observations<T>>::contains_key((audit_id, control_point_id), observation_id),
                <Error<T>>::NoObservationAvailable
            );
            ensure!(
                <Evidences<T>>::contains_key(audit_id, evidence_id),
                <Error<T>>::NoEvidenceAvailable
            );

            <EvidenceLinks<T>>::insert(&evidence_id, &observation_id, ());

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
        #[pallet::weight(<T as Config>::WeightInfo::unlink_evidence())]
        pub fn unlink_evidence(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
            observation_id: T::ObservationId,
            evidence_id: T::EvidenceId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(
                audit.auditor == sender.clone(),
                <Error<T>>::AuditorIsNotValid
            );
            ensure!(
                <Observations<T>>::contains_key((audit_id, control_point_id), observation_id),
                <Error<T>>::NoObservationAvailable
            );
            ensure!(
                <Evidences<T>>::contains_key(audit_id, evidence_id),
                <Error<T>>::NoEvidenceAvailable
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
        #[pallet::weight(<T as Config>::WeightInfo::delete_evidence())]
        pub fn delete_evidence(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            evidence_id: T::EvidenceId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(
                audit.auditor == sender.clone(),
                <Error<T>>::AuditorIsNotValid
            );

            ensure!(
                <Evidences<T>>::contains_key(audit_id, evidence_id),
                <Error<T>>::NoEvidenceAvailable
            );

            <EvidenceLinks<T>>::remove_prefix(&evidence_id);

            <Evidences<T>>::remove(&audit_id, &evidence_id);

            Self::deposit_event(Event::EvidenceDeleted(audit_id, evidence_id));
            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --

        pub fn get_audits_by_creator(
            account: T::AccountId,
        ) -> Vec<(T::AuditId, Audit<T::AccountId>)> {
            let mut audits = Vec::new();
            <AuditsByCreator<T>>::iter_prefix(account).for_each(|(audit_id, _)| {
                let audit = <Audits<T>>::get(audit_id).unwrap();
                audits.push((audit_id, audit))
            });
            audits
        }

        pub fn get_audits_by_auditor(
            account: T::AccountId,
        ) -> Vec<(T::AuditId, Audit<T::AccountId>)> {
            let mut audits = Vec::new();
            <AuditsByAuditor<T>>::iter_prefix(account).for_each(|(audit_id, _)| {
                let audit = <Audits<T>>::get(audit_id).unwrap();
                audits.push((audit_id, audit))
            });
            audits
        }

        // -- private functions --
    }
}
