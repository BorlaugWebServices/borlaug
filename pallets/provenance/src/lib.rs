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
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use membership::Membership;
    use primitives::{
        attribute::Attribute,
        definition::Definition,
        definition_step::DefinitionStep,
        did::Did,
        process::{Process, ProcessStatus},
        process_step::ProcessStep,
        registry::Registry,
    };
    // #[cfg(not(feature = "std"))]
    // use sp_io::hashing::blake2_256;
    use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, One, UniqueSaturatedInto};
    use sp_std::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type RegistryId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
        type DefinitionId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
        type ProcessId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
        type GroupId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;

        type MembershipSource: Membership<GroupId = Self::GroupId, AccountId = Self::AccountId>;
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
        /// A new Registry was created (RegistryId)
        RegistryCreated(T::RegistryId),
        /// A Registry was Updated (RegistryId)
        RegistryUpdated(T::RegistryId),
        /// A Registry was Removed (RegistryId)
        RegistryRemoved(T::RegistryId),
        /// A new Definition was created (RegistryId)
        DefinitionCreated(T::RegistryId, T::DefinitionId),
        /// A Definition was Removed (RegistryId,DefinitionId)
        DefinitionRemoved(T::RegistryId, T::DefinitionId),
        /// A DefinitionStep was Removed (RegistryId,DefinitionId,DefinitionStepIndex)
        DefinitionStepUpdated(T::RegistryId, T::DefinitionId, DefinitionStepIndex),
        /// A new Process was created (RegistryId,DefinitionId,ProcessId)
        ProcessCreated(T::RegistryId, T::DefinitionId, T::ProcessId),
        /// A Process was Removed (RegistryId,DefinitionId,ProcessId)
        ProcessUpdated(T::RegistryId, T::DefinitionId, T::ProcessId),
        /// A Process was Removed (RegistryId,DefinitionId,ProcessId)
        ProcessRemoved(T::RegistryId, T::DefinitionId, T::ProcessId),
        /// A new ProcessStep was created (RegistryId,DefinitionId,ProcessId,DefinitionStepIndex)
        ProcessStepCreated(
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
        ValueQuery,
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
        ValueQuery,
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
        ValueQuery,
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
        ValueQuery,
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
        ProcessStep,
        ValueQuery,
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
        /// Arguments: none
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_registry(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            //TODO: make a helper function or macro
            let registry_id = Self::next_registry_id();
            let next_id = registry_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextRegistryId<T>>::put(next_id);

            <Registries<T>>::insert(&sender, &registry_id, Registry { name });

            Self::deposit_event(Event::RegistryCreated(registry_id));
            Ok(().into())
        }

        /// Update the registry
        ///
        /// Arguments: none
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_registry(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
                Error::<T>::NotFound
            );

            <Registries<T>>::mutate(&sender, &registry_id, |mut registry| registry.name = name);

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
                Error::<T>::NotFound
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
        /// Arguments: none
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
                Error::<T>::NotFound
            );

            // let definition_id = next_id!(NextDefinitionId);

            //TODO: make a helper function or macro
            let definition_id = <NextDefinitionId<T>>::get();
            let next_id = definition_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextDefinitionId<T>>::put(next_id);

            let definition = Definition { name };

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

        /// Remove a definition
        ///
        /// Arguments:
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
                Error::<T>::NotFound
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
        /// Arguments: none
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
                Error::<T>::NotFound
            );
            ensure!(
                Self::is_definition_step_in_definition(
                    registry_id,
                    definition_id,
                    definition_step_index
                ),
                Error::<T>::NotFound
            );

            //TODO: will we allow updating name?

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
        /// Arguments: none
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_process(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let definition_step = <DefinitionSteps<T>>::get((registry_id, definition_id), 0);

            ensure!(
                T::MembershipSource::is_member(definition_step.group_id, sender),
                Error::<T>::NotAttestor
            );

            //TODO: make a helper function or macro
            let process_id = Self::next_process_id();
            let next_id = process_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextProcessId<T>>::put(next_id);

            let process = Process {
                name,
                status: ProcessStatus::InProgress,
            };

            <Processes<T>>::insert((registry_id, definition_id), process_id, process);

            Self::deposit_event(Event::ProcessCreated(
                registry_id,
                definition_id,
                process_id,
            ));
            Ok(().into())
        }

        /// Update a process
        ///
        /// Arguments: none
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn update_process(origin,registry_id: T::RegistryId,definition_id: T::DefinitionId,process_id: T::ProcessId) {
        //     let sender = ensure_signed(origin)?;

        //     //TODO:is attestor
        //     ensure!(Self::is_process_in_definition(registry_id,definition_id,process_id), Error::<T>::NotFound);

        //     //TODO:update

        //     Self::deposit_event(Event::ProcessUpdated(registry_id,definition_id,process_id));
        //    Ok(().into())
        // }

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
                Error::<T>::NotFound
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

        /// Add a new process_step
        ///
        /// Arguments: none
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_process_step(
            origin: OriginFor<T>,
            did: Did,
            registry_id: T::RegistryId,
            definition_id: T::DefinitionId,
            definition_step_index: DefinitionStepIndex,
            process_id: T::ProcessId,
            attributes: Vec<Attribute>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_process_in_definition(registry_id, definition_id, process_id),
                Error::<T>::NotFound
            );

            ensure!(
                <DefinitionSteps<T>>::contains_key(
                    (registry_id, definition_id),
                    definition_step_index
                ),
                Error::<T>::NotFound
            );

            let definition_step =
                <DefinitionSteps<T>>::get((registry_id, definition_id), definition_step_index);

            ensure!(
                T::MembershipSource::is_member(definition_step.group_id, sender),
                Error::<T>::NotAttestor
            );
            ensure!(
                Self::is_valid_definition_step(
                    registry_id,
                    definition_id,
                    definition_step_index,
                    process_id
                ),
                Error::<T>::NotAttestor
            );

            let process_step = ProcessStep {
                attested_by: did,
                attributes,
            };

            <ProcessSteps<T>>::insert(
                (registry_id, definition_id, process_id),
                definition_step_index,
                process_step,
            );

            Self::deposit_event(Event::ProcessStepCreated(
                registry_id,
                definition_id,
                process_id,
                definition_step_index,
            ));
            Ok(().into())
        }

        // /// Update a new process_step
        // ///
        // /// Arguments: none
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn update_process_step(origin,registry_id: T::RegistryId,definition_id: T::DefinitionId,process_id:T::ProcessId,process_step_id: T::ProcessStepId) {
        //     let sender = ensure_signed(origin)?;

        //     //TODO:is attestor
        //     ensure!(Self::is_process_step_in_process(registry_id,definition_id,process_id,process_step_id), Error::<T>::NotFound);

        //     //TODO:update

        //     Self::deposit_event(Event::ProcessStepUpdated(registry_id,definition_id,process_id,process_step_id));
        // }

        // /// Remove a process_step
        // ///
        // /// Arguments:
        // /// - `process_step_id` ProcessStep to be removed
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn remove_process_step(origin, registry_id: T::RegistryId,definition_id: T::DefinitionId,process_id:T::ProcessId,process_step_id: T::ProcessStepId) {
        //     let sender = ensure_signed(origin)?;

        //    //TODO:is attestor
        //     ensure!(Self::is_process_step_in_process(registry_id,definition_id,process_id,process_step_id), Error::<T>::NotFound);

        //     <ProcessSteps<T>>::remove((registry_id, definition_id,process_id),process_step_id);

        //     Self::deposit_event(Event::ProcessStepRemoved(registry_id,definition_id,process_id,process_step_id));
        // }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --
        pub fn get_registries(account: T::AccountId) -> Vec<(T::RegistryId, Registry)> {
            let mut registries = Vec::new();

            <Registries<T>>::iter_prefix(account)
                .for_each(|(registry_id, registry)| registries.push((registry_id, registry)));

            registries
        }
        pub fn get_registry(account: T::AccountId, registry_id: T::RegistryId) -> Registry {
            <Registries<T>>::get(account, registry_id)
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
    macro_rules! next_id {
        ($id:ty) => {{
            let current_id = <$id<T>>::get();
            let next_id = current_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <$id<T>>::put(next_id);
            current_id
        }};
    }
}
