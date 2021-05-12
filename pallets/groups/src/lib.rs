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
            DispatchResult, DispatchResultWithPostInfo, Dispatchable, Parameter, PostDispatchInfo,
            Vec,
        },
        ensure,
        pallet_prelude::*,
        traits::{EnsureOrigin, Get},
        weights::{GetDispatchInfo, Weight},
    };
    use frame_system::pallet_prelude::*;
    use primitives::group::{Group, Votes};
    use sp_core::u32_trait::Value as U32;
    use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, Hash, One};
    use sp_std::{prelude::*, result, vec};

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
        type Origin: From<RawOrigin<Self::AccountId>>;
        /// The outer call dispatch type.
        type Proposal: Parameter
            + Dispatchable<Origin = <Self as Config>::Origin, PostInfo = PostDispatchInfo>
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

    /// Origin for the groups module.
    #[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode)]
    pub enum RawOrigin<AccountId> {
        /// It has been condoned by a given number of members of the groups from a given total.
        Members(MemberCount, MemberCount),
        /// It has been condoned by a single member of the groups.
        Member(AccountId),
    }

    /// Origin for the groups module.
    pub type Origin<T> = RawOrigin<<T as frame_system::Config>::AccountId>;

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
        /// A new Group was created (AccountId,GroupId)
        GroupCreated(T::AccountId, T::GroupId),

        Proposed(T::AccountId, T::GroupId, T::ProposalId),

        Voted(
            T::AccountId,
            T::GroupId,
            T::ProposalId,
            bool,
            MemberCount,
            MemberCount,
        ),

        Vetoed(T::AccountId, T::GroupId, T::ProposalId, bool),

        /// A motion was approved by the required threshold.
        /// \[proposal_hash\]
        Approved(T::GroupId, T::ProposalId),
        /// A motion was disapproved by the required threshold.
        /// \[proposal_hash\]
        DisApproved(T::GroupId, T::ProposalId),
        /// A motion was executed; result will be `Ok` if it returned without error.
        /// \[proposal_hash, result\]
        Executed(T::GroupId, T::ProposalId, DispatchResult),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account is not a member
        NotMember,
        /// Bad data provided in group
        BadGroup,
        /// Duplicate proposals not allowed
        DuplicateProposal,
        /// Proposal must exist
        ProposalMissing,
        /// Mismatched index
        WrongIndex,
        /// Duplicate vote ignored
        DuplicateVote,
        /// Members are already initialized!
        AlreadyInitialized,
        /// The close call was made too early, before the end of the voting.
        TooEarly,
        /// There can only be a maximum of `MaxProposals` active proposals.
        TooManyProposals,
        /// The given weight bound for the proposal was too low.
        WrongProposalWeight,
        /// The given length bound for the proposal was too low.
        WrongProposalLength,

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
        Group<T::GroupId, T::AccountId, MemberCount>,
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
            members: Vec<T::AccountId>,
            threshold: MemberCount,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(members.len() > 0, Error::<T>::BadGroup);
            ensure!(
                threshold > 0 && threshold as usize <= members.len(),
                Error::<T>::BadGroup
            );

            let group_id = Self::get_next_group_id()?;

            let group = Group {
                parent: None,
                name,
                members,
                threshold,
                funding_account: sender.clone(),
            };

            <Groups<T>>::insert(group_id, group);

            Self::deposit_event(Event::GroupCreated(sender, group_id));

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
            group: Group<T::GroupId, T::AccountId, MemberCount>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            //TODO:

            Ok(().into())
        }

        /// Remove an Group. All child groups will also be removed.
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

            //TODO:

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
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let group = Self::groups(group_id);
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
                let seats = group.members.len() as MemberCount;
                let result = proposal.dispatch(RawOrigin::Members(1, seats).into());
                Self::deposit_event(Event::Executed(
                    group_id,
                    proposal_id,
                    result.map(|_| ()).map_err(|e| e.error),
                ));

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

            let group = Self::groups(group_id);

            ensure!(group.members.contains(&sender), Error::<T>::NotMember);

            let proposal = Self::proposals(group_id, proposal_id);

            ensure!(proposal.is_some(), Error::<T>::ProposalMissing);

            let proposal = proposal.unwrap();

            let mut voting =
                Self::voting(group_id, proposal_id).ok_or(Error::<T>::ProposalMissing)?;

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
            index: T::ProposalId,
            vote: bool,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            //TODO:

            Self::deposit_event(Event::Vetoed(sender, group_id, index, vote));
            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
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
    }

    /// Ensure that the origin `o` represents at least `n` members. Returns `Ok` or an `Err`
    /// otherwise.
    pub fn ensure_members<OuterOrigin, AccountId>(
        o: OuterOrigin,
        n: MemberCount,
    ) -> result::Result<MemberCount, &'static str>
    where
        OuterOrigin: Into<result::Result<RawOrigin<AccountId>, OuterOrigin>>,
    {
        match o.into() {
            Ok(RawOrigin::Members(x, _)) if x >= n => Ok(n),
            _ => Err("bad origin: expected to be a threshold number of members"),
        }
    }

    pub struct EnsureMember<AccountId>(sp_std::marker::PhantomData<AccountId>);
    impl<
            O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>,
            AccountId: Default,
        > EnsureOrigin<O> for EnsureMember<AccountId>
    {
        type Success = AccountId;
        fn try_origin(o: O) -> Result<Self::Success, O> {
            o.into().and_then(|o| match o {
                RawOrigin::Member(id) => Ok(id),
                r => Err(O::from(r)),
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> O {
            O::from(RawOrigin::Member(Default::default()))
        }
    }

    pub struct EnsureMembers<N: U32, AccountId>(sp_std::marker::PhantomData<(N, AccountId)>);
    impl<
            O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>,
            N: U32,
            AccountId,
        > EnsureOrigin<O> for EnsureMembers<N, AccountId>
    {
        type Success = (MemberCount, MemberCount);
        fn try_origin(o: O) -> Result<Self::Success, O> {
            o.into().and_then(|o| match o {
                RawOrigin::Members(n, m) if n >= N::VALUE => Ok((n, m)),
                r => Err(O::from(r)),
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> O {
            O::from(RawOrigin::Members(N::VALUE, N::VALUE))
        }
    }

    pub struct EnsureProportionMoreThan<N: U32, D: U32, AccountId>(
        sp_std::marker::PhantomData<(N, D, AccountId)>,
    );
    impl<
            O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>,
            N: U32,
            D: U32,
            AccountId,
        > EnsureOrigin<O> for EnsureProportionMoreThan<N, D, AccountId>
    {
        type Success = ();
        fn try_origin(o: O) -> Result<Self::Success, O> {
            o.into().and_then(|o| match o {
                RawOrigin::Members(n, m) if n * D::VALUE > N::VALUE * m => Ok(()),
                r => Err(O::from(r)),
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> O {
            O::from(RawOrigin::Members(1u32, 0u32))
        }
    }

    pub struct EnsureProportionAtLeast<N: U32, D: U32, AccountId>(
        sp_std::marker::PhantomData<(N, D, AccountId)>,
    );
    impl<
            O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>,
            N: U32,
            D: U32,
            AccountId,
        > EnsureOrigin<O> for EnsureProportionAtLeast<N, D, AccountId>
    {
        type Success = ();
        fn try_origin(o: O) -> Result<Self::Success, O> {
            o.into().and_then(|o| match o {
                RawOrigin::Members(n, m) if n * D::VALUE >= N::VALUE * m => Ok(()),
                r => Err(O::from(r)),
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> O {
            O::from(RawOrigin::Members(0u32, 0u32))
        }
    }
}
