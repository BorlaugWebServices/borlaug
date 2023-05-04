//! # Provenance Module
//!
//! ## Overview
//!
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For Process Definition creators
//! * `create_registry` - Creates a new **Registry** for organizing Definitions into collections
//! * `update_registry` - Rename a **Registry**
//! * `remove_registry` - Remove a **Registry** - **Registry** must be empty.
//! * `create_definition` - Create a new **Process Definition**
//! * `set_definition_inactive` - Set a **Process Definition** to 'inactive'.
//!                             Once a **Process Definition** is made inactive, new processes cannot be created against it.
//! * `set_definition_active` - Set a **Process Definition** to 'active'.
//!                             Once a **Process Definition** is made active, it cannot be renamed and steps cannot be added or removed.
//!                             Only attestor settings may be changed.
//! * `remove_definition` - Remove a **Process Definition**. It must not have any related Processes.
//! * `update_definition_step` - Update a step of a **Process Definition**.
//!                              You can change attestors or threshold.
//! * `update_process` - A **Process Definition** creator is allowed to rename **Processes** (attestors cannot).
//! * `remove_process` - A **Process Definition** creator is allowed to remove a **Processes** (attestors cannot).
//! * `add_child_definition` - A **Process Definition** can have child definition
//! * `remove_child_definition` - A child definition can be removed
//!
//!
//! #### For Attestors
//! * `create_process` - An attestor of the first step of a **Process Definition** may create a new Process.
//!                           A Process Step can only be updated after the previous step has been attested and before the step itself is attested.
//! * `attest_process_step` - Attest that the attributes for a process step are accurate.
//!                           This is done once for each step in order and the process moves on to the next or completes if it is the last step.
//! * `complete_process` - If there are some unrequired steps that are not attested, the attestor of the last attested step can complete the process.
//!
//! ### RPC Methods
//!
//! * `get_registries` - Get the collection of **Registries** owned by a **Process Definition** creator.
//! * `get_registry` - Get a specific **Registry**.
//! * `get_definitions` - Get the collection of **Process Definitions** in a **Registry**.
//! * `get_definition` - Get a specific **Process Definition**.
//! * `get_definition_steps` - Get the collection of steps in a **Process Definition**.
//! * `get_processes` - Get the collection of **Processes** based on a specific **Process Definition**.
//! * `get_process` - Get a specific **Process**.
//! * `get_process_steps` - Get the collection of steps of a **Process**.
//! * `get_process_step` - Get a specific step of a **Process**.
//! * `can_view_definition` - Is the account the creator of the definition or an attestor on any step.
//! * `is_attestor` - Is the account the attestor for the step. (Attestors may be an individual account or a group, check if a user is a member of the group seperately.)

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
    use primitives::*;
    use sp_runtime::{
        traits::{AtLeast32Bit, CheckedAdd, One, Saturating, UniqueSaturatedFrom},
        Either,
    };
    use sp_std::prelude::*;

    const MODULE_INDEX: u8 = 3;

    #[repr(u8)]
    pub enum ExtrinsicIndex {
        Registry = 31,
        Definition = 32,
        Process = 33,
    }

    #[derive(
        Encode, Decode, Clone, frame_support::RuntimeDebug, TypeInfo, MaxEncodedLen, PartialEq,
    )]
    pub enum Releases {
        V0,
    }
    impl Default for Releases {
        fn default() -> Self {
            Releases::V0
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + timestamp::Config + groups::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type RegistryId: Parameter
            + Member
            + AtLeast32Bit
            + Default
            + Copy
            + PartialEq
            + MaxEncodedLen;
        type DefinitionId: Parameter
            + Member
            + AtLeast32Bit
            + Default
            + Copy
            + PartialEq
            + MaxEncodedLen;
        type ProcessId: Parameter
            + Member
            + AtLeast32Bit
            + Default
            + Copy
            + PartialEq
            + MaxEncodedLen;
        type DefinitionStepIndex: Parameter
            + Member
            + AtLeast32Bit
            + Default
            + Copy
            + PartialEq
            + MaxEncodedLen;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        type GetExtrinsicExtraSource: GetExtrinsicExtra<
            ModuleIndex = u8,
            ExtrinsicIndex = u8,
            AccountId = Self::AccountId,
        >;

        /// The maximum length of a name or symbol stored on-chain.
        type NameLimit: Get<u32>;

        /// The maximum length of a name or symbol stored on-chain.
        type FactStringLimit: Get<u32>;

        /// The maximum number of allowed steps in a given definition
        type DefinitionStepLimit: Get<u32>;

        /// The maximum number of attributes allowed for a given process step
        type AttributeLimit: Get<u32>;
    }

    #[pallet::event]
    // #[pallet::metadata(
    //     T::AccountId = "AccountId",
    //     T::Moment = "Moment",
    //     T::RegistryId = "RegistryId",
    //     T::DefinitionId = "DefinitionId",
    //     T::ProcessId = "ProcessId",
    //     T::DefinitionStepIndex = "DefinitionStepIndex",
    //     T::MemberCount = "MemberCount"
    // )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new Registry was created (account_id,group_account_id,registry_id)
        RegistryCreated(T::AccountId, T::AccountId, T::RegistryId),
        /// A Registry was Updated (account_id,group_account_id,registry_id)
        RegistryUpdated(T::AccountId, T::AccountId, T::RegistryId),
        /// A Registry was Removed (account_id,group_account_id,registry_id)
        RegistryRemoved(T::AccountId, T::AccountId, T::RegistryId),
        /// A new Definition was created (account_id,group_account_id,registry_id, definition_id)
        DefinitionCreated(T::AccountId, T::AccountId, T::RegistryId, T::DefinitionId),

        /// A Definition was updated to 'active' state (account_id,group_account_id,registry_id, definition_id)
        DefinitionSetActive(T::AccountId, T::AccountId, T::RegistryId, T::DefinitionId),
        /// A Definition was updated to 'inactive' state (account_id,group_account_id,registry_id, definition_id)
        DefinitionSetInactive(T::AccountId, T::AccountId, T::RegistryId, T::DefinitionId),
        /// A Definition was Removed (account_id,group_account_id,registry_id, definition_id)
        DefinitionRemoved(T::AccountId, T::AccountId, T::RegistryId, T::DefinitionId),

        /// A DefinitionStep was Updated (account_id,group_account_id,registry_id, definition_id, definition_step_index)
        DefinitionStepUpdated(
            T::AccountId,
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::DefinitionStepIndex,
        ),

        /// A new Process was created (account_id,group_account_id,registry_id, definition_id, process_id)
        ProcessCreated(
            T::AccountId,
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::ProcessId,
        ),
        /// A Process was Removed (account_id,group_account_id,registry_id, definition_id, process_id)
        ProcessUpdated(
            T::AccountId,
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::ProcessId,
        ),
        /// A Process was Removed (account_id,group_account_id,registry_id, definition_id, process_id)
        ProcessRemoved(
            T::AccountId,
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::ProcessId,
        ),
        /// A Definition was made a child of another definition (account_id,group_account_id,registry_id, definition_id, child_registry_id, child_definition_id)
        ChildDefinitionAdded(
            T::AccountId,
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::RegistryId,
            T::DefinitionId,
        ),
        /// A Definition was was removed from the child definitions of another definition (account_id,group_account_id,registry_id, definition_id, child_registry_id, child_definition_id)
        ChildDefinitionRemoved(
            T::AccountId,
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::RegistryId,
            T::DefinitionId,
        ),

        /// A  ProcessStep was updated (account_id,group_account_id,registry_id, definition_id, process_id, definition_step_index)
        ProcessStepUpdated(
            T::AccountId,
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::ProcessId,
            T::DefinitionStepIndex,
        ),
        /// A new ProcessStep was attested (account_id,registry_id, definition_id, process_id, definition_step_index)
        ProcessStepAttested(
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::ProcessId,
            T::DefinitionStepIndex,
        ),
        /// A new Process was completed (account_id,registry_id, definition_id, process_id)
        ProcessCompleted(T::AccountId, T::RegistryId, T::DefinitionId, T::ProcessId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value was None
        NoneValue,
        /// Not authorized
        NotAuthorized,
        /// A definition must have at least one step.
        DefinitionStepsRequired,
        /// A registry must be empty before you remove it.
        RegistryNotEmpty,
        /// All processes must be removed before removing a definition.
        ProcessesExist,
        /// A string exceeds the maximum allowed length
        StringLengthLimitExceeded,
        /// Incorrect Status
        IncorrectStatus,
        /// User tried to attest a step that is already attested
        ProcessStepAlreadyAttested,
        /// No id was found (either user is not owner, or entity does not exist)
        NotFound,
        /// Cannot delete non-empty registry
        NotEmpty,
        /// Process has parent processes or child processes
        HasRelatedDefinitions,
        /// Is not an attestor for the necessary definition step
        NotAttestor,
        /// A process cannot be completed until all the required steps are attested
        NotAllRequiredStepsAttested,
        /// There should be at least one step unattested when trying to complete a process
        NoUnattestedSteps,
        /// There weren't the expected number of yes votes to match the required threshold
        IncorrectThreshold,
        /// Id out of bounds
        NoIdAvailable,
        /// A definition can only be set active if all steps have attestor groups assigned
        AttestorNotSet,
    }

    #[pallet::type_value]
    pub fn UnitDefault<T: Config>() -> u64 {
        1u64
    }

    #[pallet::type_value]
    pub fn RegistryIdDefault<T: Config>() -> T::RegistryId {
        1u32.into()
    }
    #[pallet::type_value]
    pub fn DefinitionIdDefault<T: Config>() -> T::DefinitionId {
        1u32.into()
    }
    #[pallet::type_value]
    pub fn ProcessIdDefault<T: Config>() -> T::ProcessId {
        1u32.into()
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // fn on_runtime_upgrade() -> frame_support::weights::Weight {
        //     let mut weight: Weight = 0;
        //     weight += super::migration::migrate_to_v3::<T>();
        //     weight += super::migration::migrate_to_v4::<T>();
        //     weight
        // }
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
            <StorageVersion<T>>::put(Releases::V0);
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    /// Storage version of the pallet.
    ///
    /// V2 - added proposal_id to observation struct
    pub type StorageVersion<T> = StorageValue<_, Releases, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn registries)]
    /// A group can have multiple Regitries of process definitions
    /// (group_id,registry_id) => Registry
    pub(super) type Registries<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::RegistryId,
        Registry<BoundedVec<u8, <T as Config>::NameLimit>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn definitions)]
    /// A Registry can have multiple process Definitions
    /// (T::RegistryId,T::DefinitionId) => Definition
    pub(super) type Definitions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::RegistryId,
        Blake2_128Concat,
        T::DefinitionId,
        Definition<BoundedVec<u8, <T as Config>::NameLimit>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn definition_steps)]
    /// A Process has multiple steps
    /// (T::RegistryId,T::DefinitionId), u8 => DefinitionStep
    pub(super) type DefinitionSteps<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::RegistryId, T::DefinitionId),
        Blake2_128Concat,
        T::DefinitionStepIndex,
        DefinitionStep<T::AccountId, T::MemberCount, BoundedVec<u8, <T as Config>::NameLimit>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn definition_steps_by_attestor)]
    /// A Process has multiple steps
    /// (T::RegistryId,T::DefinitionId), u8 => DefinitionStep
    pub(super) type DefinitionStepsByAttestor<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        (T::RegistryId, T::DefinitionId, T::DefinitionStepIndex),
        (),
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn processes)]
    /// A process Definition can have multiple process Processes
    /// (T::RegistryId,T::DefinitionId), T::ProcessId => T::ProcessId
    pub(super) type Processes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::RegistryId, T::DefinitionId),
        Blake2_128Concat,
        T::ProcessId,
        Process<BoundedVec<u8, <T as Config>::NameLimit>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn definition_children)]
    /// A Definition can have multiple child Definitions
    /// (T::RegistryId,T::DefinitionId), T::ProcessId => T::RegistryId
    pub(super) type DefinitionChildren<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::RegistryId, T::DefinitionId),
        Blake2_128Concat,
        T::DefinitionId,
        T::RegistryId,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn definition_parents)]
    /// A Definition may be the child of multiple definitions
    /// (T::RegistryId,T::DefinitionId), T::DefinitionId => T::RegistryId
    pub(super) type DefinitionParents<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::RegistryId, T::DefinitionId),
        Blake2_128Concat,
        T::DefinitionId,
        T::RegistryId,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn process_steps)]
    /// A Process can have multiple process Process Steps
    /// (T::RegistryId,T::DefinitionId,T::ProcessId), DefinitionStepIndex => ProcessStep
    pub(super) type ProcessSteps<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::RegistryId, T::DefinitionId, T::ProcessId),
        Blake2_128Concat,
        T::DefinitionStepIndex,
        ProcessStep<
            T::ProposalId,
            T::Moment,
            BoundedVec<u8, <T as Config>::NameLimit>,
            BoundedVec<u8, <T as Config>::FactStringLimit>,
        >,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_registry_id)]
    /// The next available registry index
    pub(super) type NextRegistryId<T: Config> =
        StorageValue<_, T::RegistryId, ValueQuery, RegistryIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_definition_id)]
    /// The next available definition index
    pub(super) type NextDefinitionId<T: Config> =
        StorageValue<_, T::DefinitionId, ValueQuery, DefinitionIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_process_id)]
    /// The next available process index
    pub(super) type NextProcessId<T: Config> =
        StorageValue<_, T::ProcessId, ValueQuery, ProcessIdDefault<T>>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add a new registry
        ///
        /// Arguments:
        /// - `name` name of the registry
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::create_registry(
            name.len() as u32
        ))]
        pub fn create_registry(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            let bounded_name = enforce_limit!(name);

            <T as Config>::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Registry as u8),
                &group_account,
            );

            let registry_id = next_id!(NextRegistryId<T>, T);

            <Registries<T>>::insert(
                &group_account,
                &registry_id,
                Registry { name: bounded_name },
            );

            Self::deposit_event(Event::RegistryCreated(
                account_id,
                group_account,
                registry_id,
            ));

            Ok(().into())
        }

        /// Update the registry
        ///
        /// Arguments:
        /// - `name` new name of the registry
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::update_registry(
            name.len() as u32
        ))]
        pub fn update_registry(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Registries<T>>::contains_key(&group_account, registry_id),
                Error::<T>::NotAuthorized
            );

            let bounded_name = enforce_limit!(name);

            <Registries<T>>::mutate_exists(&group_account, &registry_id, |maybe_registry| {
                if let Some(ref mut registry) = maybe_registry {
                    registry.name = bounded_name;
                }
            });

            Self::deposit_event(Event::RegistryUpdated(
                account_id,
                group_account,
                registry_id,
            ));
            Ok(().into())
        }

        /// Remove a registry. Must not contain any definitions.
        ///
        /// Arguments:
        /// - `registry_id` Registry to be removed
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_registry())]
        pub fn remove_registry(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Registries<T>>::contains_key(&group_account, registry_id),
                Error::<T>::NotAuthorized
            );

            ensure!(
                <Definitions<T>>::iter_prefix(registry_id).next().is_none(),
                Error::<T>::RegistryNotEmpty
            );

            <Registries<T>>::remove(&group_account, registry_id);

            Self::deposit_event(Event::RegistryRemoved(
                account_id,
                group_account,
                registry_id,
            ));
            Ok(().into())
        }

        /// Add a new definition
        ///
        /// Arguments:
        /// - `registry_id` Registry to put definition in
        /// - `name` name of the definition
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::create_definition(
            name.len() as u32,
            steps.len() as u32,
            get_max_step_name::<T::AccountId,T::MemberCount>(steps) as u32
        ))]
        pub fn create_definition(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            name: Vec<u8>,
            steps: Vec<(Vec<u8>, T::AccountId, bool, T::MemberCount)>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Registries<T>>::contains_key(&group_account, registry_id),
                Error::<T>::NotAuthorized
            );

            ensure!(!steps.is_empty(), Error::<T>::DefinitionStepsRequired);

            let bounded_name = enforce_limit!(name);

            <T as Config>::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Definition as u8),
                &group_account,
            );

            let mut new_steps = vec![];
            for (name, attestor, required, threshold) in steps {
                let bounded_name = enforce_limit!(name);
                new_steps.push((bounded_name, attestor, required, threshold));
            }

            let definition_id = next_id!(NextDefinitionId<T>, T);

            let definition = Definition {
                name: bounded_name,
                status: DefinitionStatus::Active,
            };

            <Definitions<T>>::insert(registry_id, definition_id, definition);

            new_steps.into_iter().enumerate().for_each(
                |(index, (name, attestor, required, threshold))| {
                    let definition_step = DefinitionStep {
                        name,
                        attestor: attestor.clone(),
                        required,
                        threshold,
                    };

                    let definition_step_index =
                        T::DefinitionStepIndex::unique_saturated_from(index);

                    <DefinitionSteps<T>>::insert(
                        (registry_id, definition_id),
                        definition_step_index,
                        definition_step,
                    );

                    <DefinitionStepsByAttestor<T>>::insert(
                        attestor,
                        (registry_id, definition_id, definition_step_index),
                        (),
                    );
                },
            );

            Self::deposit_event(Event::DefinitionCreated(
                account_id,
                group_account,
                registry_id,
                definition_id,
            ));
            Ok(().into())
        }

        /// Set definition active
        ///
        /// Arguments:
        /// - `registry_id` Registry the definition is in
        /// - `definition_id` Definition to set active
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::set_definition_active())]
        pub fn set_definition_active(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Registries<T>>::contains_key(&group_account, registry_id),
                Error::<T>::NotAuthorized
            );

            <Definitions<T>>::mutate_exists(registry_id, definition_id, |maybe_definition| {
                if let Some(ref mut definition) = maybe_definition {
                    definition.status = DefinitionStatus::Active;
                }
            });

            Self::deposit_event(Event::DefinitionSetActive(
                account_id,
                group_account,
                registry_id,
                definition_id,
            ));
            Ok(().into())
        }
        /// Set definition inactive
        ///
        /// Arguments:
        /// - `registry_id` Registry the definition is in
        /// - `definition_id` Definition to set active
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::set_definition_inactive())]
        pub fn set_definition_inactive(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Registries<T>>::contains_key(&group_account, registry_id),
                Error::<T>::NotAuthorized
            );

            <Definitions<T>>::mutate_exists(registry_id, definition_id, |maybe_definition| {
                if let Some(ref mut definition) = maybe_definition {
                    definition.status = DefinitionStatus::Inactive;
                }
            });

            Self::deposit_event(Event::DefinitionSetInactive(
                account_id,
                group_account,
                registry_id,
                definition_id,
            ));
            Ok(().into())
        }

        /// Remove a definition - all processes must be removed first
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` Definition to be removed
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_definition(<T as Config>::DefinitionStepLimit::get()))]
        pub fn remove_definition(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Registries<T>>::contains_key(&group_account, registry_id),
                Error::<T>::NotAuthorized
            );
            ensure!(
                <Definitions<T>>::contains_key(registry_id, definition_id),
                Error::<T>::NotFound
            );

            ensure!(
                <Processes<T>>::iter_prefix((registry_id, definition_id))
                    .next()
                    .is_none(),
                Error::<T>::ProcessesExist
            );

            ensure!(
                <DefinitionChildren<T>>::iter_prefix((registry_id, definition_id))
                    .next()
                    .is_none(),
                Error::<T>::HasRelatedDefinitions
            );
            ensure!(
                <DefinitionParents<T>>::iter_prefix((registry_id, definition_id))
                    .next()
                    .is_none(),
                Error::<T>::HasRelatedDefinitions
            );

            let step_count =
                <DefinitionSteps<T>>::drain_prefix((registry_id, definition_id)).count() as u32;

            <Definitions<T>>::remove(registry_id, definition_id);

            Self::deposit_event(Event::DefinitionRemoved(
                account_id,
                group_account,
                registry_id,
                definition_id,
            ));
            Ok(Some(<T as Config>::WeightInfo::remove_definition(step_count)).into())
        }

        /// Update a definition_step
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` the Definition
        /// - `definition_step_index` index of definition step to be updated
        /// - `attestor` Attestor for the step.
        /// - `threshold` Required threshold if Attestor is a group account else set to 1
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::update_definition_step(   ))]
        pub fn update_definition_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            definition_step_index: T::DefinitionStepIndex,
            attestor: Option<T::AccountId>,
            threshold: Option<T::MemberCount>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Registries<T>>::contains_key(&group_account, registry_id),
                Error::<T>::NotAuthorized
            );
            let definition_maybe = <Definitions<T>>::get(registry_id, definition_id);
            ensure!(definition_maybe.is_some(), Error::<T>::NotFound);

            ensure!(
                <DefinitionSteps<T>>::contains_key(
                    (registry_id, definition_id),
                    definition_step_index
                ),
                Error::<T>::NotFound
            );

            <DefinitionSteps<T>>::mutate_exists(
                (registry_id, definition_id),
                definition_step_index,
                |maybe_definition_step| {
                    if let Some(ref mut definition_step) = maybe_definition_step {
                        if let Some(attestor) = &attestor {
                            <DefinitionStepsByAttestor<T>>::remove(
                                &definition_step.attestor,
                                (registry_id, definition_id, definition_step_index),
                            );

                            definition_step.attestor = attestor.clone();

                            <DefinitionStepsByAttestor<T>>::insert(
                                attestor,
                                (registry_id, definition_id, definition_step_index),
                                (),
                            );
                        }
                        if let Some(threshold) = threshold {
                            definition_step.threshold = threshold;
                        }
                    }
                },
            );

            Self::deposit_event(Event::DefinitionStepUpdated(
                account_id,
                group_account,
                registry_id,
                definition_id,
                definition_step_index,
            ));
            Ok(().into())
        }

        /// Add a new process - any attestor on the first step can create a new process  (no voting required)
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` Definition the process is related to
        /// - `name` name of the Process
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::create_process(
            name.len() as u32
        ))]
        pub fn create_process(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            let bounded_name = enforce_limit!(name);

            let definition = <Definitions<T>>::get(registry_id, definition_id);
            ensure!(definition.is_some(), Error::<T>::NotFound);

            ensure!(
                definition.unwrap().status == DefinitionStatus::Active,
                Error::<T>::IncorrectStatus
            );

            let definition_step = <DefinitionSteps<T>>::get(
                (registry_id, definition_id),
                T::DefinitionStepIndex::unique_saturated_from(0u32),
            )
            .ok_or(Error::<T>::NotFound)?;

            ensure!(
                definition_step.attestor == group_account,
                Error::<T>::NotAttestor
            );

            <T as Config>::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Process as u8),
                &group_account,
            );

            let process_id = next_id!(NextProcessId<T>, T);

            let process = Process {
                name: bounded_name,
                status: ProcessStatus::InProgress,
            };

            <Processes<T>>::insert((registry_id, definition_id), process_id, process);

            Self::deposit_event(Event::ProcessCreated(
                account_id,
                group_account,
                registry_id,
                definition_id,
                process_id,
            ));
            Ok(().into())
        }

        /// Update a process - definition owner can rename a process
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` Definition the process is related to
        /// - `process_id` Process to be renamed
        /// - `name` name of the Process
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::update_process(
            name.len()   as u32
        ))]
        pub fn update_process(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            let bounded_name = enforce_limit!(name);

            ensure!(
                <Registries<T>>::contains_key(&group_account, registry_id),
                Error::<T>::NotAuthorized
            );

            ensure!(
                <Processes<T>>::contains_key((registry_id, definition_id), process_id),
                Error::<T>::NotFound
            );

            <Processes<T>>::mutate_exists(
                (registry_id, definition_id),
                process_id,
                |maybe_process| {
                    if let Some(ref mut process) = maybe_process {
                        process.name = bounded_name;
                    }
                },
            );

            Self::deposit_event(Event::ProcessUpdated(
                account_id,
                group_account,
                registry_id,
                definition_id,
                process_id,
            ));
            Ok(().into())
        }

        /// Remove a process - can only be done by the registry owner (admin)
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` Definition the process is related to
        /// - `process_id` Process to be removed
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_process(
            <T as Config>::DefinitionStepLimit::get()
        ))]
        pub fn remove_process(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Registries<T>>::contains_key(&group_account, registry_id),
                Error::<T>::NotAuthorized
            );
            ensure!(
                <Processes<T>>::contains_key((registry_id, definition_id), process_id),
                Error::<T>::NotFound
            );

            <Processes<T>>::remove((registry_id, definition_id), process_id);
            let step_count =
                <ProcessSteps<T>>::drain_prefix((registry_id, definition_id, process_id)).count()
                    as u32;

            Self::deposit_event(Event::ProcessRemoved(
                account_id,
                group_account,
                registry_id,
                definition_id,
                process_id,
            ));
            Ok(Some(<T as Config>::WeightInfo::remove_process(step_count)).into())
        }

        /// Add Child definition
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` the Definition
        /// - `child_definition_registry_id` Registry of the child Definition
        /// - `child_definition_id` the child Definition
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::add_child_definition())]
        pub fn add_child_definition(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            child_definition_registry_id: T::RegistryId,
            child_definition_id: T::DefinitionId,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Registries<T>>::contains_key(&group_account, registry_id),
                Error::<T>::NotAuthorized
            );
            ensure!(
                <Registries<T>>::contains_key(&group_account, child_definition_registry_id),
                Error::<T>::NotAuthorized
            );
            ensure!(
                <Definitions<T>>::contains_key(registry_id, definition_id),
                Error::<T>::NotFound
            );
            ensure!(
                <Definitions<T>>::contains_key(child_definition_registry_id, child_definition_id),
                Error::<T>::NotFound
            );

            <DefinitionChildren<T>>::insert(
                (registry_id, definition_id),
                child_definition_id,
                child_definition_registry_id,
            );
            <DefinitionParents<T>>::insert(
                (child_definition_registry_id, child_definition_id),
                definition_id,
                registry_id,
            );

            Self::deposit_event(Event::ChildDefinitionAdded(
                account_id,
                group_account,
                registry_id,
                definition_id,
                child_definition_registry_id,
                child_definition_id,
            ));
            Ok(().into())
        }

        /// Remove Child definition
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` the Definition
        /// - `child_definition_registry_id` Registry the Child Definition is in
        /// - `child_definition_id` child Definition to be removed
        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_child_definition())]
        pub fn remove_child_definition(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            child_definition_registry_id: T::RegistryId,
            child_definition_id: T::DefinitionId,
        ) -> DispatchResultWithPostInfo {
            let (account_id, group_account) = ensure_account_or_executed!(origin);

            ensure!(
                <Registries<T>>::contains_key(&group_account, registry_id),
                Error::<T>::NotAuthorized
            );
            ensure!(
                <Registries<T>>::contains_key(&group_account, child_definition_registry_id),
                Error::<T>::NotAuthorized
            );

            <DefinitionChildren<T>>::remove((registry_id, definition_id), child_definition_id);
            <DefinitionParents<T>>::remove(
                (child_definition_registry_id, child_definition_id),
                definition_id,
            );

            Self::deposit_event(Event::ChildDefinitionRemoved(
                account_id,
                group_account,
                registry_id,
                definition_id,
                child_definition_registry_id,
                child_definition_id,
            ));
            Ok(().into())
        }

        /// Attest a process_step - attestors on the step must propose and vote up to the required threshold
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` Definition the process is related to
        /// - `process_id` the Process
        /// - `definition_step_index` index of step to be attested
        /// - `attributes` attributes for the step
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::attest_process_step(
            attributes.len() as u32,
            get_max_attribute_name_len(attributes),
            get_max_attribute_fact_len(attributes),

        ))]
        pub fn attest_process_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
            definition_step_index: T::DefinitionStepIndex,
            attributes: Vec<Attribute<Vec<u8>, Vec<u8>>>,
        ) -> DispatchResultWithPostInfo {
            //TODO: use macro
            let either = T::GroupsOriginAccountOrApproved::ensure_origin(origin)?;
            let (sender, yes_votes, proposal_id) = match either {
                Either::Left(account_id) => (account_id, None, None),
                Either::Right((_, proposal_id, yes_votes, _, group_account)) => {
                    (group_account, yes_votes, Some(proposal_id))
                }
            };

            let definition_step =
                <DefinitionSteps<T>>::get((registry_id, definition_id), definition_step_index)
                    .ok_or(Error::<T>::NotFound)?;

            ensure!(definition_step.attestor == sender, Error::<T>::NotAttestor);

            ensure!(
                yes_votes.is_none() || yes_votes.unwrap() >= definition_step.threshold,
                Error::<T>::IncorrectThreshold
            );

            ensure!(
                !<ProcessSteps<T>>::contains_key(
                    (registry_id, definition_id, process_id),
                    definition_step_index
                ),
                Error::<T>::ProcessStepAlreadyAttested
            );

            let attributes = enforce_limit_attributes!(attributes);

            let process_step = ProcessStep {
                proposal_id,
                attested: <timestamp::Pallet<T>>::get(),
                attributes,
            };

            <ProcessSteps<T>>::insert(
                (registry_id, definition_id, process_id),
                definition_step_index,
                process_step,
            );

            let mut completed = true;
            let mut index = T::DefinitionStepIndex::unique_saturated_from(0u32);
            loop {
                if !<ProcessSteps<T>>::contains_key((registry_id, definition_id, process_id), index)
                {
                    completed = false;
                }
                index = index.saturating_add(T::DefinitionStepIndex::unique_saturated_from(1u32));
                if !completed
                    || !<DefinitionSteps<T>>::contains_key((registry_id, definition_id), index)
                {
                    break;
                }
            }

            if completed {
                <Processes<T>>::mutate_exists(
                    (registry_id, definition_id),
                    process_id,
                    |maybe_process| {
                        if let Some(ref mut process) = maybe_process {
                            process.status = ProcessStatus::Completed;
                        }
                    },
                );

                Self::deposit_event(Event::ProcessCompleted(
                    sender.clone(),
                    registry_id,
                    definition_id,
                    process_id,
                ));
            }

            Self::deposit_event(Event::ProcessStepAttested(
                sender,
                registry_id,
                definition_id,
                process_id,
                definition_step_index,
            ));

            Ok(().into())
        }

        /// Complete process - this must be used if there are some not required steps that have not been attested.
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` Definition the process is related to
        /// - `process_id` the Process
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::complete_process(<T as Config>::DefinitionStepLimit::get()))]
        pub fn complete_process(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
        ) -> DispatchResultWithPostInfo {
            let either = T::GroupsOriginAccountOrApproved::ensure_origin(origin)?;
            let (sender, yes_votes, _proposal_id) = match either {
                Either::Left(account_id) => (account_id, None, None),
                Either::Right((_, proposal_id, yes_votes, _, group_account)) => {
                    (group_account, yes_votes, Some(proposal_id))
                }
            };

            //TODO: is it OK do do all these db reads even if extrinsic is rejected?

            let step_count =
                <DefinitionSteps<T>>::iter_prefix((registry_id, definition_id)).count() as u32;

            let mut last_attested_step_index = None;
            let mut can_complete = true;

            for i in 0..step_count {
                let index = T::DefinitionStepIndex::unique_saturated_from(i);
                let step = <ProcessSteps<T>>::get((registry_id, definition_id, process_id), index);
                if step.is_some() {
                    last_attested_step_index = Some(index);
                } else {
                    let definition_step =
                        <DefinitionSteps<T>>::get((registry_id, definition_id), index);
                    if let Some(definition_step) = definition_step {
                        if definition_step.required {
                            can_complete = false;
                        }
                    } else {
                        can_complete = false;
                    }
                }
            }

            ensure!(can_complete, Error::<T>::NotAllRequiredStepsAttested);

            ensure!(
                last_attested_step_index.is_some(),
                Error::<T>::NoUnattestedSteps
            );

            if let Some(last_attested_step_index) = last_attested_step_index {
                let definition_step = <DefinitionSteps<T>>::get(
                    (registry_id, definition_id),
                    last_attested_step_index,
                );

                ensure!(
                    definition_step.is_some()
                        && definition_step.as_ref().unwrap().attestor == sender,
                    Error::<T>::NotAttestor
                );

                ensure!(
                    yes_votes.is_none() || yes_votes.unwrap() >= definition_step.unwrap().threshold,
                    Error::<T>::IncorrectThreshold
                );
            }

            <Processes<T>>::mutate_exists(
                (registry_id, definition_id),
                process_id,
                |maybe_process| {
                    if let Some(ref mut process) = maybe_process {
                        process.status = ProcessStatus::Completed;
                    }
                },
            );

            Self::deposit_event(Event::ProcessCompleted(
                sender.clone(),
                registry_id,
                definition_id,
                process_id,
            ));

            Ok(Some(<T as Config>::WeightInfo::complete_process(step_count)).into())
        }
    }

    impl<T: Config> Pallet<T> {
        // -- rpc api functions --
        pub fn get_registries(
            account_id: T::AccountId,
        ) -> Vec<(
            T::RegistryId,
            Registry<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut registries = Vec::new();

            <Registries<T>>::iter_prefix(account_id)
                .for_each(|(registry_id, registry)| registries.push((registry_id, registry)));

            registries
        }
        pub fn get_registry(
            account_id: T::AccountId,
            registry_id: T::RegistryId,
        ) -> Option<Registry<BoundedVec<u8, <T as Config>::NameLimit>>> {
            <Registries<T>>::get(account_id, registry_id)
        }

        pub fn get_definitions(
            registry_id: T::RegistryId,
        ) -> Vec<(
            T::DefinitionId,
            Definition<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut definitions = Vec::new();

            <Definitions<T>>::iter_prefix(registry_id).for_each(|(definition_id, definition)| {
                definitions.push((definition_id, definition))
            });

            definitions
        }
        pub fn get_definition(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> Option<Definition<BoundedVec<u8, <T as Config>::NameLimit>>> {
            <Definitions<T>>::get(registry_id, definition_id)
        }

        pub fn get_definition_step(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            step_index: T::DefinitionStepIndex,
        ) -> Option<
            DefinitionStep<T::AccountId, T::MemberCount, BoundedVec<u8, <T as Config>::NameLimit>>,
        > {
            <DefinitionSteps<T>>::get((registry_id, definition_id), step_index)
        }

        pub fn get_definition_steps(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> Vec<(
            T::DefinitionStepIndex,
            DefinitionStep<T::AccountId, T::MemberCount, BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut definition_steps = Vec::new();
            <DefinitionSteps<T>>::iter_prefix((registry_id, definition_id)).for_each(
                |(step_index, definition_step)| {
                    definition_steps.push((step_index, definition_step))
                },
            );
            definition_steps.sort_by(|(step_index_a, _), (step_index_b, _)| {
                step_index_a.partial_cmp(step_index_b).unwrap()
            });

            definition_steps
        }

        pub fn get_available_definitions(
            account_id: T::AccountId,
        ) -> Vec<(
            T::RegistryId,
            T::DefinitionId,
            Definition<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut definitions = Vec::new();
            <DefinitionStepsByAttestor<T>>::iter_prefix(account_id).for_each(
                |((registry_id, definition_id, step_index), _)| {
                    if step_index == T::DefinitionStepIndex::unique_saturated_from(0u32) {
                        let maybe_definition = <Definitions<T>>::get(registry_id, definition_id);
                        if let Some(definition) = maybe_definition {
                            if !definitions.iter().any(|(r_id, d_id, _)| {
                                *r_id == registry_id && *d_id == definition_id
                            }) {
                                definitions.push((registry_id, definition_id, definition));
                            }
                        }
                    }
                },
            );
            definitions
        }

        pub fn get_processes(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> Vec<(
            T::ProcessId,
            Process<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut processes = Vec::new();

            <Processes<T>>::iter_prefix((registry_id, definition_id))
                .for_each(|(process_id, process)| processes.push((process_id, process)));

            processes
        }
        pub fn get_process(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
        ) -> Option<Process<BoundedVec<u8, <T as Config>::NameLimit>>> {
            <Processes<T>>::get((registry_id, definition_id), process_id)
        }
        /// Processes where an account is the attestor on at least one step and the specified status
        pub fn get_processes_for_attestor_by_status(
            account_id: T::AccountId,
            status: ProcessStatus,
        ) -> Vec<(
            T::RegistryId,
            T::DefinitionId,
            T::ProcessId,
            Process<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut definitions = Vec::new();
            <DefinitionStepsByAttestor<T>>::iter_prefix(account_id).for_each(
                |((registry_id, definition_id, _), _)| {
                    if !definitions
                        .iter()
                        .any(|(r_id, d_id)| *r_id == registry_id && *d_id == definition_id)
                    {
                        definitions.push((registry_id, definition_id));
                    }
                },
            );

            let mut processes = Vec::new();

            definitions.iter().for_each(|(registry_id, definition_id)| {
                <Processes<T>>::iter_prefix((registry_id, definition_id)).for_each(
                    |(process_id, process)| {
                        if process.status == status {
                            processes.push((*registry_id, *definition_id, process_id, process))
                        }
                    },
                );
            });

            processes
        }

        /// Processes where an account is the attestor on the step that is pending
        pub fn get_processes_for_attestor_pending(
            account_id: T::AccountId,
        ) -> Vec<(
            T::RegistryId,
            T::DefinitionId,
            T::ProcessId,
            Process<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut definition_steps = Vec::new();
            <DefinitionStepsByAttestor<T>>::iter_prefix(account_id).for_each(
                |((registry_id, definition_id, step_index), _)| {
                    definition_steps.push((registry_id, definition_id, step_index));
                },
            );

            let mut processes = Vec::new();

            definition_steps
                .iter()
                .for_each(|(registry_id, definition_id, step_index)| {
                    <Processes<T>>::iter_prefix((registry_id, definition_id)).for_each(
                        |(process_id, process)| {
                            #[allow(clippy::collapsible_if)]
                            if process.status == ProcessStatus::InProgress {
                                if !<ProcessSteps<T>>::contains_key(
                                    (registry_id, definition_id, process_id),
                                    step_index,
                                ) && (*step_index
                                    == T::DefinitionStepIndex::unique_saturated_from(0u32)
                                    || <ProcessSteps<T>>::contains_key(
                                        (registry_id, definition_id, process_id),
                                        step_index.saturating_sub(
                                            T::DefinitionStepIndex::unique_saturated_from(1u32),
                                        ),
                                    ))
                                {
                                    processes.push((
                                        *registry_id,
                                        *definition_id,
                                        process_id,
                                        process,
                                    ))
                                }
                            }
                        },
                    );
                });

            processes
        }

        pub fn get_process_steps(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
        ) -> Vec<(
            T::DefinitionStepIndex,
            ProcessStep<
                T::ProposalId,
                T::Moment,
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
        )> {
            let mut process_steps = Vec::new();
            <ProcessSteps<T>>::iter_prefix((registry_id, definition_id, process_id)).for_each(
                |(step_index, definition_step)| process_steps.push((step_index, definition_step)),
            );
            process_steps
        }

        pub fn get_process_step(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
            definition_step_index: T::DefinitionStepIndex,
        ) -> Option<(
            T::DefinitionStepIndex,
            ProcessStep<
                T::ProposalId,
                T::Moment,
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
        )> {
            <ProcessSteps<T>>::get(
                (registry_id, definition_id, process_id),
                definition_step_index,
            )
            .map(|process_step| (definition_step_index, process_step))
        }

        pub fn get_definition_children(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> Vec<(
            T::DefinitionId,
            Definition<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            <DefinitionChildren<T>>::iter_prefix((registry_id, definition_id))
                .filter_map(|(child_definition_id, child_registry_id)| {
                    <Definitions<T>>::get(child_registry_id, child_definition_id)
                        .map(|child_definition| (child_definition_id, child_definition))
                })
                .collect()
        }

        pub fn get_definition_parents(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> Vec<(
            T::DefinitionId,
            Definition<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            <DefinitionParents<T>>::iter_prefix((registry_id, definition_id))
                .filter_map(|(parent_definition_id, parent_registry_id)| {
                    <Definitions<T>>::get(parent_registry_id, parent_definition_id)
                        .map(|parent_definition| (parent_definition_id, parent_definition))
                })
                .collect()
        }

        // Definition creators and attestors can view definitions and processes derived from them.
        pub fn can_view_definition(
            account_id: T::AccountId,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> bool {
            if <Registries<T>>::contains_key(&account_id, registry_id) {
                true
            } else {
                let attestor_on = <DefinitionSteps<T>>::iter_prefix((registry_id, definition_id))
                    .find(|(_definition_step_index, definition_step)| {
                        definition_step.attestor == account_id
                    });
                attestor_on.is_some()
            }
        }

        pub fn is_attestor(
            account_id: T::AccountId,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            definition_step_index: T::DefinitionStepIndex,
        ) -> bool {
            let definition_step_maybe =
                <DefinitionSteps<T>>::get((registry_id, definition_id), definition_step_index);
            definition_step_maybe.map_or(false, |definition_step| {
                definition_step.attestor == account_id
            })
        }

        // -- private functions --
    }

    // -- for use in weights --

    macro_rules! max_fact_len {
        ($fact:expr,$max_fact_len:ident) => {{
            let fact_len = match &$fact {
                Fact::Bool(..) => 1u32,
                Fact::Text(string) => string.len() as u32,
                Fact::Attachment(_hash, filename) => 32u32 + (filename.len() as u32),
                Fact::Location(..) => 2u32,
                Fact::Did(..) => 32u32,
                Fact::Float(..) => 8u32,
                Fact::U8(..) => 1u32,
                Fact::U16(..) => 2u32,
                Fact::U32(..) => 4u32,
                Fact::U128(..) => 16u32,
                Fact::Date(..) => 4u32,
                // Fact::Iso8601(..) => 17u32, //Timezone should be max 10 ?
            };
            if fact_len > $max_fact_len {
                $max_fact_len = fact_len;
            };
        }};
    }

    fn get_max_step_name<AccountId, MemberCount>(
        steps: &[(Vec<u8>, AccountId, bool, MemberCount)],
    ) -> u32 {
        let mut max_step_name_len = 0;
        steps.iter().for_each(|(name, _, _, _)| {
            if name.len() as u32 > max_step_name_len {
                max_step_name_len = name.len() as u32;
            };
        });
        max_step_name_len
    }
    fn get_max_attribute_name_len(attributes: &[Attribute<Vec<u8>, Vec<u8>>]) -> u32 {
        let mut max_attribute_name_len = 0;
        attributes.iter().for_each(|attribute| {
            if attribute.name.len() as u32 > max_attribute_name_len {
                max_attribute_name_len = attribute.name.len() as u32;
            };
        });
        max_attribute_name_len
    }

    fn get_max_attribute_fact_len(attributes: &[Attribute<Vec<u8>, Vec<u8>>]) -> u32 {
        let mut max_fact_len = 0;
        attributes.iter().for_each(|attribute| {
            max_fact_len!(attribute.fact, max_fact_len);
        });
        max_fact_len
    }
}
