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
//! #### For the Audit Creator
//! * `create_audit` - Creates a new audit. Assigning an Auditing Organization is done as part of creation. Auditing Organization cannot be changed.
//! * `delete_audit` - Delete an audit. Audits can only be deleted before they have been accepted by an Auditing Organization
//!
//! #### For the Auditing Organization
//! * `accept_audit` - The Auditing Organization accepts an Audit
//! * `assign_auditors` - The Auditing Organization assigns and auditor or auditors. Use a group or subgroup when assigning multiple auditors.
//! * `reject_audit` - The Auditing Organization rejects an Audit
//! * `complete_audit` - The Auditing Organization completes an Audit
//! * `create_observation` - An Auditor creates an observation
//!
//! #### For the Auditors
//! * `create_evidence` - An Auditor creates an item of evidence
//! * `link_evidence` - An Auditor links evidence to an observation
//! * `unlink_evidence` - An Auditor removes a link between evidence and an observation
//! * `delete_evidence` - An Auditor removes a link between evidence and an observation
//! * `link_audit` - An Auditor links to another audit
//! * `unlink_audit` - An Auditor removes a link to another audit
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

pub mod migration;

#[frame_support::pallet]
pub mod pallet {

    pub use super::weights::WeightInfo;
    use core::convert::TryInto;
    use extrinsic_extra::GetExtrinsicExtra;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use primitives::{bounded_vec::BoundedVec, *};
    use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One};
    use sp_std::prelude::*;

    const MODULE_INDEX: u8 = 5;

    #[repr(u8)]
    pub enum ExtrinsicIndex {
        Audit = 51,
        Observation = 52,
        Evidence = 53,
    }

    #[derive(Encode, Decode, Clone, frame_support::RuntimeDebug, PartialEq)]
    pub enum Releases {
        V1,
        V2,
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
        /// The maximum length of a url (evidence).
        type UrlLimit: Get<u32>;
        /// The maximum number of evidence_links that can be removed in one attempt when deleting evidence.
        type MaxLinkRemove: Get<u32>;
    }

    #[pallet::event]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::AuditId = "AuditId",
        T::ObservationId = "ObservationId",
        T::ControlPointId = "ControlPointId",
        T::EvidenceId = "EvidenceId",
        T::ProposalId = "ProposalId"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New registry created (auditing_org, proposal_id, audit_id)
        AuditCreated(T::AccountId, T::ProposalId, T::AuditId),
        /// Audit deleted (auditing_org, proposal_id, audit_id)
        AuditRemoved(T::AccountId, T::ProposalId, T::AuditId),
        /// Audit was accepted (auditing_org, proposal_id, audit_id)
        AuditAccepted(T::AccountId, T::ProposalId, T::AuditId),
        /// Audit was rejected (auditing_org, proposal_id, audit_id)
        AuditRejected(T::AccountId, T::ProposalId, T::AuditId),
        /// Audit was accepted (auditing_org, proposal_id, audit_id, auditors)
        AuditorsAssigned(T::AccountId, T::ProposalId, T::AuditId, T::AccountId),
        /// Audit was started (auditors, proposal_id, audit_id)
        AuditStarted(T::AccountId, T::ProposalId, T::AuditId),
        /// Audit was completed (auditing_org, proposal_id, audit_id)
        AuditCompleted(T::AccountId, T::ProposalId, T::AuditId),
        /// Audit was linked (auditing_org, parent_audit_id, child_audit_id)
        AuditLinked(T::AccountId, T::AuditId, T::AuditId),
        /// Audit was unlinked (auditing_org, parent_audit_id, child_audit_id)
        AuditUnlinked(T::AccountId, T::AuditId, T::AuditId),
        /// New observation created (auditors, proposal_id, audit_id, control_point_id, observation_id)
        ObservationCreated(
            T::AccountId,
            T::ProposalId,
            T::AuditId,
            T::ControlPointId,
            T::ObservationId,
        ),
        /// Evidence Attached (auditors, proposal_id, audit_id, evidence_id)
        EvidenceAttached(T::AccountId, T::ProposalId, T::AuditId, T::EvidenceId),
        /// Evidence Linked to Observation (auditors, proposal_id, audit_id, evidence_id, observation_id)
        EvidenceLinked(
            T::AccountId,
            T::ProposalId,
            T::AuditId,
            T::EvidenceId,
            T::ObservationId,
        ),
        /// Evidence Unlinked from Observation (auditors, proposal_id, audit_id, evidence_id, observation_id)
        EvidenceUnlinked(
            T::AccountId,
            T::ProposalId,
            T::AuditId,
            T::EvidenceId,
            T::ObservationId,
        ),
        /// Evidence Deleted from Audit (auditors, proposal_id, audit_id, evidence_id)      
        EvidenceDeleted(T::AccountId, T::ProposalId, T::AuditId, T::EvidenceId),
        /// Evidence could not be deleted due to too many observation links. Call delete_evidence again. (auditors, proposal_id, audit_id, evidence_id)
        EvidenceDeleteFailed(T::AccountId, T::ProposalId, T::AuditId, T::EvidenceId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A string exceeds the maximum allowed length
        StringLengthLimitExceeded,
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
        /// The audit must be in the `Accepted` or `InProgress` or `Completed` state.
        AuditIsNotAcceptedOrInProgressOrCompleted,
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
        /// The maximum allowed url length was exceeded
        UrLLimitExceeded,
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
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            //forgot to update version number so removed
            // super::migration::migrate_to_v2::<T>()
            0
        }
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        phantom: PhantomData<T>,
    }
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                phantom: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <StorageVersion<T>>::put(Releases::V2);
        }
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    /// Storage version of the pallet.
    ///
    /// V2 - added proposal_id to observation struct
    pub type StorageVersion<T> = StorageValue<_, Releases, OptionQuery>;

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
    /// Audit by audit_id
    pub type Audits<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AuditId,
        Audit<T::AccountId, T::ProposalId>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn linked_audits)]
    /// Linked audits
    pub type LinkedAudits<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AuditId,
        Blake2_128Concat,
        T::AuditId,
        (),
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn audit_by_proposal)]
    /// Audit by proposal_id
    pub type AuditByProposal<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ProposalId, T::AuditId, OptionQuery>;

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
    #[pallet::getter(fn observation_by_proposal)]
    /// Observation by proposal_id
    pub type ObservationByProposal<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::ProposalId,
        (T::AuditId, T::ControlPointId, T::ObservationId),
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
        Observation<T::ProposalId>,
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
        Evidence<
            T::ProposalId,
            BoundedVec<u8, <T as Config>::NameLimit>,
            BoundedVec<u8, <T as Config>::UrlLimit>,
        >,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn evidence_by_proposal)]
    /// Evidence by proposal_id
    pub type EvidenceByProposal<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ProposalId, (T::AuditId, T::EvidenceId), OptionQuery>;

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
        /// `unique_ref` : a reference to ensure that proposals remain unique.
        #[pallet::weight(<T as Config>::WeightInfo::create_audit())]
        pub fn create_audit(
            origin: OriginFor<T>,
            auditing_org: T::AccountId,
            _unique_ref: u32,
        ) -> DispatchResultWithPostInfo {
            let (_, proposal_id, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let audit_id = next_id!(NextAuditId<T>, T);

            let audit = Audit {
                proposal_id,
                status: AuditStatus::Requested,
                audit_creator: group_account.clone(),
                auditing_org: auditing_org.clone(),
                auditors: None,
            };

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Audit as u8),
                &group_account,
            );

            <Audits<T>>::insert(&audit_id, audit);
            <AuditByProposal<T>>::insert(&proposal_id, audit_id);
            <AuditsByCreator<T>>::insert(&group_account, &audit_id, ());
            <AuditsByAuditingOrg<T>>::insert(&auditing_org, &audit_id, ());

            Self::deposit_event(Event::AuditCreated(group_account, proposal_id, audit_id));
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
            let (_, proposal_id, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Requested,
                <Error<T>>::AuditIsNotRequested
            );
            ensure!(audit.audit_creator == group_account, <Error<T>>::NotCreator);

            <Audits<T>>::remove(&audit_id);
            <AuditByProposal<T>>::remove(&proposal_id);
            <AuditsByCreator<T>>::remove(&group_account, &audit_id);
            <AuditsByAuditingOrg<T>>::remove(&audit.auditing_org, &audit_id);

            Self::deposit_event(Event::AuditRemoved(group_account, proposal_id, audit_id));
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
            let (_, proposal_id, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Requested,
                <Error<T>>::AuditIsNotRequested
            );
            ensure!(audit.auditing_org == group_account, <Error<T>>::NotAuditor);
            audit.status = AuditStatus::Accepted;

            <Audits<T>>::insert(audit_id, audit);

            Self::deposit_event(Event::AuditAccepted(group_account, proposal_id, audit_id));
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
            let (_, proposal_id, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Accepted || audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotAcceptedOrInProgress
            );
            ensure!(audit.auditing_org == group_account, <Error<T>>::NotAuditor);

            let replacing = audit.auditors.is_some();
            if replacing {
                <AuditsByAuditors<T>>::remove(&audit.auditors.unwrap(), &audit_id);
            }

            audit.auditors = Some(auditors.clone());

            <Audits<T>>::insert(audit_id, audit);

            <AuditsByAuditors<T>>::insert(&auditors, &audit_id, ());

            Self::deposit_event(Event::AuditorsAssigned(
                group_account,
                proposal_id,
                audit_id,
                auditors,
            ));

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
            let (_, proposal_id, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Requested,
                <Error<T>>::AuditIsNotRequested
            );
            ensure!(audit.auditing_org == group_account, <Error<T>>::NotAuditor);

            audit.status = AuditStatus::Rejected;

            <Audits<T>>::insert(audit_id, audit);

            Self::deposit_event(Event::AuditRejected(group_account, proposal_id, audit_id));
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
            let (_, proposal_id, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(audit.auditing_org == group_account, <Error<T>>::NotAuditor);

            audit.status = AuditStatus::Completed;

            <Audits<T>>::insert(audit_id, audit);

            Self::deposit_event(Event::AuditCompleted(group_account, proposal_id, audit_id));
            Ok(().into())
        }

        /// Link Audit
        ///
        /// Arguments:
        /// - `parent_audit_id`
        /// - `child_audit_id`
        #[pallet::weight(<T as Config>::WeightInfo::link_audit())]
        pub fn link_audit(
            origin: OriginFor<T>,
            parent_audit_id: T::AuditId,
            child_audit_id: T::AuditId,
        ) -> DispatchResultWithPostInfo {
            let (_, _, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_parent_audit = <Audits<T>>::get(parent_audit_id);
            ensure!(maybe_parent_audit.is_some(), <Error<T>>::AuditNotFound);
            let parent_audit = maybe_parent_audit.unwrap();
            ensure!(
                parent_audit.auditing_org == group_account,
                <Error<T>>::NotAuditor
            );
            ensure!(
                parent_audit.status == AuditStatus::Accepted
                    || parent_audit.status == AuditStatus::InProgress
                    || parent_audit.status == AuditStatus::Completed,
                <Error<T>>::AuditIsNotAcceptedOrInProgressOrCompleted
            );

            let maybe_child_audit = <Audits<T>>::get(child_audit_id);
            ensure!(maybe_child_audit.is_some(), <Error<T>>::AuditNotFound);
            let child_audit = maybe_child_audit.unwrap();
            ensure!(
                child_audit.auditing_org == group_account,
                <Error<T>>::NotAuditor
            );
            ensure!(
                child_audit.status == AuditStatus::Accepted
                    || child_audit.status == AuditStatus::InProgress
                    || child_audit.status == AuditStatus::Completed,
                <Error<T>>::AuditIsNotAcceptedOrInProgressOrCompleted
            );

            <LinkedAudits<T>>::insert(parent_audit_id, child_audit_id, ());

            Self::deposit_event(Event::AuditLinked(
                group_account,
                parent_audit_id,
                child_audit_id,
            ));
            Ok(().into())
        }
        /// Unlink Audit
        ///
        /// Arguments:
        /// - `parent_audit_id`
        /// - `child_audit_id`
        #[pallet::weight(<T as Config>::WeightInfo::unlink_audit())]
        pub fn unlink_audit(
            origin: OriginFor<T>,
            parent_audit_id: T::AuditId,
            child_audit_id: T::AuditId,
        ) -> DispatchResultWithPostInfo {
            let (_, _, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_parent_audit = <Audits<T>>::get(parent_audit_id);
            ensure!(maybe_parent_audit.is_some(), <Error<T>>::AuditNotFound);
            let parent_audit = maybe_parent_audit.unwrap();
            ensure!(
                parent_audit.auditing_org == group_account,
                <Error<T>>::NotAuditor
            );

            let maybe_child_audit = <Audits<T>>::get(child_audit_id);
            ensure!(maybe_child_audit.is_some(), <Error<T>>::AuditNotFound);
            let child_audit = maybe_child_audit.unwrap();
            ensure!(
                child_audit.auditing_org == group_account,
                <Error<T>>::NotAuditor
            );

            <LinkedAudits<T>>::remove(parent_audit_id, child_audit_id);

            Self::deposit_event(Event::AuditUnlinked(
                group_account,
                parent_audit_id,
                child_audit_id,
            ));
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
            compliance: Option<Compliance>,
            procedural_note_hash: Option<[u8; 32]>,
        ) -> DispatchResultWithPostInfo {
            let (_, proposal_id, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let mut audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Accepted || audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotAcceptedOrInProgress
            );
            ensure!(audit.auditors.is_some(), <Error<T>>::AuditorNotAssigned);
            ensure!(
                *audit.auditors.as_ref().unwrap() == group_account,
                <Error<T>>::NotAuditor
            );

            if audit.status == AuditStatus::Accepted {
                Self::deposit_event(Event::AuditStarted(
                    group_account.clone(),
                    proposal_id,
                    audit_id,
                ));

                audit.status = AuditStatus::InProgress;
                <Audits<T>>::insert(audit_id, audit);
            }

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Observation as u8),
                &group_account,
            );

            let observation_id = next_id!(NextObservationId<T>, T);

            let observation = Observation {
                proposal_id,
                compliance,
                procedural_note_hash,
            };

            <Observations<T>>::insert((&audit_id, &control_point_id), &observation_id, observation);
            <ObservationByProposal<T>>::insert(
                &proposal_id,
                (audit_id, control_point_id, observation_id),
            );

            Self::deposit_event(Event::ObservationCreated(
                group_account,
                proposal_id,
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
            name.len() as u32,
            content_type.len() as u32,
            url.as_ref().map_or(0,|url|url.len()) as u32,
            hash.len() as u32,

        ))]
        pub fn create_evidence(
            origin: OriginFor<T>,
            audit_id: T::AuditId,
            name: Vec<u8>,
            content_type: Vec<u8>,
            url: Option<Vec<u8>>,
            //TODO: use [u8; 32]
            hash: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let (_, proposal_id, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::Accepted || audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotAcceptedOrInProgress
            );
            ensure!(audit.auditors.is_some(), <Error<T>>::AuditorNotAssigned);
            ensure!(
                *audit.auditors.as_ref().unwrap() == group_account,
                <Error<T>>::NotAuditor
            );

            let bounded_content_type = enforce_limit!(content_type);
            let bounded_hash = enforce_limit!(hash);
            let bounded_name = enforce_limit!(name);
            let bounded_url = enforce_url_limit_option!(url);

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Evidence as u8),
                &group_account,
            );

            let evidence_id = next_id!(NextEvidenceId<T>, T);

            let evidence = Evidence {
                proposal_id,
                content_type: bounded_content_type,
                hash: bounded_hash,
                name: bounded_name,
                url: bounded_url,
            };

            <Evidences<T>>::insert(&audit_id, &evidence_id, evidence);

            <EvidenceByProposal<T>>::insert(&proposal_id, (audit_id, evidence_id));

            Self::deposit_event(Event::EvidenceAttached(
                group_account,
                proposal_id,
                audit_id,
                evidence_id,
            ));
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
            let (_, proposal_id, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(audit.auditors.is_some(), <Error<T>>::AuditorNotAssigned);
            ensure!(
                *audit.auditors.as_ref().unwrap() == group_account,
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
                group_account,
                proposal_id,
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
            let (_, proposal_id, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(audit.auditors.is_some(), <Error<T>>::AuditorNotAssigned);
            ensure!(
                *audit.auditors.as_ref().unwrap() == group_account,
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
                group_account,
                proposal_id,
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
            let (_, proposal_id, _, _, group_account) =
                <T as groups::Config>::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let maybe_audit = <Audits<T>>::get(audit_id);
            ensure!(maybe_audit.is_some(), <Error<T>>::AuditNotFound);
            let audit = maybe_audit.unwrap();
            ensure!(
                audit.status == AuditStatus::InProgress,
                <Error<T>>::AuditIsNotInProgress
            );
            ensure!(audit.auditors.is_some(), <Error<T>>::AuditorNotAssigned);
            ensure!(
                *audit.auditors.as_ref().unwrap() == group_account,
                <Error<T>>::NotAuditor
            );
            let maybe_evidence = <Evidences<T>>::get(audit_id, evidence_id);
            ensure!(maybe_evidence.is_some(), <Error<T>>::EvidenceNotFound);
            let evidence = maybe_evidence.unwrap();
            ensure!(
                link_count <= <T as Config>::MaxLinkRemove::get(),
                <Error<T>>::RemoveLinkLimitExceeded
            );

            for (i, (observation_id, _)) in
                <EvidenceLinksByEvidence<T>>::drain_prefix(evidence_id).enumerate()
            {
                if i as u32 >= link_count {
                    Self::deposit_event(Event::EvidenceDeleteFailed(
                        group_account,
                        proposal_id,
                        audit_id,
                        evidence_id,
                    ));
                    return Ok(().into());
                }
                <EvidenceLinksByObservation<T>>::remove(observation_id, evidence_id);
            }

            <Evidences<T>>::remove(&audit_id, &evidence_id);
            <EvidenceByProposal<T>>::remove(&evidence.proposal_id);

            Self::deposit_event(Event::EvidenceDeleted(
                group_account,
                proposal_id,
                audit_id,
                evidence_id,
            ));
            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --

        pub fn get_audits_by_creator(
            account: T::AccountId,
        ) -> Vec<(T::AuditId, Audit<T::AccountId, T::ProposalId>)> {
            let mut audits = Vec::new();
            <AuditsByCreator<T>>::iter_prefix(account).for_each(|(audit_id, _)| {
                let audit = <Audits<T>>::get(audit_id).unwrap();
                audits.push((audit_id, audit))
            });
            audits
        }

        pub fn get_audits_by_auditing_org(
            account: T::AccountId,
        ) -> Vec<(T::AuditId, Audit<T::AccountId, T::ProposalId>)> {
            let mut audits = Vec::new();
            <AuditsByAuditingOrg<T>>::iter_prefix(account).for_each(|(audit_id, _)| {
                let audit = <Audits<T>>::get(audit_id).unwrap();
                audits.push((audit_id, audit))
            });
            audits
        }

        pub fn get_audits_by_auditors(
            account: T::AccountId,
        ) -> Vec<(T::AuditId, Audit<T::AccountId, T::ProposalId>)> {
            let mut audits = Vec::new();
            <AuditsByAuditors<T>>::iter_prefix(account).for_each(|(audit_id, _)| {
                let audit = <Audits<T>>::get(audit_id).unwrap();
                audits.push((audit_id, audit))
            });
            audits
        }

        pub fn get_linked_audits(
            audit_id: T::AuditId,
        ) -> Vec<(T::AuditId, Audit<T::AccountId, T::ProposalId>)> {
            let mut audits = Vec::new();
            <LinkedAudits<T>>::iter_prefix(audit_id).for_each(|(child_audit_id, _)| {
                let audit_maybe = <Audits<T>>::get(child_audit_id);
                if let Some(audit) = audit_maybe {
                    audits.push((child_audit_id, audit));
                }
            });
            audits
        }

        pub fn get_audit(audit_id: T::AuditId) -> Option<Audit<T::AccountId, T::ProposalId>> {
            <Audits<T>>::get(audit_id)
        }

        pub fn get_audit_by_proposal(
            proposal_id: T::ProposalId,
        ) -> Option<(T::AuditId, Audit<T::AccountId, T::ProposalId>)> {
            <AuditByProposal<T>>::get(proposal_id)
                .map(|audit_id| (audit_id, <Audits<T>>::get(audit_id).unwrap()))
        }

        pub fn get_observation(
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
            observation_id: T::ObservationId,
        ) -> Option<(
            Observation<T::ProposalId>,
            Vec<(
                T::EvidenceId,
                Evidence<
                    T::ProposalId,
                    BoundedVec<u8, <T as Config>::NameLimit>,
                    BoundedVec<u8, <T as Config>::UrlLimit>,
                >,
            )>,
        )> {
            <Observations<T>>::get((audit_id, control_point_id), observation_id).map(
                |observation| {
                    let mut evidences = Vec::new();
                    <EvidenceLinksByObservation<T>>::iter_prefix(observation_id).for_each(
                        |(evidence_id, _)| {
                            if let Some(evidence) = <Evidences<T>>::get(audit_id, evidence_id) {
                                evidences.push((evidence_id, evidence));
                            }
                        },
                    );
                    (observation, evidences)
                },
            )
        }

        pub fn get_observation_by_proposal(
            proposal_id: T::ProposalId,
        ) -> Option<(
            T::ObservationId,
            Observation<T::ProposalId>,
            Vec<(
                T::EvidenceId,
                Evidence<
                    T::ProposalId,
                    BoundedVec<u8, <T as Config>::NameLimit>,
                    BoundedVec<u8, <T as Config>::UrlLimit>,
                >,
            )>,
        )> {
            <ObservationByProposal<T>>::get(proposal_id).map(
                |(audit_id, control_point_id, observation_id)| {
                    let mut evidences = Vec::new();
                    <EvidenceLinksByObservation<T>>::iter_prefix(observation_id).for_each(
                        |(evidence_id, _)| {
                            if let Some(evidence) = <Evidences<T>>::get(audit_id, evidence_id) {
                                evidences.push((evidence_id, evidence));
                            }
                        },
                    );
                    (
                        observation_id,
                        <Observations<T>>::get((audit_id, control_point_id), observation_id)
                            .unwrap(),
                        evidences,
                    )
                },
            )
        }

        pub fn get_observation_by_control_point(
            audit_id: T::AuditId,
            control_point_id: T::ControlPointId,
        ) -> Vec<(
            T::ObservationId,
            Observation<T::ProposalId>,
            Vec<(
                T::EvidenceId,
                Evidence<
                    T::ProposalId,
                    BoundedVec<u8, <T as Config>::NameLimit>,
                    BoundedVec<u8, <T as Config>::UrlLimit>,
                >,
            )>,
        )> {
            let mut observations = Vec::new();
            <Observations<T>>::iter_prefix((audit_id, control_point_id)).for_each(
                |(observation_id, observation)| {
                    let mut evidences = Vec::new();
                    <EvidenceLinksByObservation<T>>::iter_prefix(observation_id).for_each(
                        |(evidence_id, _)| {
                            if let Some(evidence) = <Evidences<T>>::get(audit_id, evidence_id) {
                                evidences.push((evidence_id, evidence));
                            }
                        },
                    );
                    observations.push((observation_id, observation, evidences))
                },
            );
            observations
        }

        pub fn get_evidence(
            audit_id: T::AuditId,
            evidence_id: T::EvidenceId,
        ) -> Option<
            Evidence<
                T::ProposalId,
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::UrlLimit>,
            >,
        > {
            <Evidences<T>>::get(audit_id, evidence_id)
        }

        pub fn get_evidence_by_audit(
            audit_id: T::AuditId,
        ) -> Vec<(
            T::EvidenceId,
            Evidence<
                T::ProposalId,
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::UrlLimit>,
            >,
        )> {
            let mut evidences = Vec::new();
            <Evidences<T>>::iter_prefix(audit_id)
                .for_each(|(evidence_id, evidence)| evidences.push((evidence_id, evidence)));
            evidences
        }

        pub fn get_evidence_by_proposal(
            proposal_id: T::ProposalId,
        ) -> Option<(
            T::EvidenceId,
            Evidence<
                T::ProposalId,
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::UrlLimit>,
            >,
        )> {
            <EvidenceByProposal<T>>::get(proposal_id).map(|(audit_id, evidence_id)| {
                (
                    evidence_id,
                    <Evidences<T>>::get(audit_id, evidence_id).unwrap(),
                )
            })
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
