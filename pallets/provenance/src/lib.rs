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
//! * `update_definition` - Rename a **Process Definition**
//! * `set_definition_active` - Set a **Process Definition** to 'active'.
//!                             Once a **Process Definition** is made active, it cannot be renamed and steps cannot be added or removed.
//!                             Only attestor settings may be changed.
//! * `remove_definition` - Remove a **Process Definition**. It must not have any related Processes.
//! * `create_definition_step` - Add a step to a **Process Definition**.
//! * `update_definition_step` - Update a step of a **Process Definition**.
//!                              You can rename the step or change attestors or threshold.
//!                              Renaming is only possible before the **Process Definition** is set to 'active'.
//! * `delete_definition_step` - Delete a step of a **Process Definition**.
//!                              Deletion is only possible before the **Process Definition** is set to 'active'.
//! * `update_process` - A **Process Definition** creator is allowed to rename **Processes** (attestors cannot).
//! * `remove_process` - A **Process Definition** creator is allowed to remove a **Processes** (attestors cannot).
//!
//! #### For Attestors
//! * `create_process` - An attestor of the first step of a **Process Definition** may create a new Process.
//! * `update_process_step` - An attestor may update the attributes of a process step. All attributes are replaced on each update.
//!                           A Process Step can only be updated after the previous step has been attested and before the step itself is attested.
//! * `attest_process_step` - Attest that the attributes for a process step are accurate.
//!                           This is done once for each step in order and the process moves on to the next or completes if it is the last step.
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

    #[pallet::config]
    pub trait Config: frame_system::Config + groups::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type RegistryId: Parameter + Member + AtLeast32Bit + Default + Copy + PartialEq;
        type DefinitionId: Parameter + Member + AtLeast32Bit + Default + Copy + PartialEq;
        type ProcessId: Parameter + Member + AtLeast32Bit + Default + Copy + PartialEq;
        type DefinitionStepIndex: Parameter + Member + AtLeast32Bit + Default + Copy + PartialEq;

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
    #[pallet::metadata(
        T::AccountId = "AccountId",
        T::RegistryId = "RegistryId",
        T::DefinitionId = "DefinitionId",
        T::ProcessId = "ProcessId",
        T::DefinitionStepIndex = "DefinitionStepIndex",
        T::MemberCount = "MemberCount"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new Registry was created (account_id,registry_id)
        RegistryCreated(T::AccountId, T::RegistryId),
        /// A Registry was Updated (account_id,registry_id)
        RegistryUpdated(T::AccountId, T::RegistryId),
        /// A Registry was Removed (account_id,registry_id)
        RegistryRemoved(T::AccountId, T::RegistryId),
        /// A new Definition was created (account_id,registry_id, definition_id)
        DefinitionCreated(T::AccountId, T::RegistryId, T::DefinitionId),
        /// A new Definition was updated (account_id,registry_id, definition_id)
        DefinitionUpdated(T::AccountId, T::RegistryId, T::DefinitionId),
        /// A Definition was updated to 'active' state (account_id,registry_id, definition_id)
        DefinitionSetActive(T::AccountId, T::RegistryId, T::DefinitionId),
        /// A Definition was updated to 'inactive' state (account_id,registry_id, definition_id)
        DefinitionSetInactive(T::AccountId, T::RegistryId, T::DefinitionId),
        /// A Definition was Removed (account_id,registry_id, definition_id)
        DefinitionRemoved(T::AccountId, T::RegistryId, T::DefinitionId),
        /// A DefinitionStep was Created (account_id,registry_id, definition_id, definition_step_index)
        DefinitionStepCreated(
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::DefinitionStepIndex,
        ),
        /// A DefinitionStep was Updated (account_id,registry_id, definition_id, definition_step_index)
        DefinitionStepUpdated(
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::DefinitionStepIndex,
        ),
        /// A DefinitionStep was Removed (account_id,registry_id, definition_id, definition_step_index)
        DefinitionStepRemoved(
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::DefinitionStepIndex,
        ),
        /// A new Process was created (account_id,registry_id, definition_id, process_id)
        ProcessCreated(T::AccountId, T::RegistryId, T::DefinitionId, T::ProcessId),
        /// A Process was Removed (account_id,registry_id, definition_id, process_id)
        ProcessUpdated(T::AccountId, T::RegistryId, T::DefinitionId, T::ProcessId),
        /// A Process was Removed (account_id,registry_id, definition_id, process_id)
        ProcessRemoved(T::AccountId, T::RegistryId, T::DefinitionId, T::ProcessId),
        /// A new ProcessStep was created (account_id,registry_id, definition_id, process_id, definition_step_index)
        ProcessStepCreated(
            T::AccountId,
            T::RegistryId,
            T::DefinitionId,
            T::ProcessId,
            T::DefinitionStepIndex,
        ),
        /// A  ProcessStep was updated (account_id,registry_id, definition_id, process_id, definition_step_index)
        ProcessStepUpdated(
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
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value was None
        NoneValue,
        /// Not authorized
        NotAuthorized,
        /// A registry must be empty before you remove it.
        RegistryNotEmpty,
        /// All processes must be removed before removing a definition.
        ProcessesExist,
        /// A string exceeds the maximum allowed length
        BadString,
        /// IncorrectStatus
        IncorrectStatus,
        /// No id was found (either user is not owner, or entity does not exist)
        NotFound,
        /// Cannot delete non-empty registry
        NotEmpty,
        /// Is not an attestor for the necessary definition step
        NotAttestor,
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
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

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
        #[pallet::weight(<T as Config>::WeightInfo::create_registry(
            name.len() as u32
        ))]
        pub fn create_registry(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let bounded_name = enforce_limit!(name);

            <T as Config>::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Registry as u8),
                &sender,
            );

            let registry_id = next_id!(NextRegistryId<T>, T);

            <Registries<T>>::insert(&sender, &registry_id, Registry { name: bounded_name });

            Self::deposit_event(Event::RegistryCreated(sender, registry_id));

            Ok(().into())
        }

        /// Update the registry
        ///
        /// Arguments:
        /// - `name` new name of the registry
        #[pallet::weight(<T as Config>::WeightInfo::update_registry(
            name.len() as u32
        ))]
        pub fn update_registry(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
                Error::<T>::NotAuthorized
            );

            let bounded_name = enforce_limit!(name);

            <Registries<T>>::mutate_exists(&sender, &registry_id, |maybe_registry| {
                if let Some(ref mut registry) = maybe_registry {
                    registry.name = bounded_name;
                }
            });

            Self::deposit_event(Event::RegistryUpdated(sender, registry_id));
            Ok(().into())
        }

        /// Remove a registry. Must not contain any definitions.
        ///
        /// Arguments:
        /// - `registry_id` Registry to be removed
        #[pallet::weight(<T as Config>::WeightInfo::remove_registry())]
        pub fn remove_registry(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
                Error::<T>::NotAuthorized
            );

            ensure!(
                <Definitions<T>>::iter_prefix(registry_id).next().is_none(),
                Error::<T>::RegistryNotEmpty
            );

            <Registries<T>>::remove(&sender, registry_id);

            Self::deposit_event(Event::RegistryRemoved(sender, registry_id));
            Ok(().into())
        }

        /// Add a new definition
        ///
        /// Arguments:
        /// - `registry_id` Registry to put definition in
        /// - `name` name of the definition        
        #[pallet::weight(<T as Config>::WeightInfo::create_definition(
            name.len() as u32,
        ))]
        pub fn create_definition(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
                Error::<T>::NotAuthorized
            );

            let bounded_name = enforce_limit!(name);

            <T as Config>::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Definition as u8),
                &sender,
            );

            let definition_id = next_id!(NextDefinitionId<T>, T);

            let definition = Definition {
                name: bounded_name,
                status: DefinitionStatus::Creating,
            };

            <Definitions<T>>::insert(registry_id, definition_id, definition);

            Self::deposit_event(Event::DefinitionCreated(sender, registry_id, definition_id));
            Ok(().into())
        }

        /// Update definition. Status must be in status `Creating`
        ///
        /// Arguments:
        /// - `registry_id` Registry the definition is in
        /// - `name` new name of the definition
        #[pallet::weight(<T as Config>::WeightInfo::update_definition(
            name.len() as u32,
        ))]
        pub fn update_definition(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
                Error::<T>::NotAuthorized
            );

            let bounded_name = enforce_limit!(name);

            let definition = <Definitions<T>>::get(registry_id, definition_id);
            ensure!(definition.is_some(), Error::<T>::NotFound);
            let mut definition = definition.unwrap();
            ensure!(
                definition.status == DefinitionStatus::Creating,
                Error::<T>::IncorrectStatus
            );

            definition.name = bounded_name;
            <Definitions<T>>::insert(registry_id, definition_id, definition);

            Self::deposit_event(Event::DefinitionUpdated(sender, registry_id, definition_id));
            Ok(().into())
        }

        /// Set definition active
        ///
        /// Arguments:
        /// - `registry_id` Registry the definition is in
        /// - `definition_id` Definition to set active
        #[pallet::weight(<T as Config>::WeightInfo::set_definition_active(<T as Config>::DefinitionStepLimit::get()))]
        pub fn set_definition_active(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
                Error::<T>::NotAuthorized
            );

            let mut attestor_set = true;
            let mut step_count = 0;
            <DefinitionSteps<T>>::iter_prefix((registry_id, definition_id)).for_each(
                |(_step_index, definition_step)| {
                    step_count = step_count + 1;
                    if !definition_step.attestor.is_some() {
                        attestor_set = false;
                    }
                },
            );
            ensure!(attestor_set, Error::<T>::AttestorNotSet);

            <Definitions<T>>::mutate_exists(registry_id, definition_id, |maybe_definition| {
                if let Some(ref mut definition) = maybe_definition {
                    definition.status = DefinitionStatus::Active;
                }
            });

            Self::deposit_event(Event::DefinitionSetActive(
                sender,
                registry_id,
                definition_id,
            ));
            Ok(Some(<T as Config>::WeightInfo::set_definition_active(step_count)).into())
        }
        /// Set definition inactive
        ///
        /// Arguments:
        /// - `registry_id` Registry the definition is in
        /// - `definition_id` Definition to set active
        #[pallet::weight(<T as Config>::WeightInfo::set_definition_inactive())]
        pub fn set_definition_inactive(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
                Error::<T>::NotAuthorized
            );

            <Definitions<T>>::mutate_exists(registry_id, definition_id, |maybe_definition| {
                if let Some(ref mut definition) = maybe_definition {
                    definition.status = DefinitionStatus::Inactive;
                }
            });

            Self::deposit_event(Event::DefinitionSetInactive(
                sender,
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
        #[pallet::weight(<T as Config>::WeightInfo::remove_definition(<T as Config>::DefinitionStepLimit::get()))]
        pub fn remove_definition(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
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

            let step_count =
                <DefinitionSteps<T>>::drain_prefix((registry_id, definition_id)).count() as u32;

            <Definitions<T>>::remove(registry_id, definition_id);

            Self::deposit_event(Event::DefinitionRemoved(sender, registry_id, definition_id));
            Ok(Some(<T as Config>::WeightInfo::remove_definition(step_count)).into())
        }

        //TODO: reorder steps?

        /// Add a new definition step. Caller should ensure definition_step_index equals existing step count.
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` the Definition
        /// - `definition_step_index` New definition_step_index (should equal existing step count)
        /// - `name` name of the Definition Step
        /// - `attestor` Attestor for the step. Optional at this stage.
        /// - `threshold` Required threshold if Attestor is a group account else set to 1
        #[pallet::weight(<T as Config>::WeightInfo::create_definition_step(
            name.len() as u32,
        ))]
        pub fn create_definition_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            definition_step_index: T::DefinitionStepIndex,
            name: Vec<u8>,
            attestor: Option<T::AccountId>,
            threshold: T::MemberCount,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
                Error::<T>::NotAuthorized
            );
            let definition_maybe = <Definitions<T>>::get(registry_id, definition_id);
            ensure!(definition_maybe.is_some(), Error::<T>::NotFound);
            ensure!(
                definition_maybe.unwrap().status == DefinitionStatus::Creating,
                Error::<T>::IncorrectStatus
            );

            let bounded_name = enforce_limit!(name);

            let definition_step = DefinitionStep {
                name: bounded_name,
                attestor,
                threshold,
            };

            <DefinitionSteps<T>>::insert(
                (registry_id, definition_id),
                definition_step_index,
                definition_step,
            );

            Self::deposit_event(Event::DefinitionStepCreated(
                sender,
                registry_id,
                definition_id,
                definition_step_index,
            ));
            Ok(().into())
        }

        /// Update a definition_step
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` the Definition
        /// - `definition_step_index` New definition_step_index (should equal existing step count)
        /// - `name` name of the Definition Step. Can only be changed when in status `Creating`
        /// - `attestor` Attestor for the step. Can only be set to None when in status `Creating`
        /// - `threshold` Required threshold if Attestor is a group account else set to 1
        #[pallet::weight(<T as Config>::WeightInfo::update_definition_step(
            name.as_ref().map_or(0,|a|a.len()) as u32,
        ))]
        pub fn update_definition_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            definition_step_index: T::DefinitionStepIndex,
            name: Option<Vec<u8>>,
            attestor: Option<Option<T::AccountId>>,
            threshold: Option<T::MemberCount>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
                Error::<T>::NotAuthorized
            );
            let definition_maybe = <Definitions<T>>::get(registry_id, definition_id);
            ensure!(definition_maybe.is_some(), Error::<T>::NotFound);
            let definition = definition_maybe.unwrap();
            //can only rename when in status `Creating`
            ensure!(
                name.is_none() || definition.status == DefinitionStatus::Creating,
                Error::<T>::IncorrectStatus
            );
            //can only set attestor to None when in status `Creating`
            ensure!(
                attestor.is_none()
                    || attestor.as_ref().unwrap().is_some()
                    || definition.status == DefinitionStatus::Creating,
                Error::<T>::IncorrectStatus
            );
            ensure!(
                <DefinitionSteps<T>>::contains_key(
                    (registry_id, definition_id),
                    definition_step_index
                ),
                Error::<T>::NotFound
            );

            let bounded_name = enforce_limit_option!(name);

            <DefinitionSteps<T>>::mutate_exists(
                (registry_id, definition_id),
                definition_step_index,
                |maybe_definition_step| {
                    if let Some(ref mut definition_step) = maybe_definition_step {
                        if let Some(bounded_name) = bounded_name {
                            definition_step.name = bounded_name;
                        }
                        if let Some(attestor) = attestor {
                            definition_step.attestor = attestor;
                        }
                        if let Some(threshold) = threshold {
                            definition_step.threshold = threshold;
                        }
                    }
                },
            );

            Self::deposit_event(Event::DefinitionStepUpdated(
                sender,
                registry_id,
                definition_id,
                definition_step_index,
            ));
            Ok(().into())
        }

        /// Delete a definition_step
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` Definition the process is related to
        /// - `definition_step_index` New definition_step_index (should equal existing step count)
        #[pallet::weight(<T as Config>::WeightInfo::delete_definition_step(
            <T as Config>::DefinitionStepLimit::get()
        ))]
        pub fn delete_definition_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            mut definition_step_index: T::DefinitionStepIndex,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
                Error::<T>::NotAuthorized
            );
            let definition_maybe = <Definitions<T>>::get(registry_id, definition_id);
            ensure!(definition_maybe.is_some(), Error::<T>::NotFound);
            let definition = definition_maybe.unwrap();
            ensure!(
                definition.status == DefinitionStatus::Creating,
                Error::<T>::IncorrectStatus
            );
            ensure!(
                <DefinitionSteps<T>>::contains_key(
                    (registry_id, definition_id),
                    definition_step_index
                ),
                Error::<T>::NotFound
            );

            <DefinitionSteps<T>>::remove((registry_id, definition_id), definition_step_index);

            let one = T::DefinitionStepIndex::unique_saturated_from(1u32);

            let mut step_count = 0u32;
            loop {
                step_count = step_count + 1;
                definition_step_index = definition_step_index.saturating_add(one);
                let next_step =
                    <DefinitionSteps<T>>::take((registry_id, definition_id), definition_step_index);
                match next_step {
                    Some(next_step) => <DefinitionSteps<T>>::insert(
                        (registry_id, definition_id),
                        definition_step_index.saturating_sub(one),
                        next_step,
                    ),
                    None => break,
                };
            }

            Self::deposit_event(Event::DefinitionStepRemoved(
                sender,
                registry_id,
                definition_id,
                definition_step_index,
            ));
            Ok(Some(<T as Config>::WeightInfo::set_definition_active(step_count)).into())
        }

        /// Add a new process - any attestor on the first step can create a new process  (no voting required)
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` Definition the process is related to
        /// - `name` name of the Process
        #[pallet::weight(<T as Config>::WeightInfo::create_process(
            name.len() as u32
        ))]
        pub fn create_process(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

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
                definition_step.attestor == Some(sender.clone()),
                Error::<T>::NotAttestor
            );

            <T as Config>::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Process as u8),
                &sender,
            );

            let process_id = next_id!(NextProcessId<T>, T);

            let process = Process {
                name: bounded_name,
                status: ProcessStatus::InProgress,
            };

            <Processes<T>>::insert((registry_id, definition_id), process_id, process);

            let process_step = ProcessStep {
                attested: false,
                attributes: vec![],
            };

            <ProcessSteps<T>>::insert(
                (registry_id, definition_id, process_id),
                T::DefinitionStepIndex::unique_saturated_from(0u32),
                process_step,
            );

            Self::deposit_event(Event::ProcessCreated(
                sender,
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
            let sender = ensure_account_or_group!(origin);

            let bounded_name = enforce_limit!(name);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
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
                sender,
                registry_id,
                definition_id,
                process_id,
            ));
            Ok(().into())
        }

        /// Remove a process
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` Definition the process is related to
        /// - `process_id` Process to be removed
        #[pallet::weight(<T as Config>::WeightInfo::remove_process(
            <T as Config>::DefinitionStepLimit::get()
        ))]
        pub fn remove_process(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            ensure!(
                <Registries<T>>::contains_key(&sender, registry_id),
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
                sender,
                registry_id,
                definition_id,
                process_id,
            ));
            Ok(Some(<T as Config>::WeightInfo::set_definition_active(step_count)).into())
        }

        /// Update a process_step - any attestor on the step can update the step (no voting required)
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` Definition the process is related to
        /// - `process_id` the Process
        /// - `definition_step_index` index of step to be updated
        /// - `attributes` attributes of step (any previous attributes are replaced)
        #[pallet::weight(<T as Config>::WeightInfo::update_process_step(
            attributes.len() as u32,
            get_max_attribute_name_len(attributes),
            get_max_attribute_fact_len(attributes)
        ))]
        pub fn update_process_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
            definition_step_index: T::DefinitionStepIndex,
            attributes: Vec<Attribute<Vec<u8>, Vec<u8>>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_account_or_group!(origin);

            let definition_step =
                <DefinitionSteps<T>>::get((registry_id, definition_id), definition_step_index)
                    .ok_or(Error::<T>::NotFound)?;

            ensure!(
                definition_step.attestor == Some(sender.clone()),
                Error::<T>::NotAttestor
            );

            let attributes = enforce_limit_attributes!(attributes);

            <ProcessSteps<T>>::mutate_exists(
                (registry_id, definition_id, process_id),
                definition_step_index,
                |maybe_process_step| {
                    if let Some(ref mut process_step) = maybe_process_step {
                        process_step.attributes = attributes;
                    }
                },
            );

            Self::deposit_event(Event::ProcessStepUpdated(
                sender,
                registry_id,
                definition_id,
                process_id,
                definition_step_index,
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
        #[pallet::weight(<T as Config>::WeightInfo::attest_process_step( ))]
        pub fn attest_process_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
            definition_step_index: T::DefinitionStepIndex,
        ) -> DispatchResultWithPostInfo {
            let either = T::GroupsOriginAccountOrApproved::ensure_origin(origin)?;
            let (sender, yes_votes) = match either {
                Either::Left(account_id) => (account_id, None),
                Either::Right((_, yes_votes, _, group_account)) => (group_account, yes_votes),
            };

            let definition_step =
                <DefinitionSteps<T>>::get((registry_id, definition_id), definition_step_index)
                    .ok_or(Error::<T>::NotFound)?;

            ensure!(
                definition_step.attestor == Some(sender.clone()),
                Error::<T>::NotAttestor
            );

            ensure!(
                yes_votes.is_none() || yes_votes.unwrap() >= definition_step.threshold,
                Error::<T>::IncorrectThreshold
            );

            <ProcessSteps<T>>::mutate_exists(
                (registry_id, definition_id, process_id),
                definition_step_index,
                |maybe_process_step| {
                    if let Some(ref mut process_step) = maybe_process_step {
                        process_step.attested = true;
                    }
                },
            );

            let one = T::DefinitionStepIndex::unique_saturated_from(1u32);

            if <DefinitionSteps<T>>::contains_key(
                (registry_id, definition_id),
                definition_step_index.saturating_add(one),
            ) {
                let process_step = ProcessStep {
                    attested: false,
                    attributes: vec![],
                };

                <ProcessSteps<T>>::insert(
                    (registry_id, definition_id, process_id),
                    definition_step_index.saturating_add(one),
                    process_step,
                );
            } else {
                <Processes<T>>::mutate_exists(
                    (registry_id, definition_id),
                    process_id,
                    |maybe_process| {
                        if let Some(ref mut process) = maybe_process {
                            process.status = ProcessStatus::Completed;
                        }
                    },
                );
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
    }

    impl<T: Config> Module<T> {
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
        pub fn get_process_steps(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
        ) -> Vec<
            ProcessStep<
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
        > {
            let mut process_steps = Vec::new();
            <ProcessSteps<T>>::iter_prefix((registry_id, definition_id, process_id)).for_each(
                |(step_index, definition_step)| process_steps.push((step_index, definition_step)),
            );
            process_steps.sort_by(|(step_index_a, _), (step_index_b, _)| {
                step_index_a.partial_cmp(step_index_b).unwrap()
            });

            process_steps
                .into_iter()
                .map(|(_, process_step)| process_step)
                .collect()
        }

        pub fn get_process_step(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
            definition_step_index: T::DefinitionStepIndex,
        ) -> Option<
            ProcessStep<
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
        > {
            <ProcessSteps<T>>::get(
                (registry_id, definition_id, process_id),
                definition_step_index,
            )
        }

        // -- private functions --
    }

    // -- for use in weights --

    macro_rules! max_fact_len {
        ($fact:expr,$max_fact_len:ident) => {{
            let fact_len = match &$fact {
                Fact::Text(string) => string.len() as u32,
                _ => 10, //give minimum of 10 and don't bother checking for anything other than Text
            };
            if fact_len > $max_fact_len {
                $max_fact_len = fact_len;
            };
        }};
    }

    fn get_max_attribute_name_len(attributes: &Vec<Attribute<Vec<u8>, Vec<u8>>>) -> u32 {
        let mut max_attribute_name_len = 0;
        attributes.into_iter().for_each(|attribute| {
            if attribute.name.len() as u32 > max_attribute_name_len {
                max_attribute_name_len = attribute.name.len() as u32;
            };
        });
        max_attribute_name_len
    }

    fn get_max_attribute_fact_len(attributes: &Vec<Attribute<Vec<u8>, Vec<u8>>>) -> u32 {
        let mut max_fact_len = 0;
        attributes.into_iter().for_each(|attribute| {
            max_fact_len!(attribute.fact, max_fact_len);
        });
        max_fact_len
    }
}
