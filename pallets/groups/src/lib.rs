//! # Group Module
//!
//! ## Overview
//!
//! An asset registry is a data registry that mediates the creation, verification, updating, and
//! deactivation of digital and physical assets. Any account holder can create an asset registry.
//! An asset can be owned, shared and transferred to an account or a DID.
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
//!
//! ### RPC Methods
//!
//! * `member_of` - Get the collection of **Groups** that an account is a member of.
//! * `get_sub_groups` - Get the collection of **Sub-groups** of a **Group**
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
            UniqueSaturatedFrom, Zero,
        },
        Either,
    };
    use sp_std::{prelude::*, vec};

    const MODULE_INDEX: u8 = 1;

    #[repr(u8)]
    pub enum ExtrinsicIndex {
        Group = 1,
        SubGroup = 2,
        //TODO: charge for proposals.
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
            + Into<u32>;

        type Currency: ReservableCurrency<Self::AccountId>;

        /// The outer origin type.
        type Origin: From<RawOrigin<Self::AccountId, Self::GroupId, Self::MemberCount>>;

        /// This allows extrinsics to be executed via a Group account and it requires that the Group Threshold is met.
        type GroupsOriginByGroupThreshold: EnsureOrigin<
            <Self as frame_system::Config>::Origin,
            Success = (
                Self::GroupId,
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
    pub enum RawOrigin<AccountId, GroupId, MemberCount> {
        /// It has been executed by a member of a group.
        /// (group_id,member_account,group_account)
        ProposalExecuted(GroupId, AccountId, AccountId),
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
        /// A new Group was created
        /// (creator,group_id,annonymous_account)
        GroupCreated(T::AccountId, T::GroupId, T::AccountId),
        /// A new SubGroup was created
        /// (parent_group_id,group_id,annonymous_account)
        SubGroupCreated(T::GroupId, T::GroupId, T::AccountId),
        /// A Group was updated
        /// (admin_group_id,group_id)
        GroupUpdated(T::GroupId, T::GroupId),
        /// A Group was removed
        /// (group_id,return_funds_too)
        GroupRemoved(T::GroupId, T::AccountId),
        /// A Group was removed
        /// (admin_group_id,group_id)
        SubGroupRemoved(T::GroupId, T::GroupId),
        /// A motion was executed by a member of the group;
        /// (group_id,proposal_hash,member,success)
        Executed(T::GroupId, T::Hash, T::AccountId, bool),
        /// A new SubGroup was created (proposer,group_id,proposal_id,threshold)
        Proposed(T::AccountId, T::GroupId, T::ProposalId, T::MemberCount),
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
        BadString,
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

    /// Groups may have child groups
    /// GroupId => Vec<GroupId>
    #[pallet::storage]
    #[pallet::getter(fn group_children)]
    pub(super) type GroupChildren<T: Config> =
        StorageMap<_, Blake2_128Concat, T::GroupId, Vec<T::GroupId>, OptionQuery>;

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

    //TODO: use doublemap to allow for much greater numbers

    /// Store vec of proposal hashes by group to ensure uniqueness ie a group may not have two identical proposals at any one time
    /// GroupId => Vec<T::Hash>
    #[pallet::storage]
    #[pallet::getter(fn proposal_hashes)]
    pub(super) type ProposalHashes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::GroupId, Vec<T::Hash>, OptionQuery>;

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
        /// - `members`: The members of the group. Sender is automatically added to members if not already included.
        /// - `threshold`: The threshold number of votes required to make modifications to the group.
        /// - `initial_balance`: The initial GRAMs to transfer from the sender to the group account to be used for calling extrinsics by the group.
        #[pallet::weight(
            T::WeightInfo::create_group(
                name.len() as u32,
                (members.len() +1) as u32
            )
        )]
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

            let bounded_name = enforce_limit!(name);

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Group as u8),
                &sender,
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

            members.sort();

            let group = Group {
                parent: None,
                name: bounded_name,
                members,
                threshold,
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
            mut members: Vec<T::AccountId>,
            threshold: T::MemberCount,
            initial_balance: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _yes_votes, _no_votes, caller_group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            ensure!(members.len() > 0, Error::<T>::MembersRequired);
            ensure!(
                threshold > Zero::zero()
                    && threshold <= T::MemberCount::unique_saturated_from(members.len() as u128),
                Error::<T>::InvalidThreshold
            );
            let bounded_name = enforce_limit!(name);

            let mut admin_group = Self::groups(caller_group_id).ok_or(Error::<T>::GroupMissing)?;
            let mut admin_group_id = caller_group_id;
            while admin_group.parent.is_some() {
                admin_group_id = admin_group.parent.unwrap();
                admin_group = Self::groups(admin_group_id).ok_or(Error::<T>::GroupMissing)?;
            }

            T::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::SubGroup as u8),
                &caller_group_account,
            );

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

            members.sort();

            let group = Group {
                parent: Some(caller_group_id),
                name: bounded_name,
                members,
                threshold,
                anonymous_account: anonymous_account.clone(),
            };

            <Groups<T>>::insert(group_id, group);
            <GroupChildren<T>>::mutate_exists(caller_group_id, |maybe_group_children| {
                if let Some(ref mut group_children) = maybe_group_children {
                    group_children.push(group_id);
                }
            });
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
        /// - `group_id`: Group to be updated
        /// - `name`: New group name
        /// - `members`: New set of members. Old set will be overwritten, so sender should ensure they are included if desired.
        /// - `threshold`: New threshold

        #[pallet::weight(T::WeightInfo::update_group(
        name.as_ref().map_or(0,|a|a.len()) as u32,
        members.as_ref().map_or(0,|a|a.len()) as u32,
    ))]
        pub fn update_group(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            name: Option<Vec<u8>>,
            members: Option<Vec<T::AccountId>>,
            threshold: Option<T::MemberCount>,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _yes_votes, _no_votes, _caller_group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;

            let bounded_name = enforce_limit_option!(name);

            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            let mut admin_group = group;
            let mut admin_group_id = group_id;
            while caller_group_id != group_id && admin_group.parent.is_some() {
                admin_group_id = admin_group.parent.unwrap();
                admin_group = Self::groups(admin_group_id).ok_or(Error::<T>::GroupMissing)?;
            }
            ensure!(caller_group_id == group_id, Error::<T>::NotGroupAdmin);

            if let Some(ref members) = members {
                ensure!(members.len() > 0, Error::<T>::MembersRequired)
            }

            <Groups<T>>::mutate(group_id, |group_option| {
                if let Some(group) = group_option {
                    if let Some(bounded_name) = bounded_name {
                        group.name = bounded_name;
                    }
                    if let Some(mut members) = members {
                        members.sort();
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

        /// Remove a Group. Remove all child groups first. Can only be called via a proposal from the group. Funds returned to specified member.
        ///
        /// - `group_id`: Group to be updated
        /// - `return_funds_too`: account to transfer remaining group funds to

        #[pallet::weight(T::WeightInfo::remove_group(
            T::MaxProposals::get() as u32,
        ))]
        pub fn remove_group(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            return_funds_too: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _yes_votes, _no_votes, _caller_group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;
            ensure!(caller_group_id == group_id, Error::<T>::NotGroupAccount);
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.parent.is_none(), Error::<T>::NotGroup);
            ensure!(
                Self::is_member(group_id, &return_funds_too),
                Error::<T>::NotMember
            );

            ensure!(
                !<GroupChildren<T>>::contains_key(&group_id)
                    || <GroupChildren<T>>::get(&group_id).unwrap().len() == 0,
                Error::<T>::GroupHasChildren
            );

            <T as Config>::Currency::transfer(
                &group.anonymous_account,
                &return_funds_too,
                <T as Config>::Currency::free_balance(&group.anonymous_account),
                AllowDeath,
            )?;

            let proposal_count = <Proposals<T>>::drain_prefix(&group_id).count();

            <Groups<T>>::remove(&group_id);
            <GroupChildren<T>>::remove(&group_id);
            <ProposalHashes<T>>::remove(&group_id);

            Self::deposit_event(Event::GroupRemoved(group_id, return_funds_too));

            Ok((Some(T::WeightInfo::remove_group(proposal_count as u32))).into())
        }

        /// Remove a Sub-group. Remove all child groups first. Can only be called via a proposal from any group in the parent chain. Funds returned to immediate parent.
        ///
        /// - `subgroup_id`: Group to be updated   
        #[pallet::weight(T::WeightInfo::remove_group(
            T::MaxProposals::get() as u32,
        ))]
        pub fn remove_sub_group(
            origin: OriginFor<T>,
            subgroup_id: T::GroupId,
        ) -> DispatchResultWithPostInfo {
            let (caller_group_id, _yes_votes, _no_votes, _caller_group_account) =
                T::GroupsOriginByGroupThreshold::ensure_origin(origin)?;
            ensure!(caller_group_id != subgroup_id, Error::<T>::NotSubGroup);
            let group = Self::groups(subgroup_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.parent.is_some(), Error::<T>::NotSubGroup);
            let parent_group_id = group.parent.unwrap();
            let parent_group = Self::groups(parent_group_id).ok_or(Error::<T>::GroupMissing)?;
            let return_funds_too = parent_group.anonymous_account;

            let mut admin_group = group.clone();
            let mut admin_group_id = subgroup_id;
            while caller_group_id != admin_group_id && admin_group.parent.is_some() {
                admin_group_id = admin_group.parent.unwrap();
                admin_group = Self::groups(admin_group_id).ok_or(Error::<T>::GroupMissing)?;
            }
            ensure!(caller_group_id == admin_group_id, Error::<T>::NotGroupAdmin);

            ensure!(
                <GroupChildren<T>>::get(&subgroup_id).is_none()
                    || <GroupChildren<T>>::get(&subgroup_id).unwrap().len() == 0,
                Error::<T>::GroupHasChildren
            );

            <T as Config>::Currency::transfer(
                &group.anonymous_account,
                &return_funds_too,
                <T as Config>::Currency::free_balance(&group.anonymous_account),
                AllowDeath,
            )?;

            let proposal_count = <Proposals<T>>::drain_prefix(&subgroup_id).count();
            <Groups<T>>::remove(&subgroup_id);
            <GroupChildren<T>>::remove(&subgroup_id);
            <ProposalHashes<T>>::remove(&subgroup_id);

            Self::deposit_event(Event::SubGroupRemoved(admin_group_id, subgroup_id));

            Ok((Some(T::WeightInfo::remove_sub_group(proposal_count as u32))).into())
        }

        /// Execute a proposal. Use for extrinsics that don't require voting.
        ///
        /// Requires the sender to be member.
        ///
        /// - `group_id`: Group executing the extrinsic
        /// - `proposal`: Proposal to be executed
        /// - `length_bound`: The length of the Proposal for weight estimation       
        #[pallet::weight(T::WeightInfo::execute(
            *length_bound as u32,
            T::MaxMembers::get().into()
        ).saturating_add(proposal.get_dispatch_info().weight))]
        pub fn execute(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            proposal: Box<<T as Config>::Proposal>,
            length_bound: u32,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.members.contains(&sender), Error::<T>::NotMember);
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
            ));

            Ok(Self::get_result_weight(result)
                .map(|w| {
                    T::WeightInfo::execute(proposal_len as u32, group.members.len() as u32)
                        .saturating_add(w)
                })
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
            if *threshold < 2u32.into() {
                T::WeightInfo::propose_execute(
                    *length_bound,
                    T::MaxMembers::get().into(),
                ).saturating_add(proposal.get_dispatch_info().weight)
            } else {
                T::WeightInfo::propose_proposed(
                    *length_bound,
                    T::MaxMembers::get().into(),
                    T::MaxProposals::get(),
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

            let proposals_count = proposal_hashes.len();
            ensure!(
                proposal_len <= length_bound as usize,
                Error::<T>::WrongProposalLength
            );

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

                Ok(Self::get_result_weight(result)
                    .map(|w| {
                        T::WeightInfo::propose_execute(
                            proposal_len as u32,
                            group.members.len() as u32,
                        )
                        .saturating_add(w)
                    })
                    .into())
            } else {
                <ProposalHashes<T>>::mutate_exists(group_id, |maybe_proposals| {
                    if let Some(ref mut proposals) = maybe_proposals {
                        proposals.push(proposal_hash);
                    }
                });
                // let active_proposals = proposals_length + 1;

                <Proposals<T>>::insert(group_id, proposal_id, proposal);

                let votes = Votes {
                    threshold,
                    ayes: vec![sender.clone()],
                    nays: vec![],
                };
                <Voting<T>>::insert(group_id, proposal_id, votes);

                Self::deposit_event(Event::Proposed(sender, group_id, proposal_id, threshold));

                Ok(Some(<T as Config>::WeightInfo::propose_proposed(
                    proposal_len as u32,
                    group.members.len() as u32,
                    proposals_count as u32,
                ))
                .into())
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
            ensure!(group.members.contains(&sender), Error::<T>::NotMember);
            let mut voting = Self::voting(group_id, proposal_id).ok_or(Error::<T>::VoteMissing)?;

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

            Voting::<T>::insert(group_id, proposal_id, voting);

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

        /// Close a Proposal. Caller of vote should check if the vote will be approved/disaproved and call close if it will
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
            let p2 = T::MaxProposals::get();
                T::WeightInfo::close_approved(a, m, p2)
                .max(T::WeightInfo::close_disapproved(m, p2))
                .saturating_add(p1)
        })]
        pub fn close(
            origin: OriginFor<T>,
            group_id: T::GroupId,
            proposal_id: T::ProposalId,
            proposal_weight_bound: Weight,
            length_bound: u32,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let group = Self::groups(group_id).ok_or(Error::<T>::GroupMissing)?;
            ensure!(group.members.contains(&sender), Error::<T>::NotMember);

            let voting = Self::voting(group_id, proposal_id).ok_or(Error::<T>::VoteMissing)?;

            let yes_votes = T::MemberCount::unique_saturated_from(voting.ayes.len() as u128);
            let no_votes = T::MemberCount::unique_saturated_from(voting.nays.len() as u128);

            let seats = T::MemberCount::unique_saturated_from(group.members.len() as u128);
            let approved = yes_votes >= voting.threshold;
            let disapproved = seats.saturating_sub(no_votes) < voting.threshold;

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

                let proposal_count = Self::remove_proposal(group_id, proposal_id, proposal_hash);

                let proposal_weight = Self::get_result_weight(result).unwrap_or(dispatch_weight);

                return Ok((
                    Some(
                        T::WeightInfo::close_approved(
                            proposal_len as u32,
                            seats.into(),
                            proposal_count,
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

                let proposal_count = Self::remove_proposal(group_id, proposal_id, proposal_hash);

                return Ok((
                    Some(T::WeightInfo::close_disapproved(
                        seats.into(),
                        proposal_count,
                    )),
                    Pays::No,
                )
                    .into());
            } else {
                ensure!(false, Error::<T>::VotingIncomplete);
            };
            //TODO: do we remove votes?

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
            let m = T::MaxMembers::get().into();
            let p1 = *proposal_weight_bound;
            let p2 = T::MaxProposals::get();
                T::WeightInfo::veto_approved(a, m, p2)
                .max(T::WeightInfo::veto_disapproved(m, p2))
                .saturating_add(p1)
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
            //TODO: should veto power follow group threshold?
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
            let seats = T::MemberCount::unique_saturated_from(group.members.len() as u128);

            let yes_votes = T::MemberCount::unique_saturated_from(voting.ayes.len() as u128);
            let no_votes = T::MemberCount::unique_saturated_from(voting.nays.len() as u128);

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
                let proposal_count = Self::remove_proposal(group_id, proposal_id, proposal_hash);

                let proposal_weight = Self::get_result_weight(result).unwrap_or(dispatch_weight);

                return Ok((
                    Some(
                        T::WeightInfo::veto_approved(
                            proposal_len as u32,
                            seats.into(),
                            proposal_count,
                        )
                        .saturating_add(proposal_weight),
                    ),
                    Pays::Yes,
                )
                    .into());
            } else {
                Self::deposit_event(Event::DisapprovedByVeto(
                    sender,
                    group_id,
                    proposal_id,
                    yes_votes,
                    no_votes,
                ));
                let proposal_count = Self::remove_proposal(group_id, proposal_id, proposal_hash);
                return Ok((
                    Some(T::WeightInfo::veto_disapproved(
                        seats.into(),
                        proposal_count,
                    )),
                    Pays::No,
                )
                    .into());
            };
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
        ) -> Option<Group<T::GroupId, T::AccountId, T::MemberCount, BoundedVec<u8, T::NameLimit>>>
        {
            <Groups<T>>::get(group_id)
        }
        pub fn get_sub_groups(
            group_id: T::GroupId,
        ) -> Option<
            Vec<(
                T::GroupId,
                Group<T::GroupId, T::AccountId, T::MemberCount, BoundedVec<u8, T::NameLimit>>,
            )>,
        > {
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

        pub fn get_proposals(group_id: T::GroupId) -> Vec<(T::ProposalId, T::Hash)> {
            let mut proposals = Vec::new();
            <Proposals<T>>::iter_prefix(group_id).for_each(|(proposal_id, proposal)| {
                proposals.push((proposal_id, T::Hashing::hash_of(&proposal)))
            });
            proposals
        }

        pub fn get_voting(
            group_id: T::GroupId,
            proposal_id: T::ProposalId,
        ) -> Option<Votes<T::AccountId, T::MemberCount>> {
            <Voting<T>>::get(group_id, proposal_id)
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

        fn remove_proposal(
            group_id: T::GroupId,
            proposal_id: T::ProposalId,
            proposal_hash: T::Hash,
        ) -> u32 {
            <Proposals<T>>::remove(group_id, proposal_id);
            let mut proposal_count = 0;
            <ProposalHashes<T>>::mutate_exists(group_id, |maybe_proposals| {
                if let Some(ref mut proposals) = maybe_proposals {
                    proposal_count = proposals.len() as u32;
                    proposals.retain(|h| h != &proposal_hash);
                }
            });
            proposal_count
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

    /// This just verifies that the origin came from a proposal. It does NOT do any threshold checks. The proposer specifies a threshold and that is used for voting. It is up to the recieving extrinsic to enforce a threshold.
    pub struct EnsureApproved<T>(sp_std::marker::PhantomData<T>);
    impl<
            O: Into<Result<RawOrigin<T::AccountId, T::GroupId, T::MemberCount>, O>>
                + From<RawOrigin<T::AccountId, T::GroupId, T::MemberCount>>,
            T: Config,
        > EnsureOrigin<O> for EnsureApproved<T>
    {
        type Success = (
            T::GroupId,
            Option<T::MemberCount>,
            Option<T::MemberCount>,
            T::AccountId,
        );
        fn try_origin(o: O) -> Result<Self::Success, O> {
            o.into().and_then(|o| match o {
                RawOrigin::ProposalApproved(group_id, yes_votes, no_votes, group_account) => {
                    Ok((group_id, Some(yes_votes), Some(no_votes), group_account))
                }
                RawOrigin::ProposalApprovedByVeto(group_id, _, group_account) => {
                    Ok((group_id, None, None, group_account))
                }
                r => Err(O::from(r)),
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> O {
            let group_id: T::GroupId = 1u32.into();
            let group = Groups::<T>::get(group_id).unwrap();
            O::from(RawOrigin::ProposalApprovedByVeto(
                group_id,
                T::AccountId::default(),
                group.anonymous_account.clone(),
            ))
        }
    }
    /// This just verifies that the origin came from a proposal. It does NOT do any threshold checks. The proposer specifies a threshold and that is used for voting. It is up to the recieving extrinsic to enforce a threshold.
    pub struct EnsureExecuted<T>(sp_std::marker::PhantomData<T>);
    impl<
            O: Into<Result<RawOrigin<T::AccountId, T::GroupId, T::MemberCount>, O>>
                + From<RawOrigin<T::AccountId, T::GroupId, T::MemberCount>>,
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
                r => Err(O::from(r)),
            })
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn successful_origin() -> O {
            let group_id: T::GroupId = 1u32.into();
            let group = Groups::<T>::get(group_id).unwrap();
            O::from(RawOrigin::ProposalApprovedByVeto(
                group_id,
                T::AccountId::default(),
                group.anonymous_account.clone(),
            ))
        }
    }
}
