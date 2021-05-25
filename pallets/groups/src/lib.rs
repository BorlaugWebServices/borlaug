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
        dispatch::{DispatchResultWithPostInfo, Dispatchable, Parameter, PostDispatchInfo, Vec},
        ensure,
        pallet_prelude::*,
        traits::Get,
        weights::{GetDispatchInfo, Weight},
    };
    use frame_system::{self as system, pallet_prelude::*};
    use primitives::group::{Group, Votes};
    use sp_io::hashing::blake2_256;
    use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, Hash, One};
    use sp_std::{prelude::*, vec};

    /// A number of members.
    ///
    /// This also serves as a number of voting members, and since for motions, each member may
    /// vote exactly once, therefore also the number of votes for any given motion.
    pub type MemberCount = u32;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type GroupId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
        type ProposalId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;

        /// The outer origin type.
        // type Origin: From<RawOrigin<Self::AccountId>>;
        /// The outer call dispatch type.
        type Proposal: Parameter
            + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
            + From<frame_system::Call<Self>>
            + GetDispatchInfo;

        /// Maximum number of proposals allowed to be active in parallel.
        //TODO: choose correct type
        type MaxProposals: Get<MemberCount>;

        /// The maximum number of members supported by the pallet. Used for weight estimation.
        ///
        /// NOTE:
        /// + Benchmarks will need to be re-run and weights adjusted if this changes.
        /// + This pallet assumes that dependents keep to the limit without enforcing it.
        type MaxMembers: Get<MemberCount>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::metadata(
        T::Moment = "Moment",
        T::GroupId = "GroupId",
        T::ProposalId = "ProposalId",
        T::Hash = "Hash",
        T::AccountId = "AccountId"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new Group was created (creator,group_id)
        GroupCreated(T::AccountId, T::GroupId),
        /// A new SubGroup was created (creator,group_id,parent_group_id)
        SubGroupCreated(T::AccountId, T::GroupId, T::GroupId),
        /// A Group was updated (updater,group_id)
        GroupUpdated(T::AccountId, T::GroupId),
        /// A Group was removed (remover,group_id)
        GroupRemoved(T::AccountId, T::GroupId),
        /// A new SubGroup was created (proposer,group_id,proposal_id)
        Proposed(T::AccountId, T::GroupId, T::ProposalId),
        /// A proposal was voted on (voter,group_id,proposal_id,approved,yes_votes,no_votes)
        Voted(
            T::AccountId,
            T::GroupId,
            T::ProposalId,
            bool,
            MemberCount,
            MemberCount,
        ),
        /// A proposal was approved by veto (vetoer,group_id,proposal_id,approved,yes_votes,no_votes,success)
        ApprovedByVeto(
            T::AccountId,
            T::GroupId,
            T::ProposalId,
            MemberCount,
            MemberCount,
            bool,
        ),
        /// A proposal was disapproved by veto (vetoer,group_id,proposal_id,approved,yes_votes,no_votes)
        DisapprovedByVeto(
            T::AccountId,
            T::GroupId,
            T::ProposalId,
            MemberCount,
            MemberCount,
        ),
        /// A motion was approved by the required threshold and executed; (group_id,proposal_id,yes_votes,no_votes,success)
        Approved(T::GroupId, T::ProposalId, MemberCount, MemberCount, bool),
        /// A motion was disapproved by the required threshold; (group_id,proposal_id,yes_votes,no_votes)
        Disapproved(T::GroupId, T::ProposalId, MemberCount, MemberCount),
        /// A member of an admin group submitted an extrinsic as the group account  (member_account,group_id,success)
        DepositedAsGroup(T::AccountId, T::GroupId, bool),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account is not a member
        NotMember,
        /// Bad data provided in group
        BadGroup,
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
        /// Group is not a SubGroup
        NotSubGroup,
        /// User is not admin for group
        NotGroupAdmin,

        NoIdAvailable,
    }

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

    /// Groups have some properties
    /// GroupId => Group
    #[pallet::storage]
    #[pallet::getter(fn groups)]
    pub(super) type Groups<T: Config> = StorageMap<
        _,
        Identity,
        T::GroupId,
        Option<Group<T::GroupId, T::AccountId, MemberCount>>,
        ValueQuery,
    >;

    /// Groups may have child groups
    /// GroupId => Vec<GroupId>
    #[pallet::storage]
    #[pallet::getter(fn group_children)]
    pub(super) type GroupChildren<T: Config> =
        StorageMap<_, Identity, T::GroupId, Vec<T::GroupId>, ValueQuery>;

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
        Option<<T as Config>::Proposal>,
        ValueQuery,
    >;

    /// Store vec of proposal hashes by group to ensure uniqueness ie a group may not have two identical proposals at any one time
    /// GroupId => Vec<T::Hash>
    #[pallet::storage]
    #[pallet::getter(fn proposal_hashes)]
    pub(super) type ProposalHashes<T: Config> =
        StorageMap<_, Identity, T::GroupId, Vec<T::Hash>, ValueQuery>;

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
        Option<Votes<T::AccountId, T::ProposalId>>,
        ValueQuery,
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
            threshold: MemberCount,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            if !members.contains(&sender) {
                members.push(sender.clone());
            }
            ensure!(
                threshold > 0 && threshold as usize <= members.len(),
                Error::<T>::BadGroup
            );

            let group_id = Self::get_next_group_id()?;

            let anonymous_account = Self::anonymous_account(&sender, group_id);

            let group = Group {
                parent: None,
                name,
                members,
                threshold,
                funding_account: sender.clone(),
                anonymous_account,
            };

            <Groups<T>>::insert(group_id, Some(group));
            <GroupChildren<T>>::insert(group_id, Vec::<T::GroupId>::new());

            Self::deposit_event(Event::GroupCreated(sender, group_id));

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
            parent_group_id: T::GroupId,
            name: Vec<u8>,
            members: Vec<T::AccountId>,
            threshold: MemberCount,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(members.len() > 0, Error::<T>::BadGroup);
            ensure!(
                threshold > 0 && threshold as usize <= members.len(),
                Error::<T>::BadGroup
            );

            let mut admin_group = Self::groups(parent_group_id).ok_or(Error::<T>::GroupMissing)?;
            let mut admin_group_id = parent_group_id;
            while admin_group.parent.is_some() {
                admin_group_id = admin_group.parent.unwrap();
                admin_group = Self::groups(admin_group_id).ok_or(Error::<T>::GroupMissing)?;
            }
            ensure!(
                admin_group.members.contains(&sender),
                Error::<T>::NotGroupAdmin
            );

            let group_id = Self::get_next_group_id()?;

            let anonymous_account = Self::anonymous_account(&sender, group_id);

            let group = Group {
                parent: Some(parent_group_id),
                name,
                members,
                threshold,
                funding_account: admin_group.funding_account,
                anonymous_account,
            };

            <Groups<T>>::insert(group_id, Some(group));
            <GroupChildren<T>>::mutate(parent_group_id, |group_children| {
                group_children.push(group_id)
            });

            Self::deposit_event(Event::SubGroupCreated(sender, group_id, admin_group_id));

            Ok(().into())
        }

        /// Update an Group. Parent cannot be changed
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
            threshold: Option<MemberCount>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            let mut admin_group = group;
            while admin_group.parent.is_some() {
                admin_group =
                    Self::groups(admin_group.parent.unwrap()).ok_or(Error::<T>::GroupMissing)?;
            }
            ensure!(
                admin_group.members.contains(&sender),
                Error::<T>::NotGroupAdmin
            );

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

            Self::deposit_event(Event::GroupUpdated(sender, group_id));

            Ok(().into())
        }

        /// Remove a Group. All child groups will also be removed.
        ///
        /// # <weight>
        ///TODO:
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_group(
            origin: OriginFor<T>,
            group_id: T::GroupId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            let mut admin_group = group.clone();
            while admin_group.parent.is_some() {
                admin_group =
                    Self::groups(admin_group.parent.unwrap()).ok_or(Error::<T>::GroupMissing)?;
            }
            ensure!(
                admin_group.members.contains(&sender),
                Error::<T>::NotGroupAdmin
            );

            Self::remove_children(group_id);

            if let Some(parent_group_id) = group.parent {
                <GroupChildren<T>>::mutate(parent_group_id, |group_children| {
                    group_children.retain(|gid| *gid != group_id)
                });
            }

            <Groups<T>>::remove(&group_id);
            <GroupChildren<T>>::remove(&group_id);

            Self::deposit_event(Event::GroupRemoved(sender, group_id));

            Ok(().into())
        }

        /// Executes an extrinsic using the group account
        ///
        /// Requires the sender to be member.
        ///

        #[pallet::weight(proposal.get_dispatch_info().weight)]
        pub fn as_group(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            proposal: Box<<T as Config>::Proposal>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.members.contains(&sender), Error::<T>::NotMember);
            let proposal_len = proposal.using_encoded(|x| x.len());

            let result = proposal
                .dispatch(frame_system::RawOrigin::Signed(group.anonymous_account.clone()).into());
            Self::deposit_event(Event::DepositedAsGroup(sender, group_id, result.is_ok()));

            Ok(Some(T::WeightInfo::as_group(proposal_len as u32)).into())
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
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.members.contains(&sender), Error::<T>::NotMember);
            let proposal_len = proposal.using_encoded(|x| x.len());
            let proposal_hash = T::Hashing::hash_of(&proposal);
            ensure!(
                !Self::proposal_hashes(group_id).contains(&proposal_hash),
                Error::<T>::DuplicateProposal
            );
            let proposals_length = Self::proposal_hashes(group_id).len();

            //TODO: fix
            // ensure!(
            //     proposals_length <= T::MaxProposals::get() as usize,
            //     Error::<T>::TooManyProposals
            // );

            let proposal_id = Self::get_next_proposal_id()?;

            if group.threshold < 2 {
                let result = proposal.dispatch(
                    frame_system::RawOrigin::Signed(group.anonymous_account.clone()).into(),
                );
                Self::deposit_event(Event::Approved(group_id, proposal_id, 1, 0, result.is_ok()));

                Ok(Self::get_result_weight(result)
                    .map(|w| {
                        T::WeightInfo::propose_execute(
                            proposal_len as u32,
                            group.members.len() as u32,
                        )
                        .saturating_add(w) // P1
                    })
                    .into())
            } else {
                <ProposalHashes<T>>::try_mutate(
                    group_id,
                    |proposals| -> Result<(), DispatchError> {
                        proposals.push(proposal_hash);
                        Ok(())
                    },
                )?;
                let active_proposals = proposals_length + 1;

                <Proposals<T>>::insert(group_id, proposal_id, Some(proposal));

                let votes = Votes {
                    proposal_id,
                    ayes: vec![sender.clone()],
                    nays: vec![],
                };
                <Voting<T>>::insert(group_id, proposal_id, Some(votes));

                Self::deposit_event(Event::Proposed(sender, group_id, proposal_id));

                Ok(Some(T::WeightInfo::propose_proposed(
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

            let yes_votes = voting.ayes.len() as MemberCount;
            let no_votes = voting.nays.len() as MemberCount;
            Self::deposit_event(Event::Voted(
                sender,
                group_id,
                proposal_id,
                approve,
                yes_votes,
                no_votes,
            ));

            Voting::<T>::insert(group_id, proposal_id, Some(voting));

            let seats = group.members.len() as MemberCount;
            let approved = yes_votes >= group.threshold;
            let disapproved = seats.saturating_sub(no_votes) < group.threshold;
            // Allow (dis-)approving the proposal as soon as there are enough votes.
            if approved {
                let result = proposal
                    .dispatch(frame_system::RawOrigin::Signed(group.anonymous_account).into());
                Self::deposit_event(Event::Approved(
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                    result.is_ok(),
                ));

                <Proposals<T>>::remove(group_id, proposal_id);
                ProposalHashes::<T>::mutate(group_id, |proposals| {
                    proposals.retain(|h| h != &proposal_hash)
                });
            } else if disapproved {
                Self::deposit_event(Event::Disapproved(
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                ));
                <Proposals<T>>::remove(group_id, proposal_id);
                ProposalHashes::<T>::mutate(group_id, |proposals| {
                    proposals.retain(|h| h != &proposal_hash)
                });
            };

            if is_account_voting_first_time {
                Ok((
                    Some(T::WeightInfo::vote(group.members.len() as u32)),
                    Pays::No,
                )
                    .into())
            } else {
                Ok((
                    Some(T::WeightInfo::vote(group.members.len() as u32)),
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
            let yes_votes = voting.ayes.len() as MemberCount;
            let no_votes = voting.nays.len() as MemberCount;

            if approve {
                let result = proposal
                    .dispatch(frame_system::RawOrigin::Signed(group.anonymous_account).into());
                Self::deposit_event(Event::ApprovedByVeto(
                    sender,
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                    result.is_ok(),
                ));
                <Proposals<T>>::remove(group_id, proposal_id);
                ProposalHashes::<T>::mutate(group_id, |proposals| {
                    proposals.retain(|h| h != &proposal_hash)
                });
            } else {
                Self::deposit_event(Event::DisapprovedByVeto(
                    sender,
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                ));
                <Proposals<T>>::remove(group_id, proposal_id);
                ProposalHashes::<T>::mutate(group_id, |proposals| {
                    proposals.retain(|h| h != &proposal_hash)
                });
            };

            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --
        pub fn member_of(account: T::AccountId) -> Vec<T::GroupId> {
            let mut groups_ids = Vec::new();

            <Groups<T>>::iter().for_each(|(group_id, group)| {
                if group.unwrap().members.contains(&account) {
                    groups_ids.push(group_id)
                }
            });

            groups_ids
        }

        // -- private functions --

        fn get_next_group_id() -> Result<T::GroupId, Error<T>> {
            let group_id = <NextGroupId<T>>::get();
            <NextGroupId<T>>::put(
                group_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoIdAvailable)?,
            );
            Ok(group_id)
        }

        fn get_next_proposal_id() -> Result<T::ProposalId, Error<T>> {
            let group_id = <NextProposalId<T>>::get();
            <NextProposalId<T>>::put(
                group_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoIdAvailable)?,
            );
            Ok(group_id)
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
            <GroupChildren<T>>::get(group_id)
                .into_iter()
                .for_each(|child_group_id| {
                    //TODO: should we emit event for every child group?
                    Self::remove_children(child_group_id);
                    <Groups<T>>::remove(&child_group_id);
                    <GroupChildren<T>>::remove(&child_group_id);
                });
        }
    }
}
