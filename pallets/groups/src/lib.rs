//! # Group Module
//!
//! ## Overview
//!
//! //TODO: The groups module provides mechanisms for proposals, group ownership of data etc
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For general users
//! * `create_group` - Creates a new **Group**
//!                    The creator transfers some funds to the **Group** account as part of creation.
//!
//! #### For **Group** members
//! * `update_group` - Members of a **Group** can update the group via a **Proposal**.
//! * `remove_group` - Members of a **Group** can remove the group via a **Proposal**.
//!                    Funds remaining in the **Group** account are transfered to the specified account.
//! * `create_sub_group` - A **Group** creates a **Sub-group**.
//!                        The some funds are transfered from the **Group** account to the **Sub-group** account as part of creation.
//! * `update_sub_group` - Members of a **Group** can update a sub_group via a **Proposal**.
//! * `remove_sub_group` - Members of a **Group** can remove a **Sub-group** of the **Group** via a **Proposal**.
//! * `execute` - A member of a **Group**/**Sub-group** can execute certain extrinsics on behalf of the **Group**.
//! * `propose` - A member of a **Group**/**Sub-group** can propose certain extrinsics on behalf of the **Group**.
//!               The caller specifies threshold and voting proceeds until that threshold is met or cannot be met.
//!               The threshold is not checked at this stage but is instead checked upon extrinsic execution and depends on the requirements of the extrinsic called.
//!               If the specified threshold is 1, the extrinsic is executed immediately.
//! * `vote` - A member of a **Group**/**Sub-group** can vote on pending **Proposals**
//! *          A member may change thier vote while the **Proposal** is still in progress, but there is an extra charge.
//! * `close` - After voting a caller should check the vote tallies and call `close` if the threshold is met or cannot be met.
//! * `veto` - A member of a **Group** can veto an ongoing **Proposal** (override the existing votes with either yay or nay).
//! * `withdraw_funds_group` - A **Group** can choose to withdraw funds from the group account to a chosen account via a **Proposal**
//! * `withdraw_funds_sub_group` - A **Group** can choose to withdraw funds from one of its sub_groups into its account via a **Proposal**
//! * `send_funds_to_sub_group` - A **Group** can choose to send funds from its account to one of its sub_groups into  via a **Proposal**
//! Note: adding funds to a group account from an individual account can be done using the built in substrate tranfer extrinsic
//!
//! ### RPC Methods
//!
//! * `member_of` - Get the collection of **Groups** that an account is a member of.
//! * `get_group` - Get a **Group**
//! * `get_sub_groups` - Get the collection of **Sub-groups** of a **Group**
//! * `get_proposal` - Get a **Proposal**
//! * `get_proposals` - Get the collection of outstanding **Proposals** of a **Group**
//! * `get_voting` - Get the current votes on a **Proposal**

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
    use frame_support::{
        codec::{Decode, Encode},
        dispatch::{DispatchResultWithPostInfo, Dispatchable, Parameter, PostDispatchInfo, Vec},
        ensure,
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement::AllowDeath, Get, ReservableCurrency},
        weights::{GetDispatchInfo, Weight},
    };
    use frame_system::{self as system, pallet_prelude::*};
    use group_info::GroupInfo;
    use primitives::{bounded_vec::BoundedVec, *};
    use sp_io::hashing::blake2_256;
    use sp_runtime::{
        traits::{
            AtLeast32Bit, AtLeast32BitUnsigned, CheckedAdd, Hash, One, Saturating,
            UniqueSaturatedInto, Zero,
        },
        Either,
    };
    use sp_std::{iter::Sum, prelude::*, vec};

    const MODULE_INDEX: u8 = 1;

    #[repr(u8)]
    pub enum ExtrinsicIndex {
        Group = 1,
        SubGroup = 2,
        Proposal = 3,
    }

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
        type MemberCount: Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + PartialEq
            + PartialOrd
            + Sum
            + Into<u32>;

        type Currency: ReservableCurrency<Self::AccountId>;

        /// The outer origin type.
        type Origin: From<
            RawOrigin<Self::AccountId, Self::GroupId, Self::ProposalId, Self::MemberCount>,
        >;

        /// This allows extrinsics to be executed via a Group account and it requires that the Group Threshold is met.
        type GroupsOriginByGroupThreshold: EnsureOrigin<
            <Self as frame_system::Config>::Origin,
            Success = (
                Self::GroupId,
                Self::ProposalId,
                Option<Self::MemberCount>,
                Option<Self::MemberCount>,
                Self::AccountId,
            ),
        >;
        /// This allows extrinsics to be executed via a Group account but it does not check the group threshold.
        /// The called extrinsic should require a threshold.
        type GroupsOriginByCallerThreshold: EnsureOrigin<
            <Self as frame_system::Config>::Origin,
            Success = (
                Self::GroupId,
                Self::ProposalId,
                Option<Self::MemberCount>,
                Option<Self::MemberCount>,
                Self::AccountId,
            ),
        >;
        /// This allows extrinsics to be executed via a Group account but there is no proposal or voting.
        /// Any member of the group may execute the extrinsic.
        /// The specific member account is also recorded, which may be useful.
        type GroupsOriginExecuted: EnsureOrigin<
            <Self as frame_system::Config>::Origin,
            Success = (Self::GroupId, Self::AccountId, Self::AccountId),
        >;
        /// This allows extrinsics to be executed either by individual accounts or via a Group account.
        /// If a group account is used, it requires that the Group Threshold is met.
        type GroupsOriginAccountOrThreshold: EnsureOrigin<
            <Self as frame_system::Config>::Origin,
            Success = Either<
                Self::AccountId,
                (
                    Self::GroupId,
                    Self::ProposalId,
                    Option<Self::MemberCount>,
                    Option<Self::MemberCount>,
                    Self::AccountId,
                ),
            >,
        >;
        /// This allows extrinsics to be executed either by individual accounts or via a Group account.
        /// If a group account is used, it does not check the group threshold. The called extrinsic should require a threshold.
        type GroupsOriginAccountOrApproved: EnsureOrigin<
            <Self as frame_system::Config>::Origin,
            Success = Either<
                Self::AccountId,
                (
                    Self::GroupId,
                    Self::ProposalId,
                    Option<Self::MemberCount>,
                    Option<Self::MemberCount>,
                    Self::AccountId,
                ),
            >,
        >;
        /// This allows extrinsics to be executed either by individual accounts or via a Group account.
        /// If a group account is used, there is no proposal or voting. Any member of the group may execute the extrinsic.
        /// The specific member account is also recorded, which may be useful.
        type GroupsOriginAccountOrExecuted: EnsureOrigin<
            <Self as frame_system::Config>::Origin,
            Success = Either<Self::AccountId, (Self::GroupId, Self::AccountId, Self::AccountId)>,
        >;

        type Proposal: Parameter
            + Dispatchable<Origin = <Self as Config>::Origin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>;

        /// Maximum number of proposals allowed to be active in parallel.        
        type MaxProposals: Get<u32>;
        /// Maximum length of proposal        
        type MaxProposalLength: Get<u32>;

        /// The maximum number of members supported by the pallet. Used for weight estimation.      
        type MaxMembers: Get<Self::MemberCount>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        type GetExtrinsicExtraSource: GetExtrinsicExtra<
            ModuleIndex = u8,
            ExtrinsicIndex = u8,
            AccountId = Self::AccountId,
        >;

        /// The maximum length of strings.
        type NameLimit: Get<u32>;
        /// The maximum length of parent child relationships.
        type GroupChainLimit: Get<u32>;
    }

    /// Origin for groups module proposals.

    #[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode)]
    pub enum RawOrigin<AccountId, GroupId, ProposalId, MemberCount> {
        /// It has been executed by a member of a group.
        /// (group_id,member_account,group_account)
        ProposalExecuted(GroupId, AccountId, AccountId),
        /// It has been condoned by a given number of members of the group.
        /// (group_id,yes_votes,no_votes,group_account)
        ProposalApproved(GroupId, ProposalId, MemberCount, MemberCount, AccountId),
        /// It has been approved by veto.
        /// (group_id,veto_account,group_account)
        ProposalApprovedByVeto(GroupId, ProposalId, AccountId, AccountId),
    }

    /// Origin for the groups module.
    #[pallet::origin]
    pub type Origin<T> = RawOrigin<
        <T as frame_system::Config>::AccountId,
        <T as Config>::ProposalId,
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
        T::MemberCount = "MemberCount",
        T::Currency = "Currency",
        <T::Currency as Currency<T::AccountId>>::Balance = "Balance",
        DispatchError = "DispatchError"  
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new Group was created
        /// (creator,group_id,anonymous_account,initial_balance)
        GroupCreated(
            T::AccountId,
            T::GroupId,
            T::AccountId,
            <T::Currency as Currency<T::AccountId>>::Balance,
        ),
        /// A Group was updated
        /// (group_id)
        GroupUpdated(T::GroupId),
        /// A Group was removed
        /// (group_id,return_funds_too)
        GroupRemoved(T::GroupId, T::AccountId),
        /// A new SubGroup was created
        /// (parent_group_id,sub_group_id,anonymous_account,initial_balance)
        SubGroupCreated(
            T::GroupId,
            T::GroupId,
            T::AccountId,
            <T::Currency as Currency<T::AccountId>>::Balance,
        ),
        /// A SubGroup was updated
        /// (parent_group_id,sub_group_id)
        SubGroupUpdated(T::GroupId, T::GroupId),
        /// A Group was removed
        /// (parent_group_id,sub_group_id)
        SubGroupRemoved(T::GroupId, T::GroupId),
        /// A motion was executed by a member of the group;
        /// (group_id,proposal_hash,member,success,error)
        Executed(
            T::GroupId,
            T::Hash,
            T::AccountId,
            bool,
            Option<DispatchError>,
        ),
        /// A new SubGroup was created 
        /// (proposer,group_id,proposal_id,threshold)
        Proposed(T::AccountId, T::GroupId, T::ProposalId, T::MemberCount),
        /// A proposal was voted on 
        /// (voter,group_id,proposal_id,approved)
        Voted(T::AccountId, T::GroupId, T::ProposalId, bool),
          /// A motion was approved by the required threshold and executed 
          /// (group_id,proposal_id,yes_votes,no_votes,success,error)
          Approved(
            T::GroupId,
            T::ProposalId,
            T::MemberCount,
            T::MemberCount,
            bool,
            Option<DispatchError>,
        ),
        /// A motion was disapproved by the required threshold 
        /// (group_id,proposal_id,yes_votes,no_votes)
        Disapproved(T::GroupId, T::ProposalId, T::MemberCount, T::MemberCount),
        /// A proposal was approved by veto 
        /// (vetoer,group_id,proposal_id,success,success)
        ApprovedByVeto(
            T::AccountId,
            T::GroupId,
            T::ProposalId,
            bool,
            Option<DispatchError>,
        ),
        /// A proposal was disapproved by veto 
        /// (vetoer,group_id,proposal_id)
        DisapprovedByVeto(T::AccountId, T::GroupId, T::ProposalId),
      
        /// funds were withdrawn from a group account 
        /// (group_id,target_account,amount,success)
        GroupFundsWithdrawn(
            T::GroupId,
            T::AccountId,
            <T::Currency as Currency<T::AccountId>>::Balance,
            bool,
        ),
        /// funds were withdrawn from a sub_group account 
        /// (group_id,sub_group_id,amount,success)
        SubGroupFundsWithdrawn(
            T::GroupId,
            T::GroupId,
            <T::Currency as Currency<T::AccountId>>::Balance,
            bool,
        ),
        /// funds were deposited from a group to a sub_group account 
        /// (group_id,sub_group_id,amount,success)
        SubGroupFundsDeposited(
            T::GroupId,
            T::GroupId,
            <T::Currency as Currency<T::AccountId>>::Balance,
            bool,
        ),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account is not a member
        NotMember,
        /// A group must have members
        MembersRequired,
        /// Invalid threshold provided when creating a group
        InvalidThreshold,
        /// A string exceeds the maximum allowed length
        StringLengthLimitExceeded,
        /// Failed to give requested balance to group account
        AccountCreationFailed,
        /// Duplicate proposals not allowed
        DuplicateProposal,
        /// Group must exist
        GroupMissing,
        /// Cannot remove a group that has children.
        GroupHasChildren,
        /// Proposal must exist
        ProposalMissing,
        /// Vote must exist
        VoteMissing,
        /// Mismatched index
        WrongIndex,
        /// Duplicate vote ignored
        DuplicateVote,
        /// Tried to close before sufficient votes
        VotingIncomplete,
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
        /// Group is not the parent of SubGroup
        NotParentGroup,
        /// User is not the group account (was not correctly called via proposal)
        NotGroupAccount,

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
        Blake2_128Concat,
        T::GroupId,
        Group<T::GroupId, T::AccountId, T::MemberCount, BoundedVec<u8, T::NameLimit>>,
        OptionQuery,
    >;

    /// Groups have members which are weighted.
    /// GroupId,AccountId => MemberCount
    #[pallet::storage]
    #[pallet::getter(fn group_members)]
    pub(super) type GroupMembers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::GroupId,
        Blake2_128Concat,
        T::AccountId,
        T::MemberCount,
        OptionQuery,
    >;

    /// Groups may have child groups
    /// GroupId,GroupId => ()
    #[pallet::storage]
    #[pallet::getter(fn group_children)]
    pub(super) type GroupChildren<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::GroupId,
        Blake2_128Concat,
        T::GroupId,
        (),
        OptionQuery,
    >;

    /// Groups may have proposals awaiting approval
    /// GroupId,ProposalId => Option<Proposal>
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub(super) type Proposals<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::GroupId,
        Blake2_128Concat,
        T::ProposalId,
        <T as Config>::Proposal,
        OptionQuery,
    >;

    /// Store proposal hashes by group to ensure uniqueness ie a group may not have two identical proposals at any one time
    /// GroupId, Hash => ()
    #[pallet::storage]
    #[pallet::getter(fn proposal_hashes)]
    pub(super) type ProposalHashes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::GroupId,
        Blake2_128Concat,
        T::Hash,
        (),
        OptionQuery,
    >;

    /// Votes on a given proposal, if it is ongoing.
    /// GroupId,ProposalId => Option<Votes>
    #[pallet::storage]
    #[pallet::getter(fn voting)]
    pub(super) type Voting<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::GroupId,
        Blake2_128Concat,
        T::ProposalId,
        Votes<T::AccountId, T::MemberCount>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new Group
        ///
        /// - `name`: The name of the group
        /// - `members`: The members of the group. If there are duplicates the last one will be used.
        /// - `threshold`: The threshold number of votes required to make modifications to the group.
        /// - `initial_balance`: The initial GRAMs to transfer from the sender to the group account to be used for calling extrinsics by the group.
        #[pallet::weight(
            T::WeightInfo::create_group(
                name.len() as u32,
                members.len() as u32
            )
        )]
        pub fn create_group(
            origin: OriginFor<T>,
            name: Vec<u8>,
            members: Vec<(T::AccountId, T::MemberCount)>,
            threshold: T::MemberCount,
            initial_balance: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(members.len() > 0, Error::<T>::MembersRequired);
            ensure!(threshold > Zero::zero(), Error::<T>::InvalidThreshold);

            let bounded_name = enforce_limit!(name);

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Group as u8),
                &sender,
            );

            let group_id = next_id!(NextGroupId<T>, T);

            let anonymous_account = Self::anonymous_account(&sender, group_id);

            let result = T::Currency::transfer(
                &sender,
                &anonymous_account,
                <T as Config>::Currency::minimum_balance() + initial_balance,
                AllowDeath,
            );

            ensure!(result.is_ok(), Error::<T>::AccountCreationFailed);
            let mut group =
                Group::<T::GroupId, T::AccountId, T::MemberCount, BoundedVec<u8, T::NameLimit>> {
                    name: bounded_name,
                    total_vote_weight: Zero::zero(),
                    threshold,
                    anonymous_account: anonymous_account.clone(),
                    parent: None,
                };

            Self::add_members(&mut group, group_id, members);

            <Groups<T>>::insert(group_id, group);

            Self::deposit_event(Event::GroupCreated(
                sender,
                group_id,
                anonymous_account,
                initial_balance,
            ));

            Ok(().into())
        }

        /// Update a Group.
        ///
        /// - `group_id`: Group to be updated
        /// - `name`: New group name
        /// - `add_members`: Add new members or overwrite existing member weights.
        /// - `threshold`: New threshold

        #[pallet::weight(T::WeightInfo::update_group(
            name.as_ref().map_or(0,|a|a.len()) as u32,
            add_members.as_ref().map_or(0,|a|a.len()) as u32,
            remove_members.as_ref().map_or(0,|a|a.len()) as u32,
        ))]
        pub fn update_group(
            origin: OriginFor<T>,          
            name: Option<Vec<u8>>,
            add_members: Option<Vec<(T::AccountId, T::MemberCount)>>,
            remove_members: Option<Vec<T::AccountId>>,
            threshold: Option<T::MemberCount>,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _proposal_id, _yes_votes, _no_votes, _caller_group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let bounded_name = enforce_limit_option!(name);           

            ensure!(
                <Groups<T>>::contains_key(caller_group_id),
                Error::<T>::GroupMissing
            );

            <Groups<T>>::mutate(caller_group_id, |group_option| {
                if let Some(group) = group_option {
                    if let Some(add_members) = add_members {
                        Self::add_members(group, caller_group_id, add_members);
                    }
                    if let Some(remove_members) = remove_members {
                        Self::remove_members(group, caller_group_id, remove_members);
                    }
                    if let Some(bounded_name) = bounded_name {
                        group.name = bounded_name;
                    }
                    if let Some(threshold) = threshold {
                        if threshold > Zero::zero() {
                            group.threshold = threshold;
                        }
                    }
                }
            });

            Self::deposit_event(Event::GroupUpdated(caller_group_id));

            Ok(().into())
        }

        /// Create a new SubGroup
        ///
        /// - `name`: The name of the group
        /// - `members`: The members of the group. Sender is automatically added to members if not already included.
        /// - `threshold`: The threshold number of votes required to make modifications to the group.
        /// - `initial_balance`: The initial GRAMs to transfer from the sender to the group account to be used for calling extrinsics by the group.       

        #[pallet::weight(T::WeightInfo::create_sub_group(
            name.len() as u32,
            members.len() as u32,
        ))]
        pub fn create_sub_group(
            origin: OriginFor<T>,
            name: Vec<u8>,
            members: Vec<(T::AccountId, T::MemberCount)>,
            threshold: T::MemberCount,
            initial_balance: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _proposal_id, _yes_votes, _no_votes, caller_group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            ensure!(members.len() > 0, Error::<T>::MembersRequired);
            ensure!(threshold > Zero::zero(), Error::<T>::InvalidThreshold);
            let bounded_name = enforce_limit!(name);

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::SubGroup as u8),
                &caller_group_account,
            );

            let sub_group_id = next_id!(NextGroupId<T>, T);

            let anonymous_account = Self::anonymous_account(&caller_group_account, sub_group_id);

            let result = <T as Config>::Currency::transfer(
                &caller_group_account,
                &anonymous_account,
                <T as Config>::Currency::minimum_balance() + initial_balance,
                AllowDeath,
            );

            ensure!(result.is_ok(), Error::<T>::AccountCreationFailed);

            let mut sub_group = Group {
                name: bounded_name,
                total_vote_weight: Zero::zero(),
                threshold,
                anonymous_account: anonymous_account.clone(),
                parent: Some(caller_group_id),
            };

            Self::add_members(&mut sub_group, sub_group_id, members);

            <Groups<T>>::insert(sub_group_id, sub_group);
            <GroupChildren<T>>::insert(caller_group_id, sub_group_id, ());

            Self::deposit_event(Event::SubGroupCreated(
                caller_group_id,
                sub_group_id,
                anonymous_account,
                initial_balance,
            ));

            Ok(().into())
        }

        /// Update a SubGroup.
        ///
        /// - `sub_group_id`: Group to be updated
        /// - `name`: New group name
        /// - `members`: New set of members. Old set will be overwritten, so sender should ensure they are included if desired.
        /// - `threshold`: New threshold       
        #[pallet::weight(T::WeightInfo::update_sub_group(
            name.as_ref().map_or(0,|a|a.len()) as u32,
            add_members.as_ref().map_or(0,|a|a.len()) as u32,
            remove_members.as_ref().map_or(0,|a|a.len()) as u32,
        ))]
        pub fn update_sub_group(
            origin: OriginFor<T>,
            sub_group_id: T::GroupId,
            name: Option<Vec<u8>>,
            add_members: Option<Vec<(T::AccountId, T::MemberCount)>>,
            remove_members: Option<Vec<T::AccountId>>,
            threshold: Option<T::MemberCount>,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _proposal_id, _yes_votes, _no_votes, _caller_group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let bounded_name = enforce_limit_option!(name);

            let group = Self::groups(sub_group_id).ok_or(Error::<T>::GroupMissing)?;

            ensure!(
                group.parent.is_some() && caller_group_id == group.parent.unwrap(),
                Error::<T>::NotParentGroup
            );

            <Groups<T>>::mutate(sub_group_id, |sub_group_option| {
                if let Some(sub_group) = sub_group_option {
                    if let Some(add_members) = add_members {
                        Self::add_members(sub_group, sub_group_id, add_members);
                    }
                    if let Some(remove_members) = remove_members {
                        Self::remove_members(sub_group, sub_group_id, remove_members);
                    }
                    if let Some(bounded_name) = bounded_name {
                        sub_group.name = bounded_name;
                    }
                    if let Some(threshold) = threshold {
                        if threshold > Zero::zero() {
                            sub_group.threshold = threshold;
                        }
                    }
                }
            });

            Self::deposit_event(Event::SubGroupUpdated(caller_group_id, sub_group_id));

            Ok(().into())
        }

        /// Remove a Group. Remove all child groups first. Can only be called via a proposal from the group. Funds returned to specified member.
        ///
        /// - `group_id`: Group to be updated
        /// - `return_funds_too`: account to transfer remaining group funds to
        //TODO: does remove_prefix only cost one db write?
        #[pallet::weight(T::WeightInfo::remove_group())]
        pub fn remove_group(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            return_funds_too: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _proposal_id, _yes_votes, _no_votes, _caller_group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            ensure!(caller_group_id == group_id, Error::<T>::NotGroupAccount);
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.parent.is_none(), Error::<T>::NotGroup);
            ensure!(
                <GroupMembers<T>>::contains_key(group_id, &return_funds_too),
                Error::<T>::NotMember
            );
            ensure!(
                <GroupChildren<T>>::iter_prefix(&group_id).next().is_none(),
                Error::<T>::GroupHasChildren
            );

            <T as Config>::Currency::transfer(
                &group.anonymous_account,
                &return_funds_too,
                <T as Config>::Currency::free_balance(&group.anonymous_account),
                AllowDeath,
            )?;

            <Groups<T>>::remove(&group_id);
            <GroupMembers<T>>::remove_prefix(&group_id);
            <Proposals<T>>::remove_prefix(&group_id);
            <ProposalHashes<T>>::remove_prefix(&group_id);

            Self::deposit_event(Event::GroupRemoved(group_id, return_funds_too));

            Ok(().into())
        }

        /// Remove a Sub-group. Remove all child groups first. Can only be called via a proposal from any group in the parent chain. Funds returned to immediate parent.
        ///
        /// - `sub_group_id`: Group to be updated   
        //TODO: does remove_prefix only cost one db write?
        #[pallet::weight(T::WeightInfo::remove_sub_group())]
        pub fn remove_sub_group(
            origin: OriginFor<T>,
            sub_group_id: T::GroupId,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _proposal_id, _yes_votes, _no_votes, _caller_group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let sub_group = Self::groups(sub_group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(sub_group.parent.is_some(), Error::<T>::NotSubGroup);
            ensure!(
                caller_group_id == sub_group.parent.unwrap(),
                Error::<T>::NotParentGroup
            );

            ensure!(
                <GroupChildren<T>>::iter_prefix(&sub_group_id)
                    .next()
                    .is_none(),
                Error::<T>::GroupHasChildren
            );
            let group = Self::groups(caller_group_id).ok_or(Error::<T>::GroupMissing)?;

            <T as Config>::Currency::transfer(
                &sub_group.anonymous_account,
                &group.anonymous_account,
                <T as Config>::Currency::free_balance(&sub_group.anonymous_account),
                AllowDeath,
            )?;

            <Groups<T>>::remove(&sub_group_id);
            <GroupChildren<T>>::remove(&caller_group_id, &sub_group_id);
            <GroupMembers<T>>::remove_prefix(&sub_group_id);
            <Proposals<T>>::remove_prefix(&sub_group_id);
            <ProposalHashes<T>>::remove_prefix(&sub_group_id);

            Self::deposit_event(Event::SubGroupRemoved(caller_group_id, sub_group_id));

            Ok(().into())
        }

        /// Execute a proposal. Use for extrinsics that don't require voting.
        ///
        /// Requires the sender to be member.
        ///
        /// - `group_id`: Group executing the extrinsic
        /// - `proposal`: Proposal to be executed
        /// - `length_bound`: The length of the Proposal for weight estimation       
        #[pallet::weight(T::WeightInfo::execute(
            *length_bound as u32
        ).saturating_add(proposal.get_dispatch_info().weight))]
        pub fn execute(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            proposal: Box<<T as Config>::Proposal>,
            length_bound: u32,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(
                <GroupMembers<T>>::contains_key(group_id, &sender),
                Error::<T>::NotMember
            );
            let proposal_len = proposal.using_encoded(|x| x.len());
            ensure!(
                proposal_len <= length_bound as usize,
                Error::<T>::WrongProposalLength
            );
            let proposal_hash = T::Hashing::hash_of(&proposal);

            let result = proposal.dispatch(
                RawOrigin::ProposalExecuted(
                    group_id,
                    sender.clone(),
                    group.anonymous_account.clone(),
                )
                .into(),
            );

            Self::deposit_event(Event::Executed(
                group_id,
                proposal_hash,
                sender,
                result.is_ok(),
                result.err().map(|err| err.error),
            ));

            Ok(Self::get_result_weight(result)
                .map(|w| T::WeightInfo::execute(proposal_len as u32).saturating_add(w))
                .into())
        }

        /// Add a new proposal to either be voted on or executed directly.
        ///
        /// Requires the sender to be member.
        ///
        /// - `group_id`: Group executing the extrinsic
        /// - `proposal`: Proposal to be executed
        /// - `threshold`: Declaration of the threshold required - will be checked by the extrinsic after approval.
        /// - `length_bound`: The length of the Proposal for weight estimation        

        #[pallet::weight(
            if *threshold == 1u32.into() {
                T::WeightInfo::propose_execute(
                    *length_bound,                   
                ).saturating_add(proposal.get_dispatch_info().weight)
            } else {
                T::WeightInfo::propose_proposed(
                    *length_bound,                   
                )
            }
        )]
        pub fn propose(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            proposal: Box<<T as Config>::Proposal>,
            threshold: T::MemberCount,
            length_bound: u32,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            let weight_maybe = <GroupMembers<T>>::get(group_id, &sender);
            ensure!(weight_maybe.is_some(), Error::<T>::NotMember);
            let weight = weight_maybe.unwrap();
            let proposal_len = proposal.using_encoded(|x| x.len());
            let proposal_hash = T::Hashing::hash_of(&proposal);
            ensure!(
                !<ProposalHashes<T>>::contains_key(group_id, proposal_hash),
                Error::<T>::DuplicateProposal
            );
            ensure!(
                proposal_len <= length_bound as usize,
                Error::<T>::WrongProposalLength
            );

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Proposal as u8),
                &sender,
            );
            
            let proposal_id = next_id!(NextProposalId<T>, T);

            if threshold == weight {
                //TODO: should we create votes here?
                let result = proposal.dispatch(
                    RawOrigin::ProposalApproved(
                        group_id,
                        proposal_id,
                        weight,
                        0u32.into(),
                        group.anonymous_account.clone(),
                    )
                    .into(),
                );

                Self::deposit_event(Event::Approved(
                    group_id,
                    proposal_id,
                    weight,
                    0u32.into(),
                    result.is_ok(),
                    result.err().map(|err| err.error),
                ));

                Ok(Self::get_result_weight(result)
                    .map(|w| {
                        T::WeightInfo::propose_execute(
                            proposal_len as u32,                            
                        )
                        .saturating_add(w)
                    })
                    .into())
            } else {
                <ProposalHashes<T>>::insert(group_id, proposal_hash, ());
                <Proposals<T>>::insert(group_id, proposal_id, proposal);

                let votes = Votes {
                    threshold,
                    total_vote_weight: group.total_vote_weight,
                    ayes: vec![(sender.clone(), weight)],
                    nays: vec![],
                    veto: None,
                };
                <Voting<T>>::insert(group_id, proposal_id, votes);

                Self::deposit_event(Event::Proposed(sender, group_id, proposal_id, threshold));

                Ok(().into())
            }
        }

        /// Vote on a Proposal
        ///
        /// - `group_id`: Group executing the extrinsic
        /// - `proposal_id`: Proposal to be voted on
        /// - `approve`: approval.
        #[pallet::weight(T::WeightInfo::vote(
            T::MaxMembers::get().into()
        ))]
        pub fn vote(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            proposal_id: T::ProposalId,
            approve: bool,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            let weight_maybe = <GroupMembers<T>>::get(group_id, &sender);
            ensure!(weight_maybe.is_some(), Error::<T>::NotMember);
            let weight = weight_maybe.unwrap();
            let mut voting = Self::voting(group_id, proposal_id).ok_or(Error::<T>::VoteMissing)?;

            let position_yes = voting
                .ayes
                .iter()
                .position(|(account, _)| account == &sender);
            let position_no = voting
                .nays
                .iter()
                .position(|(account, _)| account == &sender);

            // Detects first vote of the member in the motion
            let is_account_voting_first_time = position_yes.is_none() && position_no.is_none();

            if approve {
                if position_yes.is_none() {
                    voting.ayes.push((sender.clone(), weight));
                } else {
                    Err(Error::<T>::DuplicateVote)?
                }
                if let Some(pos) = position_no {
                    voting.nays.swap_remove(pos);
                }
            } else {
                if position_no.is_none() {
                    voting.nays.push((sender.clone(), weight));
                } else {
                    Err(Error::<T>::DuplicateVote)?
                }
                if let Some(pos) = position_yes {
                    voting.ayes.swap_remove(pos);
                }
            }

            Self::deposit_event(Event::Voted(sender, group_id, proposal_id, approve));

            Voting::<T>::insert(group_id, proposal_id, voting);

            if is_account_voting_first_time {
                Ok((
                    Some(<T as Config>::WeightInfo::vote(
                        group.total_vote_weight.unique_saturated_into(),
                    )),
                    Pays::No,
                )
                    .into())
            } else {
                Ok((
                    Some(<T as Config>::WeightInfo::vote(
                        group.total_vote_weight.unique_saturated_into(),
                    )),
                    Pays::Yes,
                )
                    .into())
            }
        }

        /// Close a Proposal. Caller of vote should check if the vote will be approved/disaproved and call close if it will. 
        /// Anyone can trigger a close, they don't have to be a member. It could for example be triggered by a service that watches voting.
        /// TODO: payments?
        ///
        /// - `group_id`: Group executing the extrinsic
        /// - `proposal_id`: Proposal to be closed
        /// - `proposal_weight_bound`: maximum expected weight of the proposal.
        /// - `length_bound`: length of the proposal.
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        #[pallet::weight(	{
            let a = *length_bound;
            let m = T::MaxMembers::get().into();
            let p1 = *proposal_weight_bound;            
                T::WeightInfo::close_approved(a, m)
                .max(T::WeightInfo::close_disapproved(m))
                .saturating_add(p1)
        })]
        pub fn close(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            proposal_id: T::ProposalId,
            proposal_weight_bound: Weight,
            length_bound: u32,
        ) -> DispatchResultWithPostInfo {
            let _sender = ensure_signed(origin)?;
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;            

            let voting = Self::voting(group_id, proposal_id).ok_or(Error::<T>::VoteMissing)?;

            let yes_votes = voting.ayes.into_iter().map(|(_, w)| w).sum();
            let no_votes = voting.nays.into_iter().map(|(_, w)| w).sum();            

            let approved = yes_votes >= voting.threshold;
            let disapproved = voting.total_vote_weight.saturating_sub(no_votes) < voting.threshold;

            let proposal =
                Self::proposals(group_id, proposal_id).ok_or(Error::<T>::ProposalMissing)?;

            let proposal_hash = T::Hashing::hash_of(&proposal);

            if approved {
                let proposal_len = proposal.using_encoded(|x| x.len());
                ensure!(
                    proposal_len <= length_bound as usize,
                    Error::<T>::WrongProposalLength
                );

                let dispatch_weight = proposal.get_dispatch_info().weight;
                ensure!(
                    dispatch_weight <= proposal_weight_bound,
                    Error::<T>::WrongProposalWeight
                );

                let result = proposal.dispatch(
                    RawOrigin::ProposalApproved(
                        group_id,
                        proposal_id,
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
                    result.err().map(|err| err.error),
                ));

                <Proposals<T>>::remove(group_id, proposal_id);
                <ProposalHashes<T>>::remove(group_id, proposal_hash);

                let proposal_weight = Self::get_result_weight(result).unwrap_or(dispatch_weight);

                return Ok((
                    Some(
                        T::WeightInfo::close_approved(
                            proposal_len as u32,
                            T::MaxMembers::get().into(),
                            
                        )
                        .saturating_add(proposal_weight),
                    ),
                    Pays::Yes,
                )
                    .into());
            } else if disapproved {
                Self::deposit_event(Event::Disapproved(
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                ));

                <Proposals<T>>::remove(group_id, proposal_id);
                <ProposalHashes<T>>::remove(group_id, proposal_hash);                
                return Ok((
                    Some(T::WeightInfo::close_disapproved(
                        T::MaxMembers::get().into()
                    )),
                    Pays::Yes,
                )
                    .into());
            } else {
                ensure!(false, Error::<T>::VotingIncomplete);
            };

            //this never happens
            Ok(().into())
        }

        /// Veto a Proposal
        ///
        /// - `group_id`: Group executing the extrinsic
        /// - `proposal_id`: Proposal to be closed
        /// - `approve`: approval.
        /// - `proposal_weight_bound`: maximum expected weight of the proposal.
        /// - `length_bound`: length of the proposal.
        #[pallet::weight(	{
            let a = *length_bound;            
            let p = *proposal_weight_bound;          
                T::WeightInfo::veto_approved(a,  )
                .max(T::WeightInfo::veto_disapproved())
                .saturating_add(p)
        })]
        pub fn veto(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            proposal_id: T::ProposalId,
            approve: bool,
            proposal_weight_bound: Weight,
            length_bound: u32,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.parent.is_some(), Error::<T>::NotSubGroup);
            let parent_group_id = group.parent.unwrap();
            ensure!(
                <GroupMembers<T>>::contains_key(parent_group_id, &sender),
                Error::<T>::NotParentGroup
            );

            let proposal =
                Self::proposals(group_id, proposal_id).ok_or(Error::<T>::ProposalMissing)?;
            let proposal_hash = T::Hashing::hash_of(&proposal);
            <Voting<T>>::mutate(group_id, proposal_id, |votes_option| {
                if let Some(votes) = votes_option {
                    votes.veto = Some(approve);
                }
            });

            if approve {
                let proposal_len = proposal.using_encoded(|x| x.len());
                ensure!(
                    proposal_len <= length_bound as usize,
                    Error::<T>::WrongProposalLength
                );
                let dispatch_weight = proposal.get_dispatch_info().weight;
                ensure!(
                    dispatch_weight <= proposal_weight_bound,
                    Error::<T>::WrongProposalWeight
                );

                let result = proposal.dispatch(
                    RawOrigin::ProposalApprovedByVeto(
                        group_id,
                        proposal_id,
                        sender.clone(),
                        group.anonymous_account.clone(),
                    )
                    .into(),
                );

                Self::deposit_event(Event::ApprovedByVeto(
                    sender,
                    group_id,
                    proposal_id,
                    result.is_ok(),
                    result.err().map(|err| err.error),
                ));

                <Proposals<T>>::remove(group_id, proposal_id);
                <ProposalHashes<T>>::remove(group_id, proposal_hash);

                let proposal_weight = Self::get_result_weight(result).unwrap_or(dispatch_weight);

                return Ok((
                    Some(
                        T::WeightInfo::veto_approved(
                            proposal_len as u32                          
                        )
                        .saturating_add(proposal_weight),
                    ),
                    Pays::Yes,
                )
                    .into());
            } else {
                Self::deposit_event(Event::DisapprovedByVeto(sender, group_id, proposal_id));
                <Proposals<T>>::remove(group_id, proposal_id);
                <ProposalHashes<T>>::remove(group_id, proposal_hash);

                return Ok((
                    Some(T::WeightInfo::veto_disapproved()),
                    Pays::Yes,
                )
                    .into());
            };
        }
        /// Withdraw funds from the group account
        ///
        /// - `target_account`: account to withdraw the funds to
        /// - `amount`: amount to withdraw
        #[pallet::weight(T::WeightInfo::withdraw_funds_group())]
        pub fn withdraw_funds_group(
            origin: OriginFor<T>,
            target_account: T::AccountId,
            amount: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _, _, _, group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.parent.is_none(), Error::<T>::NotGroup);

            let result = <T as Config>::Currency::transfer(
                &group_account,
                &target_account,
                amount,
                AllowDeath,
            );

            Self::deposit_event(Event::GroupFundsWithdrawn(
                group_id,
                target_account,
                amount,
                result.is_ok(),
            ));

            Ok(().into())
        }

        /// Withdraw funds from a subgroup account to the group account
        ///
        /// - `sub_group_id`: Subgroup to withdraw funds from        
        /// - `amount`: amount to withdraw     
        #[pallet::weight(T::WeightInfo::withdraw_funds_sub_group())]
        pub fn withdraw_funds_sub_group(
            origin: OriginFor<T>,
            sub_group_id: T::GroupId,
            amount: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _, _, _, caller_group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let sub_group = Self::groups(sub_group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(sub_group.parent.is_some(), Error::<T>::NotSubGroup);
            let parent_group_id = sub_group.parent.unwrap();
            ensure!(
                caller_group_id == parent_group_id,
                Error::<T>::NotParentGroup
            );

            let result = <T as Config>::Currency::transfer(
                &sub_group.anonymous_account,
                &caller_group_account,
                amount,
                AllowDeath,
            );

            Self::deposit_event(Event::SubGroupFundsWithdrawn(
                parent_group_id,
                sub_group_id,
                amount,
                result.is_ok(),
            ));

            Ok(().into())
        }

        /// Deposit funds to a subgroup account from the group account
        ///
        /// - `sub_group_id`: Subgroup to deposit funds to
        /// - `amount`: amount to send
        #[pallet::weight(T::WeightInfo::send_funds_to_sub_group())]
        pub fn send_funds_to_sub_group(
            origin: OriginFor<T>,
            sub_group_id: T::GroupId,
            amount: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _, _, _, caller_group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let sub_group = Self::groups(sub_group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(sub_group.parent.is_some(), Error::<T>::NotSubGroup);
            let parent_group_id = sub_group.parent.unwrap();
            ensure!(
                caller_group_id == parent_group_id,
                Error::<T>::NotParentGroup
            );

            let result = <T as Config>::Currency::transfer(
                &caller_group_account,
                &sub_group.anonymous_account,
                amount,
                AllowDeath,
            );

            Self::deposit_event(Event::SubGroupFundsDeposited(
                parent_group_id,
                sub_group_id,
                amount,
                result.is_ok(),
            ));

            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --
        pub fn member_of(account: T::AccountId) -> Vec<T::GroupId> {
            <Groups<T>>::iter()
                .filter_map(|(group_id, _)| {
                    <GroupMembers<T>>::contains_key(group_id, &account).then(|| group_id)
                })
                .collect()
        }

        pub fn get_group(
            group_id: T::GroupId,
        ) -> Option<(
            Group<T::GroupId, T::AccountId, T::MemberCount, BoundedVec<u8, T::NameLimit>>,
            Vec<(T::AccountId, T::MemberCount)>,
        )> {
            <Groups<T>>::get(group_id).map(|group| {
                let members = <GroupMembers<T>>::iter_prefix(group_id)
                    .map(|(account, weight)| (account, weight))
                    .collect();
                (group, members)
            })
        }
        pub fn get_sub_groups(
            group_id: T::GroupId,
        ) -> Vec<(
            T::GroupId,
            Group<T::GroupId, T::AccountId, T::MemberCount, BoundedVec<u8, T::NameLimit>>,
            Vec<(T::AccountId, T::MemberCount)>,
        )> {
            <GroupChildren<T>>::iter_prefix(group_id)
                .filter_map(|(child_group_id, _)| {
                    <Groups<T>>::get(group_id).map(|group| {
                        let members = <GroupMembers<T>>::iter_prefix(child_group_id)
                            .map(|(account, weight)| (account, weight))
                            .collect();
                        (child_group_id, group, members)
                    })
                })
                .collect()
        }

        pub fn get_proposal(
            group_id: T::GroupId,
            proposal_id: T::ProposalId,
        ) -> Option<(T::Hash, u32)> {
            <Proposals<T>>::get(group_id, proposal_id).map(|proposal| {
                (
                    T::Hashing::hash_of(&proposal),
                    proposal.encoded_size() as u32,
                )
            })
        }

        pub fn get_proposals(group_id: T::GroupId) -> Vec<(T::ProposalId, T::Hash, u32)> {
            <Proposals<T>>::iter_prefix(group_id)
                .map(|(proposal_id, proposal)| {
                    (
                        proposal_id,
                        T::Hashing::hash_of(&proposal),
                        proposal.encoded_size() as u32,
                    )
                })
                .collect()
        }

        pub fn get_voting(
            group_id: T::GroupId,
            proposal_id: T::ProposalId,
        ) -> Option<Votes<T::AccountId, T::MemberCount>> {
            <Voting<T>>::get(group_id, proposal_id)
        }

        // -- private functions --

        fn add_members(
            group: &mut Group<
                T::GroupId,
                T::AccountId,
                T::MemberCount,
                BoundedVec<u8, T::NameLimit>,
            >,
            group_id: T::GroupId,
            members: Vec<(T::AccountId, T::MemberCount)>,
        ) {
            members.into_iter().for_each(|(account, mut weight)| {
                if weight == Zero::zero() {
                    weight = 1u32.into();
                }
                let old_member_maybe = <GroupMembers<T>>::get(group_id, &account);
                if let Some(old_weight) = old_member_maybe {
                    group.total_vote_weight -= old_weight;
                }
                group.total_vote_weight += weight;
                <GroupMembers<T>>::insert(group_id, &account, weight);
            });
        }

        fn remove_members(
            group: &mut Group<
                T::GroupId,
                T::AccountId,
                T::MemberCount,
                BoundedVec<u8, T::NameLimit>,
            >,
            group_id: T::GroupId,
            members: Vec<T::AccountId>,
        ) {
            members.iter().for_each(|account| {
                let old_member_maybe = <GroupMembers<T>>::get(group_id, account);
                if let Some(old_weight) = old_member_maybe {
                    group.total_vote_weight -= old_weight;
                }
                <GroupMembers<T>>::remove(group_id, account);
            });
        }

        fn is_member(group_id: T::GroupId, account_id: &T::AccountId) -> bool {
            <GroupMembers<T>>::contains_key(group_id, account_id)
        }

        fn is_group_account(group_id: T::GroupId, account_id: &T::AccountId) -> bool {
            let group = <Groups<T>>::get(group_id);
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
    }

    impl<T: Config> GroupInfo for Module<T> {
        type AccountId = T::AccountId;
        type GroupId = T::GroupId;

        fn is_member(group_id: Self::GroupId, account_id: &Self::AccountId) -> bool {
            Self::is_member(group_id, account_id)
        }
        fn is_group_account(group_id: Self::GroupId, account_id: &Self::AccountId) -> bool {
            Self::is_group_account(group_id, account_id)
        }
    }

    /// This just verifies that the origin came from a proposal. It does NOT do any threshold checks. The proposer specifies a threshold and that is used for voting. It is up to the recieving extrinsic to enforce a threshold.
    pub struct EnsureApproved<T>(sp_std::marker::PhantomData<T>);
    impl<
            O: Into<Result<RawOrigin<T::AccountId, T::GroupId, T::ProposalId, T::MemberCount>, O>>
                + From<RawOrigin<T::AccountId, T::GroupId, T::ProposalId, T::MemberCount>>,
            T: Config,
        > EnsureOrigin<O> for EnsureApproved<T>
    {
        type Success = (
            T::GroupId,
            T::ProposalId,
            Option<T::MemberCount>,
            Option<T::MemberCount>,
            T::AccountId,
        );
        fn try_origin(o: O) -> Result<Self::Success, O> {
            o.into().and_then(|o| match o {
                RawOrigin::ProposalApproved(
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                    group_account,
                ) => Ok((
                    group_id,
                    proposal_id,
                    Some(yes_votes),
                    Some(no_votes),
                    group_account,
                )),
                RawOrigin::ProposalApprovedByVeto(group_id, proposal_id, _, group_account) => {
                    Ok((group_id, proposal_id, None, None, group_account))
                }
                r => Err(O::from(r)),
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> O {
            let group_id: T::GroupId = 1u32.into();
            let proposal_id: T::ProposalId = 1u32.into();
            let group = Groups::<T>::get(group_id).unwrap();
            O::from(RawOrigin::ProposalApprovedByVeto(
                group_id,
                proposal_id,
                T::AccountId::default(),
                group.anonymous_account.clone(),
            ))
        }
    }
    /// This just verifies that the origin came from a proposal. It does NOT do any threshold checks. The proposer specifies a threshold and that is used for voting. It is up to the recieving extrinsic to enforce a threshold.
    pub struct EnsureExecuted<T>(sp_std::marker::PhantomData<T>);
    impl<
            O: Into<Result<RawOrigin<T::AccountId, T::GroupId, T::ProposalId, T::MemberCount>, O>>
                + From<RawOrigin<T::AccountId, T::GroupId, T::ProposalId, T::MemberCount>>,
            T: Config,
        > EnsureOrigin<O> for EnsureExecuted<T>
    {
        type Success = (T::GroupId, T::AccountId, T::AccountId);
        fn try_origin(o: O) -> Result<Self::Success, O> {
            o.into().and_then(|o| match o {
                RawOrigin::ProposalExecuted(group_id, member, group_account) => {
                    Ok((group_id, member, group_account))
                }
                r => Err(O::from(r)),
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> O {
            let group_id: T::GroupId = 1u32.into();
            let group = Groups::<T>::get(group_id).unwrap();
            O::from(RawOrigin::ProposalExecuted(
                group_id,
                T::AccountId::default(),
                group.anonymous_account.clone(),
            ))
        }
    }

    /// This verifies that the origin came from a proposal and enforces the threshold is at least as great as the group threshold. The proposer specifies a threshold and voting proceeds until that threshold.

    pub struct EnsureThreshold<T>(sp_std::marker::PhantomData<T>);
    impl<
            O: Into<Result<RawOrigin<T::AccountId, T::GroupId, T::ProposalId, T::MemberCount>, O>>
                + From<RawOrigin<T::AccountId, T::GroupId, T::ProposalId, T::MemberCount>>,
            T: Config,
        > EnsureOrigin<O> for EnsureThreshold<T>
    {
        type Success = (
            T::GroupId,
            T::ProposalId,
            Option<T::MemberCount>,
            Option<T::MemberCount>,
            T::AccountId,
        );
        fn try_origin(o: O) -> Result<Self::Success, O> {
            o.into().and_then(|o| match o.clone() {
                RawOrigin::ProposalApproved(
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                    group_account,
                ) => {
                    let group_maybe = <Groups<T>>::get(group_id);
                    match group_maybe {
                        Some(group) => {
                            if yes_votes >= group.threshold {
                                Ok((
                                    group_id,
                                    proposal_id,
                                    Some(yes_votes),
                                    Some(no_votes),
                                    group_account,
                                ))
                            } else {
                                Err(O::from(o))
                            }
                        }
                        None => Err(O::from(o)),
                    }
                }
                RawOrigin::ProposalApprovedByVeto(group_id, proposal_id, _, group_account) => {
                    Ok((group_id, proposal_id, None, None, group_account))
                }
                r => Err(O::from(r)),
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> O {
            let group_id: T::GroupId = 1u32.into();
            let proposal_id: T::ProposalId = 1u32.into();
            let group = Groups::<T>::get(group_id).unwrap();
            O::from(RawOrigin::ProposalApprovedByVeto(
                group_id,
                proposal_id,
                T::AccountId::default(),
                group.anonymous_account.clone(),
            ))
        }
    }
}
