//! # Group Module
//!
//! ## Overview
//!
//! TODO:
//!
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {

    pub use super::weights::WeightInfo;

    use frame_support::{
        codec::{Decode, Encode},
        dispatch::{
            DispatchError, DispatchResultWithPostInfo, Dispatchable, Parameter, PostDispatchInfo,
            Vec,
        },
        ensure,
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement::AllowDeath, Get},
        weights::{GetDispatchInfo, Weight},
    };
    use frame_system::{self as system, pallet_prelude::*};
    use group_info::GroupInfo;
    use primitives::group::{Group, Votes};
    use sp_io::hashing::blake2_256;
    use sp_runtime::{
        traits::{
            AtLeast32Bit, AtLeast32BitUnsigned, CheckedAdd, Hash, One, Saturating,
            UniqueSaturatedFrom, Zero,
        },
        DispatchResult,
    };
    use sp_std::{prelude::*, vec};

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type GroupId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
        type ProposalId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
        /// A number of members.
        ///
        /// This also serves as a number of voting members, and since for motions, each member may
        /// vote exactly once, therefore also the number of votes for any given motion.
        type MemberCount: Parameter + AtLeast32BitUnsigned + Default + Copy + PartialEq + PartialOrd;

        type Currency: Currency<Self::AccountId>;

        /// The outer origin type.
        type Origin: From<RawOrigin<Self::AccountId, Self::GroupId, Self::MemberCount>>;

        type GroupApprovalOrigin: EnsureOrigin<
            <Self as frame_system::Config>::Origin,
            Success = (
                Self::GroupId,
                Option<Self::MemberCount>,
                Option<Self::MemberCount>,
                Self::AccountId,
            ),
        >;

        type Proposal: Parameter
            + Dispatchable<Origin = <Self as Config>::Origin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>;

        /// Maximum number of proposals allowed to be active in parallel.
        //TODO: choose correct type
        type MaxProposals: Get<Self::MemberCount>;

        /// The maximum number of members supported by the pallet. Used for weight estimation.
        ///
        /// NOTE:
        /// + Benchmarks will need to be re-run and weights adjusted if this changes.
        /// + This pallet assumes that dependents keep to the limit without enforcing it.
        type MaxMembers: Get<Self::MemberCount>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    /// Origin for groups module proposals.

    #[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode)]
    pub enum RawOrigin<AccountId, GroupId, MemberCount> {
        /// It has been condoned by a given number of members of the group.
        /// (group_id,yes_votes,no_votes,group_account)
        ProposalApproved(GroupId, MemberCount, MemberCount, AccountId),
        /// It has been approved by veto.
        /// (group_id,veto_account,group_account)
        ProposalApprovedByVeto(GroupId, AccountId, AccountId),
    }

    /// Origin for the groups module.
    #[pallet::origin]
    pub type Origin<T> = RawOrigin<
        <T as frame_system::Config>::AccountId,
        <T as Config>::GroupId,
        <T as Config>::MemberCount,
    >;

    #[pallet::event]
    #[pallet::metadata(
        T::Moment = "Moment",
        T::GroupId = "GroupId",
        T::ProposalId = "ProposalId",
        T::Hash = "Hash",
        T::AccountId = "AccountId",
        T::MemberCount = "MemberCount"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new Group was created (creator,group_id,annonymous_account)
        GroupCreated(T::AccountId, T::GroupId, T::AccountId),
        /// A new SubGroup was created (parent_group_id,group_id,annonymous_account)
        SubGroupCreated(T::GroupId, T::GroupId, T::AccountId),
        /// A Group was updated (admin_group_id,group_id)
        GroupUpdated(T::GroupId, T::GroupId),
        /// A Group was removed (admin_group_id,group_id)
        GroupRemoved(T::GroupId, T::GroupId),
        /// A new SubGroup was created (proposer,group_id,proposal_id)
        Proposed(T::AccountId, T::GroupId, T::ProposalId),
        /// A proposal was voted on (voter,group_id,proposal_id,approved,yes_votes,no_votes)
        Voted(
            T::AccountId,
            T::GroupId,
            T::ProposalId,
            bool,
            T::MemberCount,
            T::MemberCount,
        ),
        /// A proposal was approved by veto (vetoer,group_id,proposal_id,approved,yes_votes,no_votes,success)
        ApprovedByVeto(
            T::AccountId,
            T::GroupId,
            T::ProposalId,
            T::MemberCount,
            T::MemberCount,
            bool,
        ),
        /// A proposal was disapproved by veto (vetoer,group_id,proposal_id,approved,yes_votes,no_votes)
        DisapprovedByVeto(
            T::AccountId,
            T::GroupId,
            T::ProposalId,
            T::MemberCount,
            T::MemberCount,
        ),
        /// A motion was approved by the required threshold and executed; (group_id,proposal_id,yes_votes,no_votes,success)
        Approved(
            T::GroupId,
            T::ProposalId,
            T::MemberCount,
            T::MemberCount,
            bool,
        ),
        /// A motion was disapproved by the required threshold; (group_id,proposal_id,yes_votes,no_votes)
        Disapproved(T::GroupId, T::ProposalId, T::MemberCount, T::MemberCount),
        /// A member of an admin group submitted an extrinsic as the group account  (member_account,group_id,error)
        DepositedAsGroup(T::AccountId, T::GroupId, Option<DispatchError>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account is not a member
        NotMember,
        /// A group must have members
        MembersRequired,
        /// Invalid threshold provided when creating a group
        InvalidThreshold,
        /// Failed to give minimum balance to group account
        AccountCreationFailed,
        /// Duplicate proposals not allowed
        DuplicateProposal,

        /// Group must exist
        GroupMissing,
        /// Proposal must exist
        ProposalMissing,
        /// Vote must exist
        VoteMissing,
        /// Mismatched index
        WrongIndex,
        /// Duplicate vote ignored
        DuplicateVote,
        /// Members are already initialized!
        AlreadyInitialized,
        /// There can only be a maximum of `MaxProposals` active proposals.
        TooManyProposals,
        /// The given weight bound for the proposal was too low.
        WrongProposalWeight,
        /// The given length bound for the proposal was too low.
        WrongProposalLength,
        /// Group is a SubGroup but should be a Group
        NotGroup,
        /// Group is not a SubGroup
        NotSubGroup,
        /// User is not admin for group
        NotGroupAdmin,
        /// User is not the group account (was not correctly called via proposal)
        NotGroupAccount,

        NoIdAvailable,
    }

    // #[pallet::type_value]
    // pub fn ExistentialDepositRequirement<T: Config>() -> T::Balance {
    //     1u32.into()
    // }

    #[pallet::type_value]
    pub fn GroupIdDefault<T: Config>() -> T::GroupId {
        1u32.into()
    }

    #[pallet::type_value]
    pub fn ProposalIdDefault<T: Config>() -> T::ProposalId {
        1u32.into()
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_group_id)]
    /// The next available group id
    pub(super) type NextGroupId<T: Config> =
        StorageValue<_, T::GroupId, ValueQuery, GroupIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    /// The next available proposal id
    pub(super) type NextProposalId<T: Config> =
        StorageValue<_, T::ProposalId, ValueQuery, ProposalIdDefault<T>>;

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

    /// Groups have some properties
    /// GroupId => Group
    #[pallet::storage]
    #[pallet::getter(fn groups)]
    pub(super) type Groups<T: Config> = StorageMap<
        _,
        Identity,
        T::GroupId,
        Group<T::GroupId, T::AccountId, T::MemberCount>,
        OptionQuery,
    >;

    /// Groups may have child groups
    /// GroupId => Vec<GroupId>
    #[pallet::storage]
    #[pallet::getter(fn group_children)]
    pub(super) type GroupChildren<T: Config> =
        StorageMap<_, Identity, T::GroupId, Vec<T::GroupId>, OptionQuery>;

    /// Groups may have proposals awaiting approval
    /// GroupId,ProposalId => Option<Proposal>
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub(super) type Proposals<T: Config> = StorageDoubleMap<
        _,
        Identity,
        T::GroupId,
        Identity,
        T::ProposalId,
        <T as Config>::Proposal,
        OptionQuery,
    >;

    /// Store vec of proposal hashes by group to ensure uniqueness ie a group may not have two identical proposals at any one time
    /// GroupId => Vec<T::Hash>
    #[pallet::storage]
    #[pallet::getter(fn proposal_hashes)]
    pub(super) type ProposalHashes<T: Config> =
        StorageMap<_, Identity, T::GroupId, Vec<T::Hash>, OptionQuery>;

    /// Votes on a given proposal, if it is ongoing.
    /// GroupId,ProposalId => Option<Votes>
    #[pallet::storage]
    #[pallet::getter(fn voting)]
    pub(super) type Voting<T: Config> = StorageDoubleMap<
        _,
        Identity,
        T::GroupId,
        Identity,
        T::ProposalId,
        Votes<T::AccountId, T::ProposalId, T::MemberCount>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new Group
        ///
        /// # <weight>
        /// TODO:
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_group(
            origin: OriginFor<T>,
            name: Vec<u8>,
            mut members: Vec<T::AccountId>,
            threshold: T::MemberCount,
            initial_balance: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            if !members.contains(&sender) {
                members.push(sender.clone());
            }
            ensure!(
                threshold > Zero::zero()
                    && threshold <= T::MemberCount::unique_saturated_from(members.len() as u128),
                Error::<T>::InvalidThreshold
            );

            let group_id = next_id!(NextGroupId<T>, T);

            let anonymous_account = Self::anonymous_account(&sender, group_id);

            let result = <T as Config>::Currency::transfer(
                &sender,
                &anonymous_account,
                <T as Config>::Currency::minimum_balance() + initial_balance,
                AllowDeath,
            );

            ensure!(result.is_ok(), Error::<T>::AccountCreationFailed);

            let group = Group {
                parent: None,
                name,
                members,
                threshold,
                funding_account: sender.clone(),
                anonymous_account: anonymous_account.clone(),
            };

            <Groups<T>>::insert(group_id, group);
            <GroupChildren<T>>::insert(group_id, Vec::<T::GroupId>::new());
            <ProposalHashes<T>>::insert(group_id, Vec::<T::Hash>::new());

            Self::deposit_event(Event::GroupCreated(sender, group_id, anonymous_account));

            Ok(().into())
        }

        /// Create a new SubGroup
        ///
        /// # <weight>
        /// TODO:
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_sub_group(
            origin: OriginFor<T>,
            name: Vec<u8>,
            members: Vec<T::AccountId>,
            threshold: T::MemberCount,
            initial_balance: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _yes_votes, _no_votes, caller_group_account) =
                T::GroupApprovalOrigin::ensure_origin(origin)?;

            ensure!(members.len() > 0, Error::<T>::MembersRequired);
            ensure!(
                threshold > Zero::zero()
                    && threshold <= T::MemberCount::unique_saturated_from(members.len() as u128),
                Error::<T>::InvalidThreshold
            );

            let mut admin_group = Self::groups(caller_group_id).ok_or(Error::<T>::GroupMissing)?;
            let mut admin_group_id = caller_group_id;
            while admin_group.parent.is_some() {
                admin_group_id = admin_group.parent.unwrap();
                admin_group = Self::groups(admin_group_id).ok_or(Error::<T>::GroupMissing)?;
            }

            let group_id = next_id!(NextGroupId<T>, T);

            let anonymous_account = Self::anonymous_account(&caller_group_account, group_id);

            //TODO: Since groups pay for transactions, we will need to give them more currency at some point
            let result = <T as Config>::Currency::transfer(
                &caller_group_account,
                &anonymous_account,
                <T as Config>::Currency::minimum_balance() + initial_balance,
                AllowDeath,
            );

            ensure!(result.is_ok(), Error::<T>::AccountCreationFailed);

            let group = Group {
                parent: Some(caller_group_id),
                name,
                members,
                threshold,
                funding_account: admin_group.funding_account,
                anonymous_account: anonymous_account.clone(),
            };

            <Groups<T>>::insert(group_id, group);
            <GroupChildren<T>>::try_mutate_exists(
                caller_group_id,
                |maybe_group_children| -> DispatchResult {
                    let group_children = maybe_group_children
                        .as_mut()
                        .ok_or(Error::<T>::GroupMissing)?;
                    group_children.push(group_id);
                    Ok(())
                },
            )?;
            <ProposalHashes<T>>::insert(group_id, Vec::<T::Hash>::new());

            Self::deposit_event(Event::SubGroupCreated(
                admin_group_id,
                group_id,
                anonymous_account,
            ));

            Ok(().into())
        }

        /// Update a Group. Can only be called via a proposal from any group in the parent chain.
        ///
        /// # <weight>
        /// TODO:
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_group(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            name: Option<Vec<u8>>,
            members: Option<Vec<T::AccountId>>,
            threshold: Option<T::MemberCount>,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _yes_votes, _no_votes, _caller_group_account) =
                T::GroupApprovalOrigin::ensure_origin(origin)?;

            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            let mut admin_group = group;
            let mut admin_group_id = group_id;
            while caller_group_id != group_id && admin_group.parent.is_some() {
                admin_group_id = admin_group.parent.unwrap();
                admin_group = Self::groups(admin_group_id).ok_or(Error::<T>::GroupMissing)?;
            }
            ensure!(caller_group_id == group_id, Error::<T>::NotGroupAdmin);

            <Groups<T>>::mutate(group_id, |group_option| {
                if let Some(group) = group_option {
                    if let Some(name) = name {
                        group.name = name;
                    }
                    if let Some(members) = members {
                        group.members = members;
                    }
                    if let Some(threshold) = threshold {
                        group.threshold = threshold;
                    }
                }
            });

            Self::deposit_event(Event::GroupUpdated(admin_group_id, group_id));

            Ok(().into())
        }

        /// Remove a Group. All child groups will also be removed. Can only be called via a proposal from any group in the parent chain.
        ///
        /// # <weight>
        ///TODO:
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_group(
            origin: OriginFor<T>,
            group_id: T::GroupId,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _yes_votes, _no_votes, _caller_group_account) =
                T::GroupApprovalOrigin::ensure_origin(origin)?;

            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            let mut admin_group = group.clone();
            let mut admin_group_id = group_id;
            while caller_group_id != group_id && admin_group.parent.is_some() {
                admin_group_id = admin_group.parent.unwrap();
                admin_group = Self::groups(admin_group_id).ok_or(Error::<T>::GroupMissing)?;
            }
            ensure!(caller_group_id == group_id, Error::<T>::NotGroupAdmin);

            Self::remove_children(group_id);

            if let Some(parent_group_id) = group.parent {
                <GroupChildren<T>>::try_mutate_exists(
                    parent_group_id,
                    |maybe_group_children| -> DispatchResult {
                        let group_children = maybe_group_children
                            .as_mut()
                            .ok_or(Error::<T>::GroupMissing)?;
                        group_children.retain(|gid| *gid != group_id);
                        Ok(())
                    },
                )?;
            }

            <Groups<T>>::remove(&group_id);
            <GroupChildren<T>>::remove(&group_id);
            <ProposalHashes<T>>::remove(&group_id);

            Self::deposit_event(Event::GroupRemoved(admin_group_id, group_id));

            Ok(().into())
        }

        /// Add a new proposal to either be voted on or executed directly.
        ///
        /// Requires the sender to be member.
        ///

        #[pallet::weight(proposal.get_dispatch_info().weight)]
        pub fn propose(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            proposal: Box<<T as Config>::Proposal>,
            //TODO: how do we add compact macro?
            // #[compact]
            threshold: T::MemberCount,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.members.contains(&sender), Error::<T>::NotMember);
            let proposal_len = proposal.using_encoded(|x| x.len());
            let proposal_hash = T::Hashing::hash_of(&proposal);
            ensure!(
                <ProposalHashes<T>>::contains_key(group_id),
                Error::<T>::GroupMissing
            );
            let proposal_hashes = <ProposalHashes<T>>::get(group_id).unwrap();
            ensure!(
                !proposal_hashes.contains(&proposal_hash),
                Error::<T>::DuplicateProposal
            );
            let proposals_length = proposal_hashes.len();

            let proposal_id = next_id!(NextProposalId<T>, T);

            if threshold < 2u32.into() {
                let result = proposal.dispatch(
                    RawOrigin::ProposalApproved(
                        group_id,
                        1u32.into(),
                        0u32.into(),
                        group.anonymous_account.clone(),
                    )
                    .into(),
                );
                Self::deposit_event(Event::Approved(
                    group_id,
                    proposal_id,
                    1u32.into(),
                    0u32.into(),
                    result.is_ok(),
                ));

                let weight = Self::get_result_weight(result).map(|w| {
                    <T as Config>::WeightInfo::propose_execute(
                        proposal_len as u32,
                        group.members.len() as u32,
                    )
                    .saturating_add(w) // P1
                });

                //TODO: what to do on insufficint funds

                Ok(weight.into())
            } else {
                <ProposalHashes<T>>::try_mutate_exists(
                    group_id,
                    |maybe_proposals| -> DispatchResult {
                        let proposals = maybe_proposals
                            .as_mut()
                            .ok_or(Error::<T>::ProposalMissing)?;
                        proposals.push(proposal_hash);
                        Ok(())
                    },
                )?;
                let active_proposals = proposals_length + 1;

                <Proposals<T>>::insert(group_id, proposal_id, proposal);

                let votes = Votes {
                    proposal_id,
                    threshold,
                    ayes: vec![sender.clone()],
                    nays: vec![],
                };
                <Voting<T>>::insert(group_id, proposal_id, votes);

                Self::deposit_event(Event::Proposed(sender, group_id, proposal_id));

                Ok(Some(<T as Config>::WeightInfo::propose_proposed(
                    proposal_len as u32,
                    group.members.len() as u32,
                    active_proposals as u32,
                ))
                .into())
            }
        }

        /// Vote on a Proposal
        ///
        /// # <weight>
        ///TODO:
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn vote(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            proposal_id: T::ProposalId,
            approve: bool,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.members.contains(&sender), Error::<T>::NotMember);
            let proposal =
                Self::proposals(group_id, proposal_id).ok_or(Error::<T>::ProposalMissing)?;
            let proposal_hash = T::Hashing::hash_of(&proposal);
            let mut voting = Self::voting(group_id, proposal_id).ok_or(Error::<T>::VoteMissing)?;
            ensure!(voting.proposal_id == proposal_id, Error::<T>::WrongIndex);
            let position_yes = voting.ayes.iter().position(|a| a == &sender);
            let position_no = voting.nays.iter().position(|a| a == &sender);

            // Detects first vote of the member in the motion
            let is_account_voting_first_time = position_yes.is_none() && position_no.is_none();

            if approve {
                if position_yes.is_none() {
                    voting.ayes.push(sender.clone());
                } else {
                    Err(Error::<T>::DuplicateVote)?
                }
                if let Some(pos) = position_no {
                    voting.nays.swap_remove(pos);
                }
            } else {
                if position_no.is_none() {
                    voting.nays.push(sender.clone());
                } else {
                    Err(Error::<T>::DuplicateVote)?
                }
                if let Some(pos) = position_yes {
                    voting.ayes.swap_remove(pos);
                }
            }

            let yes_votes = T::MemberCount::unique_saturated_from(voting.ayes.len() as u128);
            let no_votes = T::MemberCount::unique_saturated_from(voting.nays.len() as u128);
            Self::deposit_event(Event::Voted(
                sender,
                group_id,
                proposal_id,
                approve,
                yes_votes,
                no_votes,
            ));

            Voting::<T>::insert(group_id, proposal_id, voting.clone());

            let seats = T::MemberCount::unique_saturated_from(group.members.len() as u128);
            let approved = yes_votes >= voting.threshold;
            let disapproved = seats.saturating_sub(no_votes) < voting.threshold;
            // Allow (dis-)approving the proposal as soon as there are enough votes.
            if approved {
                let result = proposal.dispatch(
                    RawOrigin::ProposalApproved(
                        group_id,
                        yes_votes,
                        no_votes,
                        group.anonymous_account.clone(),
                    )
                    .into(),
                );
                Self::deposit_event(Event::Approved(
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                    result.is_ok(),
                ));
                Self::remove_proposal(group_id, proposal_id, proposal_hash)?;
            } else if disapproved {
                Self::deposit_event(Event::Disapproved(
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                ));
                Self::remove_proposal(group_id, proposal_id, proposal_hash)?;
            };

            if is_account_voting_first_time {
                Ok((
                    Some(<T as Config>::WeightInfo::vote(group.members.len() as u32)),
                    Pays::No,
                )
                    .into())
            } else {
                Ok((
                    Some(<T as Config>::WeightInfo::vote(group.members.len() as u32)),
                    Pays::Yes,
                )
                    .into())
            }
        }

        /// Veto a Proposal
        ///
        /// # <weight>
        ///TODO:
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn veto(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            proposal_id: T::ProposalId,
            approve: bool,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.parent.is_some(), Error::<T>::NotSubGroup);
            //is sender in parent members else recurse parent
            let mut parent_group =
                Self::groups(group.parent.unwrap()).ok_or(Error::<T>::GroupMissing)?;
            while !parent_group.members.contains(&sender) {
                ensure!(parent_group.parent.is_some(), Error::<T>::NotGroupAdmin);
                parent_group =
                    Self::groups(parent_group.parent.unwrap()).ok_or(Error::<T>::GroupMissing)?;
            }

            let proposal =
                Self::proposals(group_id, proposal_id).ok_or(Error::<T>::ProposalMissing)?;
            let proposal_hash = T::Hashing::hash_of(&proposal);
            let voting = Self::voting(group_id, proposal_id).ok_or(Error::<T>::VoteMissing)?;

            let yes_votes = T::MemberCount::unique_saturated_from(voting.ayes.len() as u128);
            let no_votes = T::MemberCount::unique_saturated_from(voting.nays.len() as u128);

            if approve {
                let result = proposal.dispatch(
                    RawOrigin::ProposalApprovedByVeto(
                        group_id,
                        sender.clone(),
                        group.anonymous_account.clone(),
                    )
                    .into(),
                );
                Self::deposit_event(Event::ApprovedByVeto(
                    sender,
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                    result.is_ok(),
                ));
                Self::remove_proposal(group_id, proposal_id, proposal_hash)?;
            } else {
                Self::deposit_event(Event::DisapprovedByVeto(
                    sender,
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                ));
                Self::remove_proposal(group_id, proposal_id, proposal_hash)?;
            };

            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --
        pub fn member_of(account: T::AccountId) -> Vec<T::GroupId> {
            let mut groups_ids = Vec::new();

            <Groups<T>>::iter().for_each(|(group_id, group)| {
                if group.members.contains(&account) {
                    groups_ids.push(group_id)
                }
            });

            groups_ids
        }

        pub fn get_group(
            group_id: T::GroupId,
        ) -> Option<Group<T::GroupId, T::AccountId, T::MemberCount>> {
            <Groups<T>>::get(group_id)
        }
        pub fn get_sub_groups(
            group_id: T::GroupId,
        ) -> Option<Vec<(T::GroupId, Group<T::GroupId, T::AccountId, T::MemberCount>)>> {
            let maybe_group_ids = <GroupChildren<T>>::get(group_id);
            maybe_group_ids.map(|group_ids| {
                group_ids
                    .into_iter()
                    .filter_map(|group_id| {
                        <Groups<T>>::get(group_id).map(|group| (group_id, group))
                    })
                    .collect()
            })
        }

        // -- private functions --

        fn is_member(groupd_id: T::GroupId, account_id: &T::AccountId) -> bool {
            let group = <Groups<T>>::get(groupd_id);
            if group.is_none() {
                return false;
            }
            group.unwrap().members.contains(account_id)
        }

        fn is_group_account(groupd_id: T::GroupId, account_id: &T::AccountId) -> bool {
            let group = <Groups<T>>::get(groupd_id);
            if group.is_none() {
                return false;
            }
            group.unwrap().anonymous_account == *account_id
        }

        /// Return the weight of a dispatch call result as an `Option`.
        ///
        /// Will return the weight regardless of what the state of the result is.
        fn get_result_weight(result: DispatchResultWithPostInfo) -> Option<Weight> {
            match result {
                Ok(post_info) => post_info.actual_weight,
                Err(err) => err.post_info.actual_weight,
            }
        }

        /// Calculate the address of an anonymous account.
        ///
        /// - `who`: The spawner account.
        /// - `index`: A disambiguation index, in case this is called multiple times in the same
        /// transaction (e.g. with `utility::batch`). Unless you're using `batch` you probably just
        /// want to use `0`.

        fn anonymous_account(who: &T::AccountId, index: T::GroupId) -> T::AccountId {
            let (height, ext_index) = (
                system::Module::<T>::block_number(),
                system::Module::<T>::extrinsic_index().unwrap_or_default(),
            );
            let entropy =
                (b"modlpy/proxy____", who, height, ext_index, index).using_encoded(blake2_256);
            T::AccountId::decode(&mut &entropy[..]).unwrap_or_default()
        }

        fn remove_children(group_id: T::GroupId) {
            <GroupChildren<T>>::get(group_id).map(|group_ids| {
                group_ids.into_iter().for_each(|child_group_id| {
                    //TODO: should we emit event for every child group?
                    Self::remove_children(child_group_id);
                    <Groups<T>>::remove(&child_group_id);
                    <GroupChildren<T>>::remove(&child_group_id);
                })
            });
        }
        fn remove_proposal(
            group_id: T::GroupId,
            proposal_id: T::ProposalId,
            proposal_hash: T::Hash,
        ) -> DispatchResult {
            <Proposals<T>>::remove(group_id, proposal_id);
            <ProposalHashes<T>>::try_mutate_exists(
                group_id,
                |maybe_proposals| -> DispatchResult {
                    let proposals = maybe_proposals
                        .as_mut()
                        .ok_or(Error::<T>::ProposalMissing)?;
                    proposals.retain(|h| h != &proposal_hash);
                    Ok(())
                },
            )?;
            Ok(())
        }
    }

    impl<T: Config> GroupInfo for Module<T> {
        type AccountId = T::AccountId;
        type GroupId = T::GroupId;

        fn is_member(groupd_id: Self::GroupId, account_id: &Self::AccountId) -> bool {
            Self::is_member(groupd_id, account_id)
        }
        fn is_group_account(groupd_id: Self::GroupId, account_id: &Self::AccountId) -> bool {
            Self::is_group_account(groupd_id, account_id)
        }
    }

    pub struct EnsureApproved<AccountId, GroupId, MemberCount>(
        sp_std::marker::PhantomData<(AccountId, GroupId, MemberCount)>,
    );
    impl<
            O: Into<Result<RawOrigin<AccountId, GroupId, MemberCount>, O>>
                + From<RawOrigin<AccountId, GroupId, MemberCount>>,
            AccountId,
            GroupId,
            MemberCount,
        > EnsureOrigin<O> for EnsureApproved<AccountId, GroupId, MemberCount>
    {
        type Success = (GroupId, Option<MemberCount>, Option<MemberCount>, AccountId);
        fn try_origin(o: O) -> Result<Self::Success, O> {
            o.into().and_then(|o| match o {
                RawOrigin::ProposalApproved(group_id, yes_votes, no_votes, group_account) => {
                    Ok((group_id, Some(yes_votes), Some(no_votes), group_account))
                }
                RawOrigin::ProposalApprovedByVeto(group_id, _, group_account) => {
                    Ok((group_id, None, None, group_account))
                }
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> O {
            //TODO: fix
            O::from(RawOrigin::ProposalApproved(None, 0, 0, None))
        }
    }

    pub struct EnsureThreshold<T>(sp_std::marker::PhantomData<T>);
    impl<
            O: Into<Result<RawOrigin<T::AccountId, T::GroupId, T::MemberCount>, O>>
                + From<RawOrigin<T::AccountId, T::GroupId, T::MemberCount>>,
            T: Config,
        > EnsureOrigin<O> for EnsureThreshold<T>
    {
        type Success = (
            T::GroupId,
            Option<T::MemberCount>,
            Option<T::MemberCount>,
            T::AccountId,
        );
        fn try_origin(o: O) -> Result<Self::Success, O> {
            o.into().and_then(|o| match o.clone() {
                RawOrigin::ProposalApproved(group_id, yes_votes, no_votes, group_account) => {
                    let group_maybe = <Groups<T>>::get(group_id);
                    match group_maybe {
                        Some(group) => {
                            if yes_votes >= group.threshold {
                                Ok((group_id, Some(yes_votes), Some(no_votes), group_account))
                            } else {
                                Err(O::from(o))
                            }
                        }
                        None => Err(O::from(o)),
                    }
                }
                RawOrigin::ProposalApprovedByVeto(group_id, _, group_account) => {
                    Ok((group_id, None, None, group_account))
                }
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> O {
            //TODO: fix
            O::from(RawOrigin::ProposalApproved(None, None, None, None))
        }
    }
}
