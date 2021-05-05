//! # Org Module
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

    use codec::Encode;
    use frame_support::{
        dispatch::{DispatchResultWithPostInfo, Dispatchable, PostDispatchInfo},
        pallet_prelude::*,
        weights::GetDispatchInfo,
    };
    use frame_system::pallet_prelude::*;
    use primitives::org::OrgGroup;
    use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, One};
    use sp_std::prelude::*;

    /// Simple index type for proposal counting.
    pub type ProposalIndex = u32;

    /// A number of members.
    ///
    /// This also serves as a number of voting members, and since for motions, each member may
    /// vote exactly once, therefore also the number of votes for any given motion.
    pub type MemberCount = u32;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type OrgGroupId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;

        /// The outer origin type.
        type Origin: From<RawOrigin<Self::AccountId>>;
        /// The outer call dispatch type.
        type Proposal: Parameter
            + Dispatchable<Origin = <Self as Config>::Origin, PostInfo = PostDispatchInfo>
            + From<frame_system::Call<Self>>
            + GetDispatchInfo;

        /// Maximum number of proposals allowed to be active in parallel.
        type MaxProposals: Get<ProposalIndex>;

        /// The maximum number of members supported by the pallet. Used for weight estimation.
        ///
        /// NOTE:
        /// + Benchmarks will need to be re-run and weights adjusted if this changes.
        /// + This pallet assumes that dependents keep to the limit without enforcing it.
        type MaxMembers: Get<MemberCount>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    /// Origin for the collective module.
    #[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode)]
    pub enum RawOrigin<AccountId> {
        /// It has been condoned by a given number of members of the collective from a given total.
        Members(MemberCount, MemberCount),
        /// It has been condoned by a single member of the collective.
        Member(AccountId),
    }

    /// Origin for the collective module.
    pub type Origin<T> = RawOrigin<<T as frame_system::Config>::AccountId>;

    #[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
    /// Info for keeping track of a motion being voted on.
    pub struct Votes<AccountId> {
        /// The proposal's unique index.
        index: ProposalIndex,
        /// The number of approval votes that are needed to pass the motion.
        threshold: MemberCount,
        /// The current set of voters that approved it.
        ayes: Vec<AccountId>,
        /// The current set of voters that rejected it.
        nays: Vec<AccountId>,
    }
    #[pallet::event]
    #[pallet::metadata(T::Moment = "Moment", T::OrgGroupId = "OrgGroupId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new OrgGroup was created (AccountId,OrgGroupId)
        OrgGroupCreated(T::AccountId, T::OrgGroupId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account is not a member
        NotMember,
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
    pub fn OrgGroupIdDefault<T: Config>() -> T::OrgGroupId {
        1u32.into()
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_org_group_id)]
    /// The next available org_group index
    pub(super) type NextOrgGroupId<T: Config> =
        StorageValue<_, T::OrgGroupId, ValueQuery, OrgGroupIdDefault<T>>;

    /// Org groups have some properties
    /// OrgGroupId => OrgGroup
    #[pallet::storage]
    #[pallet::getter(fn org_groups)]
    pub(super) type OrgGroups<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::OrgGroupId,
        OrgGroup<T::OrgGroupId, T::AccountId>,
        ValueQuery,
    >;

    /// OrgGroups may have child groups
    /// T::OrgGroupId => Vec<OrgGroupId>
    #[pallet::storage]
    #[pallet::getter(fn groups)]
    pub(super) type OrgGroupChildren<T: Config> =
        StorageMap<_, Blake2_128Concat, T::OrgGroupId, Vec<T::OrgGroupId>, ValueQuery>;

    /// OrgGroups may have proposals awaiting approval
    /// T::OrgGroupId => Vec<T::Hash>
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub(super) type Proposals<T: Config> =
        StorageMap<_, Blake2_128Concat, T::OrgGroupId, Vec<T::Hash>, ValueQuery>;

    /// Actual proposal for a given hash, if it's current.
    /// T::OrgGroupId => Vec<T::Hash>
    #[pallet::storage]
    #[pallet::getter(fn proposal_of)]
    pub(super) type ProposalOf<T: Config> =
        StorageMap<_, Identity, T::Hash, Option<<T as Config>::Proposal>, ValueQuery>;

	/// Votes on a given proposal, if it is ongoing.
    #[pallet::storage]
    #[pallet::getter(fn voting)]
    pub(super) type Votes<T: Config> =
        StorageMap<_, Identity, T::Hash, Option<<T as Config>::Proposal>, ValueQuery>;


    pub Voting get(fn voting):
    map hasher(identity) T::Hash => Option<Votes<T::AccountId, T::BlockNumber>>;


    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new OrgGroup
        ///
        /// # <weight>
        /// - O(1).
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_org_group(
            origin: OriginFor<T>,
            org_group: OrgGroup<T::OrgGroupId, T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            //TODO:

            Ok(().into())
        }

        /// Update an OrgGroup. Parent cannot be changed
        ///
        /// # <weight>
        /// - O(1).
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_org_group(
            origin: OriginFor<T>,
            org_group: OrgGroup<T::OrgGroupId, T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            //TODO:

            Ok(().into())
        }

        /// Remove an OrgGroup. All child groups will also be removed.
        ///
        /// # <weight>
        /// - O(1).
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_org_group(
            origin: OriginFor<T>,
            org_group_id: T::OrgGroupId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            //TODO:

            Ok(().into())
        }

        /// Add a new proposal to either be voted on or executed directly.
        ///
        /// Requires the sender to be member.
        ///
        /// `threshold` determines whether `proposal` is executed directly (`threshold < 2`)
        /// or put up for voting.
        ///
        /// # <weight>
        /// ## Weight
        /// - `O(B + M + P1)` or `O(B + M + P2)` where:
        ///   - `B` is `proposal` size in bytes (length-fee-bounded)
        ///   - `M` is members-count (code- and governance-bounded)
        ///   - branching is influenced by `threshold` where:
        ///     - `P1` is proposal execution complexity (`threshold < 2`)
        ///     - `P2` is proposals-count (code-bounded) (`threshold >= 2`)
        /// - DB:
        ///   - 1 storage read `is_member` (codec `O(M)`)
        ///   - 1 storage read `ProposalOf::contains_key` (codec `O(1)`)
        ///   - DB accesses influenced by `threshold`:
        ///     - EITHER storage accesses done by `proposal` (`threshold < 2`)
        ///     - OR proposal insertion (`threshold <= 2`)
        ///       - 1 storage mutation `Proposals` (codec `O(P2)`)
        ///       - 1 storage mutation `ProposalCount` (codec `O(1)`)
        ///       - 1 storage write `ProposalOf` (codec `O(B)`)
        ///       - 1 storage write `Voting` (codec `O(M)`)
        ///   - 1 event
        /// # </weight>
        #[pallet::weight(
			if *threshold < 2 {
				T::WeightInfo::propose_execute(
					*length_bound, // B
					T::MaxMembers::get(), // M
				).saturating_add(proposal.get_dispatch_info().weight) // P1
			} else {
				T::WeightInfo::propose_proposed(
					*length_bound, // B
					T::MaxMembers::get(), // M
					T::MaxProposals::get(), // P2
				)
			}
		)]
        fn propose(
            origin: OriginFor<T>,
            #[compact] threshold: MemberCount,
            org_group_id: T::OrgGroupId,
            proposal: Box<<T as Config>::Proposal>,
            #[compact] length_bound: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let members = Self::members();
            ensure!(members.contains(&who), Error::<T>::NotMember);

            let proposal_len = proposal.using_encoded(|x| x.len());
            ensure!(
                proposal_len <= length_bound as usize,
                Error::<T>::WrongProposalLength
            );
            let proposal_hash = T::Hashing::hash_of(&proposal);
            ensure!(
                !<ProposalOf<T, I>>::contains_key(proposal_hash),
                Error::<T>::DuplicateProposal
            );

            if threshold < 2 {
                let seats = Self::members().len() as MemberCount;
                let result = proposal.dispatch(RawOrigin::Members(1, seats).into());
                Self::deposit_event(RawEvent::Executed(
                    proposal_hash,
                    result.map(|_| ()).map_err(|e| e.error),
                ));

                Ok(get_result_weight(result)
                    .map(|w| {
                        T::WeightInfo::propose_execute(
                            proposal_len as u32,  // B
                            members.len() as u32, // M
                        )
                        .saturating_add(w) // P1
                    })
                    .into())
            } else {
                let active_proposals = <Proposals<T>>::try_mutate(
                    org_group_id,
                    |proposals| -> Result<usize, DispatchError> {
                        proposals.push(proposal_hash);
                        ensure!(
                            proposals.len() <= T::MaxProposals::get() as usize,
                            Error::<T>::TooManyProposals
                        );
                        Ok(proposals.len())
                    },
                )?;
                let index = Self::proposal_count();
                <ProposalCount>::mutate(|i| *i += 1);
                <ProposalOf<T>>::insert(proposal_hash, *proposal);

                let votes = Votes {
                    index,
                    threshold,
                    ayes: vec![who.clone()],
                    nays: vec![],
                };
                <Voting<T>>::insert(proposal_hash, votes);

                Self::deposit_event(RawEvent::Proposed(who, index, proposal_hash, threshold));

                Ok(Some(T::WeightInfo::propose_proposed(
                    proposal_len as u32,     // B
                    members.len() as u32,    // M
                    active_proposals as u32, // P2
                ))
                .into())
            }
        }
    }
    impl<T: Config> Module<T> {
        // -- private functions --

        fn get_next_org_group_id() -> Result<T::OrgGroupId, Error<T>> {
            let org_group_id = <NextOrgGroupId<T>>::get();
            <NextOrgGroupId<T>>::put(
                org_group_id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoIdAvailable)?,
            );
            Ok(org_group_id)
        }
    }
}
