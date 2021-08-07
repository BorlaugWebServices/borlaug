//! # Audit Module
//!
//! ## Overview
//!
//! Audits are created by an Organization which then requests and Auditing Organization to carry out the Audit.
//! The Auditing Organization then assigns one or more Auditors to the audit.
//! All routes may be called by either an individual account or a group from the Groups module. When groups are used, the group threshold is used.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create_audit` - Creates a new audit. Assigning an Auditing Organization is done as part of creation. Auditing Organization cannot be changed.
//!     Used by: Audit Creator
//! * `delete_audit` - Delete an audit. Audits can only be deleted before they have been accepted by an Auditing Organization
//!     Used by: Audit Creator
//! * `accept_audit` - The Auditing Organization accepts an Audit
//!     Used by: Auditing Organization
//! * `assign_auditors` - The Auditing Organization assigns and auditor or auditors. Use a group or subgroup when assigning multiple auditors.
//!     Used by: Auditing Organization
//! * `reject_audit` - The Auditing Organization rejects an Audit
//!     Used by: Auditing Organization
//! * `complete_audit` - The Auditing Organization completes an Audit
//!     Used by: Auditing Organization
//! * `create_observation` - An Auditor creates an observation
//!     Used by: Auditors
//! * `create_evidence` - An Auditor creates an item of evidence
//!     Used by: Auditors
//! * `link_evidence` - An Auditor links evidence to an observation
//!     Used by: Auditors
//! * `unlink_evidence` - An Auditor removes a link between evidence and an observation
//!     Used by: Auditors
//! * `delete_evidence` - An Auditor removes a link between evidence and an observation
//!     Used by: Auditors
//!
//! ### RPC Methods
//!
//! * `get_audits_by_creator` - Get the collection of audits by Audit Creator
//! * `get_audits_by_auditing_org` - Get the collection of audits by Auditing Organization
//! * `get_audits_by_auditors` - Get the collection of audits by Auditors
//! * `get_audit` - Get an audit by Audit Id
//! * `get_observation_by_control_point` - Get the collection of observations by Control Point
//! * `get_evidence` - Get the collection of evidence for an Audit
//! * `get_observation_by_control_point` - Get the collection of observations by Control Point
//! * `get_evidence_links_by_evidence` - Get the collection of observations linked to an item of evidence
//! * `get_evidence_links_by_observation` - Get the collection of evidence linked to an observation

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
    use extrinsic_extra::GetExtrinsicExtra;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use primitives::{bounded_vec::BoundedVec, *};
    use sp_runtime::{
        traits::{AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One},
        Either,
    };
    use sp_std::prelude::*;

    const MODULE_INDEX: u8 = 5;

    #[repr(u8)]
    pub enum ExtrinsicIndex {
        Audit = 51,
        Observation = 52,
        Evidence = 53,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + groups::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        ///A unique id for each audit. Serial generated on chain.
        type AuditId: Parameter
            + Member
            + AtLeast32Bit
            + Copy
            + MaybeSerializeDeserialize
            + PartialEq;
        ///An identifying id for each control point. Provided by the caller. Only needs to be unique per audit.
        type ControlPointId: Parameter
            + Member
            + AtLeast32Bit
            + Copy
            + MaybeSerializeDeserialize
            + PartialEq;
        ///An identifying id for each evidence item. Serial generated on chain.
        type EvidenceId: Parameter
            + Member
            + AtLeast32Bit
            + Copy
            + MaybeSerializeDeserialize
            + PartialEq;
        ///An identifying id for each observation. Serial generated on chain.
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
        /// The maximum number of evidence_links that can be removed in one attempt when deleting evidence.
        type MaxLinkRemove: Get<u32>;
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
        /// New registry created (owner, audit_id)
        AuditCreated(T::AccountId, T::AuditId),
        /// Audit deleted (owner, audit_id)
        AuditRemoved(T::AccountId, T::AuditId),
        /// Audit was accepted (auditing_org, audit_id)
        AuditAccepted(T::AccountId, T::AuditId),
        /// Audit was rejected (auditing_org, audit_id)
        AuditRejected(T::AccountId, T::AuditId),
        /// Audit was accepted (auditing_org, audit_id, auditors)
        AuditorsAssigned(T::AccountId, T::AuditId, T::AccountId),
        /// Audit was started (auditors, audit_id)
        AuditStarted(T::AccountId, T::AuditId),
        /// Audit was completed (auditing_org, audit_id)
        AuditCompleted(T::AccountId, T::AuditId),
        /// New observation created (auditors, auditor,audit_id, control_point_id, observation_id)
        ObservationCreated(
            T::AccountId,
            T::AccountId,
            T::AuditId,
            T::ControlPointId,
            T::ObservationId,
        ),
        /// Evidence Attached (auditors, audit_id, evidence_id)
        EvidenceAttached(T::AccountId, T::AuditId, T::EvidenceId),
        /// Evidence Linked to Observation (auditors, audit_id, evidence_id, observation_id)
        EvidenceLinked(T::AccountId, T::AuditId, T::EvidenceId, T::ObservationId),
        /// Evidence Unlinked from Observation (auditors, audit_id, evidence_id, observation_id)
        EvidenceUnlinked(T::AccountId, T::AuditId, T::EvidenceId, T::ObservationId),
        /// Evidence Deleted from Audit (auditors, audit_id, evidence_id)       
        EvidenceDeleted(T::AccountId, T::AuditId, T::EvidenceId),
        /// Evidence could not be deleted due to too many observation links. Call delete_evidence again. (auditors, audit_id, evidence_id)
        EvidenceDeleteFailed(T::AccountId, T::AuditId, T::EvidenceId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A string exceeds the maximum allowed length
        BadString,
        /// A sequential id exceeded its upper bound. Please report this to chain council.
        NoIdAvailable,
        /// The audit does not exist
        AuditNotFound,
        /// The caller must be the audit creator to execute this action.
        NotCreator,
        /// The audit must be in the `Requested` state.
        AuditIsNotRequested,
        /// The audit must be in the `InProgress` state.
        AuditIsNotInProgress,
        /// The audit must be in the `Accepted` or `InProgress` state.
        AuditIsNotAcceptedOrInProgress,
        /// The audit does not have an auditor assigned
        AuditorNotAssigned,
        /// The caller must be the auditor to execute this action.
        NotAuditor,
        /// The Observation does not exist
        ObservationNotFound,
        /// The Evidence does not exist
        EvidenceNotFound,
        /// The max Evidence link limit was exceeded
        RemoveLinkLimitExceeded,
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
    /// Incrementing nonce
    pub type Nonce<T> = StorageValue<_, u64, ValueQuery, UnitDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_audit_id)]
    /// The next available audit index
    pub type NextAuditId<T: Config> = StorageValue<_, T::AuditId, ValueQuery, AuditIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_observation_id)]
    /// The next available observation index
    pub type NextObservationId<T: Config> =
        StorageValue<_, T::ObservationId, ValueQuery, ObservationIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_evidence_id)]
    /// The next available evidence index
    pub type NextEvidenceId<T: Config> =
        StorageValue<_, T::EvidenceId, ValueQuery, EvidenceIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn audits)]
    /// Audits by audit_id
    pub type Audits<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AuditId, Audit<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn audits_by_creator)]
    /// Audits by creator
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
    #[pallet::getter(fn audits_by_auditing_org)]
    /// Audits by auditing organization
    pub type AuditsByAuditingOrg<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AuditId,
        (),
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn audits_by_auditors)]
    /// Audits by auditors
    pub type AuditsByAuditors<T: Config> = StorageDoubleMap<
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
    /// audit_id, control_point => Observation
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
    /// audit_id, evidence_id => Evidence(Name, Content-Type, URL, Hash))
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
    #[pallet::getter(fn evidence_links_by_evidence)]
    /// observation_id, evidence_id => ()
    pub type EvidenceLinksByEvidence<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::EvidenceId,
        Blake2_128Concat,
        T::ObservationId,
        (),
        OptionQuery,
    >;
    #[pallet::storage]
    #[pallet::getter(fn evidence_links_by_observation)]
    /// observation_id, evidence_id => ()
    pub type EvidenceLinksByObservation<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::ObservationId,
        Blake2_128Concat,
        T::EvidenceId,
        (),
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new audit
        ///
        /// Arguments:
        /// `auditing_org` : account_id of the auditing_org (can be an individual account or a group)
        #[pallet::weight(<T as Config>::WeightInfo::create_audit())]
        pub fn create_audit(
            origin: OriginFor<T>,
            auditing_org: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_threshold!(origin);

            let audit_id = next_id!(NextAuditId<T>, T);

            let audit = Audit {
                status: AuditStatus::Requested,
                audit_creator: sender.clone(),
                auditing_org: auditing_org.clone(),
                auditors: None,
            };

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Audit as u8),
                &sender,
            );

            <Audits<T>>::insert(&audit_id, audit);
            <AuditsByCreator<T>>::insert(&sender, &audit_id, ());
            <AuditsByAuditingOrg<T>>::insert(&auditing_org, &audit_id, ());

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
            let sender = ensure_account_or_threshold!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Requested,
                <Error<T>>::AuditIsNotRequested
            );
            ensure!(
                audit.audit_creator == sender.clone(),
                <Error<T>>::NotCreator
            );

            <Audits<T>>::remove(&audit_id);
            <AuditsByCreator<T>>::remove(&sender, &audit_id);
            <AuditsByAuditingOrg<T>>::remove(&audit.auditing_org, &audit_id);

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
            let sender = ensure_account_or_threshold!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Requested,
                <Error<T>>::AuditIsNotRequested
            );
            ensure!(audit.auditing_org == sender.clone(), <Error<T>>::NotAuditor);
            audit.status = AuditStatus::Accepted;

            <Audits<T>>::insert(audit_id, audit);

            Self::deposit_event(Event::AuditAccepted(sender, audit_id));
            Ok(().into())
        }

        /// Auditor Assigns Auditors
        ///
        /// Arguments:
        /// - `audit_id` the Audit
        /// - `auditors` the account or group account of the auditors that will create observations/evidence

        #[pallet::weight(<T as Config>::WeightInfo::assign_auditors_initial_assign().max(<T as Config>::WeightInfo::assign_auditors_replace()))]
        pub fn assign_auditors(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            auditors: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_threshold!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Accepted || audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotAcceptedOrInProgress
            );
            ensure!(audit.auditing_org == sender.clone(), <Error<T>>::NotAuditor);

            let replacing = audit.auditors.is_some();
            if replacing {
                <AuditsByAuditors<T>>::remove(&audit.auditors.unwrap(), &audit_id);
            }

            audit.auditors = Some(auditors.clone());

            <Audits<T>>::insert(audit_id, audit);

            <AuditsByAuditors<T>>::insert(&auditors, &audit_id, ());

            Self::deposit_event(Event::AuditorsAssigned(sender, audit_id, auditors));

            Ok((Some(if replacing {
                <T as Config>::WeightInfo::assign_auditors_replace()
            } else {
                <T as Config>::WeightInfo::assign_auditors_initial_assign()
            }))
            .into())
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
            let sender = ensure_account_or_threshold!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Requested,
                <Error<T>>::AuditIsNotRequested
            );
            ensure!(audit.auditing_org == sender.clone(), <Error<T>>::NotAuditor);

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
            let sender = ensure_account_or_threshold!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(audit.auditing_org == sender.clone(), <Error<T>>::NotAuditor);

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
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Accepted || audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotAcceptedOrInProgress
            );
            ensure!(audit.auditors.is_some(), <Error<T>>::AuditorNotAssigned);
            ensure!(
                *audit.auditors.as_ref().unwrap() == group_account.clone(),
                <Error<T>>::NotAuditor
            );

            if audit.status == AuditStatus::Accepted {
                audit.status = AuditStatus::InProgress;
                <Audits<T>>::insert(audit_id, audit);
            }

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Observation as u8),
                &group_account,
            );

            let observation_id = next_id!(NextObservationId<T>, T);

            <Observations<T>>::insert((&audit_id, &control_point_id), &observation_id, observation);

            Self::deposit_event(Event::ObservationCreated(
                group_account,
                account_id,
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
            let sender = ensure_account_or_threshold!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Accepted || audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotAcceptedOrInProgress
            );
            ensure!(audit.auditors.is_some(), <Error<T>>::AuditorNotAssigned);
            ensure!(
                audit.auditors.unwrap() == sender.clone(),
                <Error<T>>::NotAuditor
            );

            let bounded_content_type = enforce_limit!(evidence.content_type);
            let bounded_hash = enforce_limit!(evidence.hash);
            let bounded_name = enforce_limit!(evidence.name);
            let bounded_url = enforce_limit_option!(evidence.url);

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Evidence as u8),
                &sender,
            );

            let evidence_id = next_id!(NextEvidenceId<T>, T);

            let evidence = Evidence {
                content_type: bounded_content_type,
                hash: bounded_hash,
                name: bounded_name,
                url: bounded_url,
            };

            <Evidences<T>>::insert(&audit_id, &evidence_id, evidence);

            Self::deposit_event(Event::EvidenceAttached(sender, audit_id, evidence_id));
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
            let sender = ensure_account_or_threshold!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(audit.auditors.is_some(), <Error<T>>::AuditorNotAssigned);
            ensure!(
                audit.auditors.unwrap() == sender.clone(),
                <Error<T>>::NotAuditor
            );
            ensure!(
                <Observations<T>>::contains_key((audit_id, control_point_id), observation_id),
                <Error<T>>::ObservationNotFound
            );
            ensure!(
                <Evidences<T>>::contains_key(audit_id, evidence_id),
                <Error<T>>::EvidenceNotFound
            );

            <EvidenceLinksByEvidence<T>>::insert(evidence_id, observation_id, ());
            <EvidenceLinksByObservation<T>>::insert(observation_id, evidence_id, ());

            Self::deposit_event(Event::EvidenceLinked(
                sender,
                audit_id,
                evidence_id,
                observation_id,
            ));
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
            let sender = ensure_account_or_threshold!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(audit.auditors.is_some(), <Error<T>>::AuditorNotAssigned);
            ensure!(
                audit.auditors.unwrap() == sender.clone(),
                <Error<T>>::NotAuditor
            );
            ensure!(
                <Observations<T>>::contains_key((audit_id, control_point_id), observation_id),
                <Error<T>>::ObservationNotFound
            );
            ensure!(
                <Evidences<T>>::contains_key(audit_id, evidence_id),
                <Error<T>>::EvidenceNotFound
            );

            <EvidenceLinksByEvidence<T>>::remove(evidence_id, observation_id);
            <EvidenceLinksByObservation<T>>::remove(observation_id, evidence_id);

            Self::deposit_event(Event::EvidenceUnlinked(
                sender,
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
        /// - `link_count` a declaration of how many links the evidence has (for weight estimation)
        #[pallet::weight(<T as Config>::WeightInfo::delete_evidence(*link_count))]
        pub fn delete_evidence(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            evidence_id: T::EvidenceId,
            link_count: u32,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_threshold!(origin);

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(audit.auditors.is_some(), <Error<T>>::AuditorNotAssigned);
            ensure!(
                audit.auditors.unwrap() == sender.clone(),
                <Error<T>>::NotAuditor
            );
            ensure!(
                <Evidences<T>>::contains_key(audit_id, evidence_id),
                <Error<T>>::EvidenceNotFound
            );
            ensure!(
                link_count <= <T as Config>::MaxLinkRemove::get(),
                <Error<T>>::RemoveLinkLimitExceeded
            );

            let mut i = 0;
            for (observation_id, _) in <EvidenceLinksByEvidence<T>>::drain_prefix(evidence_id) {
                if i >= link_count {
                    Self::deposit_event(Event::EvidenceDeleteFailed(sender, audit_id, evidence_id));
                    return Ok(().into());
                }
                <EvidenceLinksByObservation<T>>::remove(observation_id, evidence_id);
                i = i + 1;
            }

            <Evidences<T>>::remove(&audit_id, &evidence_id);

            Self::deposit_event(Event::EvidenceDeleted(sender, audit_id, evidence_id));
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

        pub fn get_audits_by_auditing_org(
            account: T::AccountId,
        ) -> Vec<(T::AuditId, Audit<T::AccountId>)> {
            let mut audits = Vec::new();
            <AuditsByAuditingOrg<T>>::iter_prefix(account).for_each(|(audit_id, _)| {
                let audit = <Audits<T>>::get(audit_id).unwrap();
                audits.push((audit_id, audit))
            });
            audits
        }

        pub fn get_audits_by_auditors(
            account: T::AccountId,
        ) -> Vec<(T::AuditId, Audit<T::AccountId>)> {
            let mut audits = Vec::new();
            <AuditsByAuditors<T>>::iter_prefix(account).for_each(|(audit_id, _)| {
                let audit = <Audits<T>>::get(audit_id).unwrap();
                audits.push((audit_id, audit))
            });
            audits
        }

        pub fn get_audit(audit_id: T::AuditId) -> Option<Audit<T::AccountId>> {
            <Audits<T>>::get(audit_id)
        }

        pub fn get_observation(
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
            observation_id: T::ObservationId,
        ) -> Option<Observation> {
            <Observations<T>>::get((audit_id, control_point_id), observation_id)
        }

        pub fn get_observation_by_control_point(
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
        ) -> Vec<(T::ObservationId, Observation)> {
            let mut observations = Vec::new();
            <Observations<T>>::iter_prefix((audit_id, control_point_id)).for_each(
                |(observation_id, observation)| observations.push((observation_id, observation)),
            );
            observations
        }

        pub fn get_evidence(
            audit_id: T::AuditId,
            evidence_id: T::EvidenceId,
        ) -> Option<Evidence<BoundedVec<u8, <T as Config>::NameLimit>>> {
            <Evidences<T>>::get(audit_id, evidence_id)
        }

        pub fn get_evidence_by_audit(
            audit_id: T::AuditId,
        ) -> Vec<(
            T::EvidenceId,
            Evidence<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut evidences = Vec::new();
            <Evidences<T>>::iter_prefix(audit_id)
                .for_each(|(evidence_id, evidence)| evidences.push((evidence_id, evidence)));
            evidences
        }

        pub fn get_evidence_links_by_evidence(evidence_id: T::EvidenceId) -> Vec<T::ObservationId> {
            let mut evidence_links = Vec::new();
            <EvidenceLinksByEvidence<T>>::iter_prefix(evidence_id)
                .for_each(|(observation_id, _)| evidence_links.push(observation_id));
            evidence_links
        }

        pub fn get_evidence_links_by_observation(
            observation_id: T::ObservationId,
        ) -> Vec<T::EvidenceId> {
            let mut evidence_links = Vec::new();
            <EvidenceLinksByObservation<T>>::iter_prefix(observation_id)
                .for_each(|(evidence_id, _)| evidence_links.push(evidence_id));
            evidence_links
        }

        // -- private functions --
    }
}
