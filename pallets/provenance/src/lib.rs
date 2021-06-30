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
    use extrinsic_extra::GetExtrinsicExtra;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Currency,
    };
    use frame_system::pallet_prelude::*;
    use group_info::GroupInfo;
    use primitives::{
        attribute::Attribute,
        definition::{Definition, DefinitionStatus},
        definition_step::DefinitionStep,
        process::{Process, ProcessStatus},
        process_step::ProcessStep,
        registry::Registry,
    };
    use sp_runtime::{
        traits::{AtLeast32Bit, CheckedAdd, One, UniqueSaturatedInto},
        DispatchResult,
    };
    use sp_std::prelude::*;

    const MODULE_INDEX: u8 = 3;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type RegistryId: Parameter + Member + AtLeast32Bit + Default + Copy + PartialEq;
        type DefinitionId: Parameter + Member + AtLeast32Bit + Default + Copy + PartialEq;
        type ProcessId: Parameter + Member + AtLeast32Bit + Default + Copy + PartialEq;
        type GroupId: Parameter + Member + AtLeast32Bit + Eq + Default + Copy + PartialEq;
        type MemberCount: Parameter
            + Member
            + PartialOrd
            + AtLeast32Bit
            + Eq
            + Default
            + Copy
            + PartialEq;

        type Origin: From<groups::RawOrigin<Self::AccountId, Self::GroupId, Self::MemberCount>>;

        type GroupApprovalOrigin: EnsureOrigin<
            <Self as frame_system::Config>::Origin,
            Success = (
                Self::GroupId,
                Option<Self::MemberCount>,
                Option<Self::MemberCount>,
                Self::AccountId,
            ),
        >;

        type GroupInfoSource: GroupInfo<GroupId = Self::GroupId, AccountId = Self::AccountId>;

        type Currency: Currency<Self::AccountId>;

        type GetExtrinsicExtraSource: GetExtrinsicExtra<
            ModuleIndex = u8,
            ExtrinsicIndex = u8,
            Balance = <Self::Currency as Currency<Self::AccountId>>::Balance,
        >;
    }

    pub type DefinitionStepIndex = u8;

    #[pallet::event]
    #[pallet::metadata(
        T::RegistryId = "RegistryId",
        T::DefinitionId = "DefinitionId",
        T::ProcessId = "ProcessId",
        T::GroupId = "GroupId"
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
        /// IncorrectStatus
        IncorrectStatus,
        /// No id was found (either user is not owner, or entity does not exist)
        NotFound,
        /// Cannot delete non-empty registry
        NotEmpty,
        /// Is not an attestor for the necessary definition step
        NotAttestor,
        /// Id out of bounds
        NoIdAvailable,
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
    /// An account can have multiple Regitries of process definitions
    /// (T::AccountId,T::RegistryId) => T::RegistryId
    pub(super) type Registries<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::RegistryId,
        Registry,
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
        Definition,
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
        DefinitionStep<T::GroupId>,
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
        Process,
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
        ProcessStep<T::AccountId>,
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
                T::GroupApprovalOrigin::ensure_origin(origin)?;

            // ensure!(group_account.is_some(), Error::<T>::NotAuthorized);
            // let group_account = group_account.unwrap();

            // let sender = ensure_signed(origin)?;
            // ensure!(
            //     T::GroupOriginSource::ensure_minimum(group_id, origin.into(), 1u32),
            //     Error::<T>::NotAuthorized
            // );

            // debug::info!("{:?}", T::Currency::total_balance(&sender));

            // let (deducted, a) = T::Currency::slash(
            //     &sender,
            //     T::GetExtrinsicExtraSource::get_extrinsic_extra(&MODULE_INDEX, &1u8),
            // );

            // debug::info!("{:?}", a);

            // debug::info!(
            //     "{:?}",
            //     T::GetExtrinsicExtraSource::get_extrinsic_extra(&MODULE_INDEX, &1u8)
            // );

            // debug::info!("{:?}", T::Currency::total_balance(&sender));

            let registry_id = next_id!(NextRegistryId<T>, T);

            <Registries<T>>::insert(&group_account, &registry_id, Registry { name });

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
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
                Error::<T>::NotAuthorized
            );

            <Registries<T>>::try_mutate_exists(
                &sender,
                &registry_id,
                |maybe_registry| -> DispatchResult {
                    let mut registry = maybe_registry.as_mut().ok_or(Error::<T>::NotFound)?;
                    registry.name = name;
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
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
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

            <Registries<T>>::remove(sender, registry_id);

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
            definition_steps: Vec<DefinitionStep<T::GroupId>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
                Error::<T>::NotAuthorized
            );

            let definition_id = next_id!(NextDefinitionId<T>, T);

            let definition = Definition {
                name,
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
            definition_steps: Option<Vec<DefinitionStep<T::GroupId>>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
                Error::<T>::NotAuthorized
            );

            let definition = <Definitions<T>>::get(registry_id, definition_id);
            ensure!(definition.is_some(), Error::<T>::NotFound);

            ensure!(
                definition.unwrap().status == DefinitionStatus::Creating,
                Error::<T>::IncorrectStatus
            );

            if let Some(name) = name {
                <Definitions<T>>::try_mutate_exists(
                    registry_id,
                    definition_id,
                    |maybe_definition| -> DispatchResult {
                        let mut definition =
                            maybe_definition.as_mut().ok_or(Error::<T>::NotFound)?;
                        definition.name = name;
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
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
                Error::<T>::NotAuthorized
            );

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
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
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
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
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
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
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

            <DefinitionSteps<T>>::try_mutate_exists(
                (registry_id, definition_id),
                definition_step_index,
                |maybe_definition_step| -> DispatchResult {
                    let mut definition_step =
                        maybe_definition_step.as_mut().ok_or(Error::<T>::NotFound)?;
                    definition_step.name = name;
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

        /// Add a new process
        ///
        /// Arguments:
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_process(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let definition = <Definitions<T>>::get(registry_id, definition_id);
            ensure!(definition.is_some(), Error::<T>::NotFound);

            ensure!(
                definition.unwrap().status == DefinitionStatus::Active,
                Error::<T>::IncorrectStatus
            );

            let definition_step = <DefinitionSteps<T>>::get((registry_id, definition_id), 0)
                .ok_or(Error::<T>::NotFound)?;

            ensure!(
                T::GroupInfoSource::is_group_account(definition_step.group_id, &sender),
                Error::<T>::NotAttestor
            );

            let process_id = next_id!(NextProcessId<T>, T);

            let process = Process {
                name,
                status: ProcessStatus::InProgress,
            };

            <Processes<T>>::insert((registry_id, definition_id), process_id, process);

            let process_step = ProcessStep {
                attested_by: None,
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
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
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
                    process.name = name;
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
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
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

        // /// Add a new process_step
        // ///
        // /// Arguments:
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn create_process_step(
        //     origin: OriginFor<T>,
        //     registry_id: T::RegistryId,
        //     definition_id: T::DefinitionId,
        //     definition_step_index: DefinitionStepIndex,
        //     process_id: T::ProcessId,
        //     attributes: Vec<Attribute>,
        // ) -> DispatchResultWithPostInfo {
        //     let sender = ensure_signed(origin)?;

        //     ensure!(
        //         Self::is_process_in_definition(registry_id, definition_id, process_id),
        //         Error::<T>::NotFound
        //     );

        //     ensure!(
        //         <DefinitionSteps<T>>::contains_key(
        //             (registry_id, definition_id),
        //             definition_step_index
        //         ),
        //         Error::<T>::NotFound
        //     );

        //     let definition_step =
        //         <DefinitionSteps<T>>::get((registry_id, definition_id), definition_step_index)
        //             .ok_or(Error::<T>::NotFound)?;

        //     ensure!(
        //         T::GroupInfoSource::is_group_account(definition_step.group_id, &sender),
        //         Error::<T>::NotAttestor
        //     );
        //     ensure!(
        //         Self::is_valid_definition_step(
        //             registry_id,
        //             definition_id,
        //             definition_step_index,
        //             process_id
        //         ),
        //         Error::<T>::NotAttestor
        //     );

        //     let process_step = ProcessStep {
        //         attested_by: None,
        //         attributes,
        //     };

        //     <ProcessSteps<T>>::insert(
        //         (registry_id, definition_id, process_id),
        //         definition_step_index,
        //         process_step,
        //     );

        //     Self::deposit_event(Event::ProcessStepCreated(
        //         registry_id,
        //         definition_id,
        //         process_id,
        //         definition_step_index,
        //     ));
        //     Ok(().into())
        // }

        /// Update a process_step
        ///
        /// Arguments:
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_process_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
            definition_step_index: DefinitionStepIndex,
            attributes: Vec<Attribute>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let definition_step =
                <DefinitionSteps<T>>::get((registry_id, definition_id), definition_step_index)
                    .ok_or(Error::<T>::NotFound)?;

            ensure!(
                T::GroupInfoSource::is_group_account(definition_step.group_id, &sender),
                Error::<T>::NotAttestor
            );

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

        /// Attest a process_step
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
            let sender = ensure_signed(origin)?;

            let definition_step =
                <DefinitionSteps<T>>::get((registry_id, definition_id), definition_step_index)
                    .ok_or(Error::<T>::NotFound)?;

            ensure!(
                T::GroupInfoSource::is_group_account(definition_step.group_id, &sender),
                Error::<T>::NotAttestor
            );

            //TODO: how do we record attestor? (sender is group account, not actual attestor)

            <ProcessSteps<T>>::try_mutate_exists(
                (registry_id, definition_id, process_id),
                definition_step_index,
                |maybe_process_step| -> DispatchResult {
                    let mut process_step =
                        maybe_process_step.as_mut().ok_or(Error::<T>::NotFound)?;
                    process_step.attested_by = Some(sender);
                    Ok(())
                },
            )?;

            if <DefinitionSteps<T>>::contains_key(
                (registry_id, definition_id),
                definition_step_index + 1,
            ) {
                let process_step = ProcessStep {
                    attested_by: None,
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
        pub fn get_registries(account: T::AccountId) -> Vec<(T::RegistryId, Registry)> {
            let mut registries = Vec::new();

            <Registries<T>>::iter_prefix(account)
                .for_each(|(registry_id, registry)| registries.push((registry_id, registry)));

            registries
        }
        pub fn get_registry(account: T::AccountId, registry_id: T::RegistryId) -> Option<Registry> {
            <Registries<T>>::get(account, registry_id)
        }

        pub fn get_definitions(registry_id: T::RegistryId) -> Vec<(T::DefinitionId, Definition)> {
            let mut definitions = Vec::new();

            <Definitions<T>>::iter_prefix(registry_id).for_each(|(definition_id, definition)| {
                definitions.push((definition_id, definition))
            });

            definitions
        }
        pub fn get_definition(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> Option<Definition> {
            <Definitions<T>>::get(registry_id, definition_id)
        }
        pub fn get_definition_steps(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
        ) -> Vec<(DefinitionStepIndex, DefinitionStep<T::GroupId>)> {
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
        ) -> Vec<(T::ProcessId, Process)> {
            let mut processes = Vec::new();

            <Processes<T>>::iter_prefix((registry_id, definition_id))
                .for_each(|(process_id, process)| processes.push((process_id, process)));

            processes
        }
        pub fn get_process(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
        ) -> Option<Process> {
            <Processes<T>>::get((registry_id, definition_id), process_id)
        }
        pub fn get_process_steps(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            process_id: T::ProcessId,
        ) -> Vec<ProcessStep<T::AccountId>> {
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
        ) -> Option<ProcessStep<T::AccountId>> {
            <ProcessSteps<T>>::get(
                (registry_id, definition_id, process_id),
                definition_step_index,
            )
        }

        // -- private functions --

        fn is_registry_owner(account: &T::AccountId, registry_id: T::RegistryId) -> bool {
            <Registries<T>>::contains_key(account, registry_id)
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

        // fn is_process_step_in_process(
        //     registry_id: T::RegistryId,
        //     definition_id: T::DefinitionId,
        //     process_id: T::ProcessId,
        //     process_step_id: DefinitionStepIndex,
        // ) -> bool {
        //     <ProcessSteps<T>>::contains_key((registry_id, definition_id, process_id), process_step_id)
        // }

        // fn is_attestor_definition_step(
        //     registry_id: T::RegistryId,
        //     definition_id: T::DefinitionId,
        //     definition_step_index: DefinitionStepIndex,
        //     did: Did,
        // ) -> bool {
        //     <Attestors<T>>::contains_key((registry_id, definition_id, definition_step_index), did)
        // }

        pub fn is_valid_definition_step(
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            definition_step_index: DefinitionStepIndex,
            process_id: T::ProcessId,
        ) -> bool {
            //must not already exist
            if <ProcessSteps<T>>::contains_key(
                (registry_id, definition_id, process_id),
                definition_step_index,
            ) {
                return false;
            }
            definition_step_index==0 ||
        //previous step must exist
        <ProcessSteps<T>>::contains_key(
            (registry_id, definition_id, process_id),
            definition_step_index - 1,
        )
        }
    }
}
