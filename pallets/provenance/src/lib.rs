//! # Provenance Module
//!
//! ## Overview
//!
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For process creators

//!
//! #### For Attestors

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use core::convert::TryInto;
    use extrinsic_extra::GetExtrinsicExtra;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use primitives::{bounded_vec::BoundedVec, *};
    use sp_runtime::{
        traits::{AtLeast32Bit, CheckedAdd, One, UniqueSaturatedInto},
        DispatchResult,
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

        type Origin: From<groups::RawOrigin<Self::AccountId, Self::GroupId, Self::MemberCount>>;

        type GetExtrinsicExtraSource: GetExtrinsicExtra<
            ModuleIndex = u8,
            ExtrinsicIndex = u8,
            AccountId = Self::AccountId,
        >;

        /// The maximum length of a name or symbol stored on-chain.
        type NameLimit: Get<u32>;

        /// The maximum length of a name or symbol stored on-chain.
        type FactStringLimit: Get<u32>;
    }

    pub type DefinitionStepIndex = u8;

    #[pallet::event]
    #[pallet::metadata(
        T::RegistryId = "RegistryId",
        T::DefinitionId = "DefinitionId",
        T::ProcessId = "ProcessId",
        T::GroupId = "GroupId",
        T::MemberCount = "MemberCount"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new Registry was created (registry_id)
        RegistryCreated(T::RegistryId),
        /// A Registry was Updated (registry_id)
        RegistryUpdated(T::RegistryId),
        /// A Registry was Removed (registry_id)
        RegistryRemoved(T::RegistryId),
        /// A new Definition was created (registry_id, definition_id)
        DefinitionCreated(T::RegistryId, T::DefinitionId),
        /// A new Definition was updated (registry_id, definition_id)
        DefinitionUpdated(T::RegistryId, T::DefinitionId),
        /// A Definition was updated to 'active' state (registry_id, definition_id)
        DefinitionSetActive(T::RegistryId, T::DefinitionId),
        /// A Definition was updated to 'inactive' state (registry_id, definition_id)
        DefinitionSetInactive(T::RegistryId, T::DefinitionId),
        /// A Definition was Removed (registry_id, definition_id)
        DefinitionRemoved(T::RegistryId, T::DefinitionId),
        /// A DefinitionStep was Removed (registry_id, definition_id, definition_step_index)
        DefinitionStepUpdated(T::RegistryId, T::DefinitionId, DefinitionStepIndex),
        /// A new Process was created (registry_id, definition_id, process_id)
        ProcessCreated(T::RegistryId, T::DefinitionId, T::ProcessId),
        /// A Process was Removed (registry_id, definition_id, process_id)
        ProcessUpdated(T::RegistryId, T::DefinitionId, T::ProcessId),
        /// A Process was Removed (registry_id, definition_id, process_id)
        ProcessRemoved(T::RegistryId, T::DefinitionId, T::ProcessId),
        /// A new ProcessStep was created (registry_id, definition_id, process_id, definition_step_index)
        ProcessStepCreated(
            T::RegistryId,
            T::DefinitionId,
            T::ProcessId,
            DefinitionStepIndex,
        ),
        /// A  ProcessStep was updated (registry_id, definition_id, process_id, definition_step_index)
        ProcessStepUpdated(
            T::RegistryId,
            T::DefinitionId,
            T::ProcessId,
            DefinitionStepIndex,
        ),
        /// A new ProcessStep was attested (registry_id, definition_id, process_id, definition_step_index)
        ProcessStepAttested(
            T::RegistryId,
            T::DefinitionId,
            T::ProcessId,
            DefinitionStepIndex,
        ),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value was None
        NoneValue,
        /// Not authorized
        NotAuthorized,
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
        AttestorGroupNotSet,
        /// The attestor group on a definition step does not exist.
        AttestorsInvalidGroup,
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
    /// (T::GroupId,T::RegistryId) => T::RegistryId
    pub(super) type Registries<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::GroupId,
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
        DefinitionStepIndex,
        DefinitionStep<T::GroupId, T::MemberCount, BoundedVec<u8, <T as Config>::NameLimit>>,
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
        DefinitionStepIndex,
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

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add a new registry
        ///
        /// Arguments: none

        #[pallet::weight( 10_000 +  T::DbWeight::get().writes(1))]
        pub fn create_registry(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            let bounded_name = enforce_limit!(name);

            <T as Config>::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Registry as u8),
                &group_account,
            );

            let registry_id = next_id!(NextRegistryId<T>, T);

            <Registries<T>>::insert(&group_id, &registry_id, Registry { name: bounded_name });

            Self::deposit_event(Event::RegistryCreated(registry_id));

            Ok(().into())
        }

        /// Update the registry
        ///
        /// Arguments:
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_registry(
            origin: OriginFor<T>,

            registry_id: T::RegistryId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                Self::is_registry_owner(group_id, registry_id),
                Error::<T>::NotAuthorized
            );

            let bounded_name = enforce_limit!(name);

            <Registries<T>>::try_mutate_exists(
                &group_id,
                &registry_id,
                |maybe_registry| -> DispatchResult {
                    let mut registry = maybe_registry.as_mut().ok_or(Error::<T>::NotFound)?;
                    registry.name = bounded_name;
                    Ok(())
                },
            )?;

            Self::deposit_event(Event::RegistryCreated(registry_id));
            Ok(().into())
        }

        /// Remove a registry
        ///
        /// Arguments:
        /// - `registry_id` Registry to be removed
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_registry(
            origin: OriginFor<T>,

            registry_id: T::RegistryId,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                Self::is_registry_owner(group_id, registry_id),
                Error::<T>::NotAuthorized
            );

            <Definitions<T>>::drain_prefix(registry_id).for_each(|(definition_id, _definition)| {
                <DefinitionSteps<T>>::remove_prefix((registry_id, definition_id));
                <Processes<T>>::drain_prefix((registry_id, definition_id)).for_each(
                    |(process_id, _process)| {
                        <ProcessSteps<T>>::remove_prefix((registry_id, definition_id, process_id));
                    },
                );
            });

            <Registries<T>>::remove(group_id, registry_id);

            Self::deposit_event(Event::RegistryRemoved(registry_id));
            Ok(().into())
        }

        /// Add a new definition
        ///
        /// Arguments:
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_definition(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            name: Vec<u8>,
            definition_steps: Vec<DefinitionStep<T::GroupId, T::MemberCount, Vec<u8>>>,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                Self::is_registry_owner(group_id, registry_id),
                Error::<T>::NotAuthorized
            );

            let bounded_name = enforce_limit!(name);

            let definition_steps = enforce_limit_definition_steps!(definition_steps);

            <T as Config>::GetExtrinsicExtraSource::charge_extrinsic_extra(
                &MODULE_INDEX,
                &(ExtrinsicIndex::Definition as u8),
                &group_account,
            );

            let definition_id = next_id!(NextDefinitionId<T>, T);

            let definition = Definition {
                name: bounded_name,
                status: DefinitionStatus::Creating,
            };

            <Definitions<T>>::insert(registry_id, definition_id, definition);

            definition_steps.into_iter().enumerate().for_each(
                |(definition_step_index, definition_step)| {
                    let definition_step_index =
                        UniqueSaturatedInto::<DefinitionStepIndex>::unique_saturated_into(
                            definition_step_index,
                        );
                    <DefinitionSteps<T>>::insert(
                        (registry_id, definition_id),
                        definition_step_index,
                        definition_step,
                    );
                },
            );

            Self::deposit_event(Event::DefinitionCreated(registry_id, definition_id));
            Ok(().into())
        }

        /// Update definition - replace name and / or steps. Status must be 'Creating'
        ///
        /// Arguments:
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_definition(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            name: Option<Vec<u8>>,
            definition_steps: Option<Vec<DefinitionStep<T::GroupId, T::MemberCount, Vec<u8>>>>,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                Self::is_registry_owner(group_id, registry_id),
                Error::<T>::NotAuthorized
            );

            let bounded_name = enforce_limit_option!(name);

            let definition_steps = enforce_limit_definition_steps_option!(definition_steps);

            let definition = <Definitions<T>>::get(registry_id, definition_id);
            ensure!(definition.is_some(), Error::<T>::NotFound);

            ensure!(
                definition.unwrap().status == DefinitionStatus::Creating,
                Error::<T>::IncorrectStatus
            );

            if let Some(bounded_name) = bounded_name {
                <Definitions<T>>::try_mutate_exists(
                    registry_id,
                    definition_id,
                    |maybe_definition| -> DispatchResult {
                        let mut definition =
                            maybe_definition.as_mut().ok_or(Error::<T>::NotFound)?;
                        definition.name = bounded_name;
                        Ok(())
                    },
                )?;
            }

            if let Some(definition_steps) = definition_steps {
                <DefinitionSteps<T>>::remove_prefix((registry_id, definition_id));

                definition_steps.into_iter().enumerate().for_each(
                    |(definition_step_index, definition_step)| {
                        let definition_step_index =
                            UniqueSaturatedInto::<DefinitionStepIndex>::unique_saturated_into(
                                definition_step_index,
                            );
                        <DefinitionSteps<T>>::insert(
                            (registry_id, definition_id),
                            definition_step_index,
                            definition_step,
                        );
                    },
                );
            }

            Self::deposit_event(Event::DefinitionUpdated(registry_id, definition_id));
            Ok(().into())
        }

        /// Set definition active
        ///
        /// Arguments:
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_definition_active(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                Self::is_registry_owner(group_id, registry_id),
                Error::<T>::NotAuthorized
            );

            let mut attestor_group_set = true;
            let mut attestors_group_valid = true;

            <DefinitionSteps<T>>::iter_prefix((registry_id, definition_id)).for_each(
                |(_step_index, definition_step)| {
                    if let Some(group_id) = definition_step.group_id {
                        if groups::Module::<T>::groups(group_id).is_none() {
                            attestors_group_valid = false;
                        }
                    } else {
                        attestor_group_set = false;
                    }
                },
            );
            ensure!(attestor_group_set, Error::<T>::AttestorGroupNotSet);
            ensure!(attestors_group_valid, Error::<T>::AttestorsInvalidGroup);

            <Definitions<T>>::try_mutate_exists(
                registry_id,
                definition_id,
                |maybe_definition| -> DispatchResult {
                    let mut definition = maybe_definition.as_mut().ok_or(Error::<T>::NotFound)?;
                    definition.status = DefinitionStatus::Active;
                    Ok(())
                },
            )?;

            Self::deposit_event(Event::DefinitionSetActive(registry_id, definition_id));
            Ok(().into())
        }
        /// Set definition inactive
        ///
        /// Arguments:
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_definition_inactive(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                Self::is_registry_owner(group_id, registry_id),
                Error::<T>::NotAuthorized
            );

            <Definitions<T>>::try_mutate_exists(
                registry_id,
                definition_id,
                |maybe_definition| -> DispatchResult {
                    let mut definition = maybe_definition.as_mut().ok_or(Error::<T>::NotFound)?;
                    definition.status = DefinitionStatus::Inactive;
                    Ok(())
                },
            )?;

            Self::deposit_event(Event::DefinitionSetInactive(registry_id, definition_id));
            Ok(().into())
        }

        /// Remove a definition
        ///
        /// Arguments:
        /// - `registry_id` Registry the Definition is in
        /// - `definition_id` Definition to be removed
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_definition(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                Self::is_registry_owner(group_id, registry_id),
                Error::<T>::NotAuthorized
            );
            ensure!(
                Self::is_definition_in_registry(registry_id, definition_id),
                Error::<T>::NotFound
            );

            <DefinitionSteps<T>>::remove_prefix((registry_id, definition_id));
            <Processes<T>>::drain_prefix((registry_id, definition_id)).for_each(
                |(process_id, _process)| {
                    <ProcessSteps<T>>::remove_prefix((registry_id, definition_id, process_id));
                },
            );

            <Definitions<T>>::remove(registry_id, definition_id);

            Self::deposit_event(Event::DefinitionRemoved(registry_id, definition_id));
            Ok(().into())
        }

        /// Update a new definition_step
        ///
        /// Arguments:
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_definition_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            definition_step_index: DefinitionStepIndex,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                Self::is_registry_owner(group_id, registry_id),
                Error::<T>::NotAuthorized
            );
            ensure!(
                Self::is_definition_step_in_definition(
                    registry_id,
                    definition_id,
                    definition_step_index
                ),
                Error::<T>::NotFound
            );
            let bounded_name = enforce_limit!(name);

            <DefinitionSteps<T>>::try_mutate_exists(
                (registry_id, definition_id),
                definition_step_index,
                |maybe_definition_step| -> DispatchResult {
                    let mut definition_step =
                        maybe_definition_step.as_mut().ok_or(Error::<T>::NotFound)?;
                    definition_step.name = bounded_name;
                    Ok(())
                },
            )?;

            //TODO: can the attestor group be changed?

            Self::deposit_event(Event::DefinitionStepUpdated(
                registry_id,
                definition_id,
                definition_step_index,
            ));
            Ok(().into())
        }

        /// Add a new process - any attestor on the first step can create a new process  (no voting required)
        ///
        /// Arguments:
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_process(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            let bounded_name = enforce_limit!(name);

            let definition = <Definitions<T>>::get(registry_id, definition_id);
            ensure!(definition.is_some(), Error::<T>::NotFound);

            ensure!(
                definition.unwrap().status == DefinitionStatus::Active,
                Error::<T>::IncorrectStatus
            );

            let definition_step = <DefinitionSteps<T>>::get((registry_id, definition_id), 0)
                .ok_or(Error::<T>::NotFound)?;

            ensure!(
                definition_step.group_id == Some(group_id),
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

            let process_step = ProcessStep {
                attested: false,
                attributes: vec![],
            };

            <ProcessSteps<T>>::insert((registry_id, definition_id, process_id), 0, process_step);

            Self::deposit_event(Event::ProcessCreated(
                registry_id,
                definition_id,
                process_id,
            ));
            Ok(().into())
        }

        /// Update a process - admin can rename a process
        ///
        /// Arguments:
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_process(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            let bounded_name = enforce_limit!(name);

            ensure!(
                Self::is_registry_owner(group_id, registry_id),
                Error::<T>::NotAuthorized
            );

            ensure!(
                Self::is_process_in_definition(registry_id, definition_id, process_id),
                Error::<T>::NotFound
            );

            <Processes<T>>::try_mutate_exists(
                (registry_id, definition_id),
                process_id,
                |maybe_process| -> DispatchResult {
                    let mut process = maybe_process.as_mut().ok_or(Error::<T>::NotFound)?;
                    process.name = bounded_name;
                    Ok(())
                },
            )?;

            Self::deposit_event(Event::ProcessUpdated(
                registry_id,
                definition_id,
                process_id,
            ));
            Ok(().into())
        }

        /// Remove a process
        ///
        /// Arguments:
        /// - `process_id` Process to be removed
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_process(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            ensure!(
                Self::is_registry_owner(group_id, registry_id),
                Error::<T>::NotAuthorized
            );
            ensure!(
                Self::is_process_in_definition(registry_id, definition_id, process_id),
                Error::<T>::NotFound
            );

            <Processes<T>>::remove((registry_id, definition_id), process_id);
            <ProcessSteps<T>>::remove_prefix((registry_id, definition_id, process_id));

            Self::deposit_event(Event::ProcessRemoved(
                registry_id,
                definition_id,
                process_id,
            ));
            Ok(().into())
        }

        /// Update a process_step - any attestor on the step can update the step (no voting required)
        ///
        /// Arguments:
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_process_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
            definition_step_index: DefinitionStepIndex,
            attributes: Vec<Attribute<Vec<u8>, Vec<u8>>>,
        ) -> DispatchResultWithPostInfo {
            let (group_id, _yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            let definition_step =
                <DefinitionSteps<T>>::get((registry_id, definition_id), definition_step_index)
                    .ok_or(Error::<T>::NotFound)?;

            ensure!(
                definition_step.group_id == Some(group_id),
                Error::<T>::NotAttestor
            );

            let attributes = enforce_limit_attributes!(attributes);

            <ProcessSteps<T>>::try_mutate_exists(
                (registry_id, definition_id, process_id),
                definition_step_index,
                |maybe_process_step| -> DispatchResult {
                    let mut process_step =
                        maybe_process_step.as_mut().ok_or(Error::<T>::NotFound)?;
                    process_step.attributes = attributes;
                    Ok(())
                },
            )?;

            Self::deposit_event(Event::ProcessStepUpdated(
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
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn attest_process_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
            definition_step_index: DefinitionStepIndex,
        ) -> DispatchResultWithPostInfo {
            let (group_id, yes_votes, _no_votes, _group_account) =
                T::GroupsOriginByCallerThreshold::ensure_origin(origin)?;

            let definition_step =
                <DefinitionSteps<T>>::get((registry_id, definition_id), definition_step_index)
                    .ok_or(Error::<T>::NotFound)?;

            ensure!(
                definition_step.group_id == Some(group_id),
                Error::<T>::NotAttestor
            );

            ensure!(
                yes_votes.is_none() || yes_votes.unwrap() >= definition_step.threshold,
                Error::<T>::IncorrectThreshold
            );

            <ProcessSteps<T>>::try_mutate_exists(
                (registry_id, definition_id, process_id),
                definition_step_index,
                |maybe_process_step| -> DispatchResult {
                    let mut process_step =
                        maybe_process_step.as_mut().ok_or(Error::<T>::NotFound)?;
                    process_step.attested = true;
                    Ok(())
                },
            )?;

            if <DefinitionSteps<T>>::contains_key(
                (registry_id, definition_id),
                definition_step_index + 1,
            ) {
                let process_step = ProcessStep {
                    attested: false,
                    attributes: vec![],
                };

                <ProcessSteps<T>>::insert(
                    (registry_id, definition_id, process_id),
                    definition_step_index + 1,
                    process_step,
                );
            } else {
                <Processes<T>>::try_mutate_exists(
                    (registry_id, definition_id),
                    process_id,
                    |maybe_process| -> DispatchResult {
                        let mut process = maybe_process.as_mut().ok_or(Error::<T>::NotFound)?;
                        process.status = ProcessStatus::Completed;
                        Ok(())
                    },
                )?;
            }

            Self::deposit_event(Event::ProcessStepAttested(
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
            group_id: T::GroupId,
        ) -> Vec<(
            T::RegistryId,
            Registry<BoundedVec<u8, <T as Config>::NameLimit>>,
        )> {
            let mut registries = Vec::new();

            <Registries<T>>::iter_prefix(group_id)
                .for_each(|(registry_id, registry)| registries.push((registry_id, registry)));

            registries
        }
        pub fn get_registry(
            group_id: T::GroupId,
            registry_id: T::RegistryId,
        ) -> Option<Registry<BoundedVec<u8, <T as Config>::NameLimit>>> {
            <Registries<T>>::get(group_id, registry_id)
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
            DefinitionStepIndex,
            DefinitionStep<T::GroupId, T::MemberCount, BoundedVec<u8, <T as Config>::NameLimit>>,
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
            definition_step_index: DefinitionStepIndex,
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

        fn is_registry_owner(group_id: T::GroupId, registry_id: T::RegistryId) -> bool {
            <Registries<T>>::contains_key(group_id, registry_id)
        }

        fn is_definition_in_registry(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> bool {
            <Definitions<T>>::contains_key(registry_id, definition_id)
        }

        fn is_definition_step_in_definition(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            definition_step_index: DefinitionStepIndex,
        ) -> bool {
            <DefinitionSteps<T>>::contains_key((registry_id, definition_id), definition_step_index)
        }
        fn is_process_in_definition(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
        ) -> bool {
            <Processes<T>>::contains_key((registry_id, definition_id), process_id)
        }

        // pub fn is_valid_definition_step(
        //     registry_id: T::RegistryId,
        //     definition_id: T::DefinitionId,
        //     definition_step_index: DefinitionStepIndex,
        //     process_id: T::ProcessId,
        // ) -> bool {
        //     //must not already exist
        //     if <ProcessSteps<T>>::contains_key(
        //         (registry_id, definition_id, process_id),
        //         definition_step_index,
        //     ) {
        //         return false;
        //     }
        //     definition_step_index==0 ||
        // //previous step must exist
        // <ProcessSteps<T>>::contains_key(
        //     (registry_id, definition_id, process_id),
        //     definition_step_index - 1,
        // )
        // }
    }
}
