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
    use primitives::{
        attestor::Attestor,
        attribute::Attribute,
        did::Did,
        sequence::{Sequence, SequenceStatus},
        sequence_step::SequenceStep,
        template::Template,
        template_step::TemplateStep,
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
        type TemplateId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
        type SequenceId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
    }

    pub type TemplateStepIndex = u8;

    #[pallet::event]
    #[pallet::metadata(
        T::RegistryId = "RegistryId",
        T::TemplateId = "TemplateId",
        T::SequenceId = "SequenceId"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new Registry was created (RegistryId)
        RegistryCreated(T::RegistryId),
        /// A Registry was Removed (RegistryId)
        RegistryRemoved(T::RegistryId),
        /// A new Template was created (RegistryId)
        TemplateCreated(T::RegistryId, T::TemplateId),
        /// A Template was Removed (RegistryId,TemplateId)
        TemplateRemoved(T::RegistryId, T::TemplateId),
        /// A TemplateStep was Removed (RegistryId,TemplateId,TemplateStepIndex)
        TemplateStepUpdated(T::RegistryId, T::TemplateId, TemplateStepIndex),
        /// A new Sequence was created (RegistryId,TemplateId,SequenceId)
        SequenceCreated(T::RegistryId, T::TemplateId, T::SequenceId),
        /// A Sequence was Removed (RegistryId,TemplateId,SequenceId)
        SequenceUpdated(T::RegistryId, T::TemplateId, T::SequenceId),
        /// A Sequence was Removed (RegistryId,TemplateId,SequenceId)
        SequenceRemoved(T::RegistryId, T::TemplateId, T::SequenceId),
        /// A new SequenceStep was created (RegistryId,TemplateId,SequenceId,TemplateStepIndex)
        SequenceStepCreated(
            T::RegistryId,
            T::TemplateId,
            T::SequenceId,
            TemplateStepIndex,
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
        /// Is not an attestor for the necessary template step
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
    pub fn TemplateIdDefault<T: Config>() -> T::TemplateId {
        1u32.into()
    }
    #[pallet::type_value]
    pub fn SequenceIdDefault<T: Config>() -> T::SequenceId {
        1u32.into()
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn registries)]
    /// An account can have multiple Regitries of process templates
    /// (T::AccountId,T::RegistryId) => T::RegistryId
    pub(super) type Registries<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::RegistryId,
        T::RegistryId,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn templates)]
    /// A Registry can have multiple process Templates
    /// (T::RegistryId,T::TemplateId) => Template
    pub(super) type Templates<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::RegistryId,
        Blake2_128Concat,
        T::TemplateId,
        Template,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn template_steps)]
    /// A Process has multiple steps
    /// (T::RegistryId,T::TemplateId), u8 => TemplateStep
    pub(super) type TemplateSteps<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::RegistryId, T::TemplateId),
        Blake2_128Concat,
        TemplateStepIndex,
        TemplateStep,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn attestors)]
    /// A Template step may have multiple attestors
    /// ((T::RegistryId,T::TemplateId,TemplateStepIndex),Did)=> Attestor
    pub(super) type Attestors<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::RegistryId, T::TemplateId, TemplateStepIndex),
        Blake2_128Concat,
        Did,
        Attestor,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn sequences)]
    /// A process Template can have multiple process Sequences
    /// (T::RegistryId,T::TemplateId), T::SequenceId => T::SequenceId
    pub(super) type Sequences<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::RegistryId, T::TemplateId),
        Blake2_128Concat,
        T::SequenceId,
        Sequence,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn sequence_steps)]
    /// A Sequence can have multiple process Sequence Steps
    /// (T::RegistryId,T::TemplateId,T::SequenceId), TemplateStepIndex => SequenceStep
    pub(super) type SequenceSteps<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::RegistryId, T::TemplateId, T::SequenceId),
        Blake2_128Concat,
        TemplateStepIndex,
        SequenceStep,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_registry_id)]
    /// The next available registry index
    pub(super) type NextRegistryId<T: Config> =
        StorageValue<_, T::RegistryId, ValueQuery, RegistryIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_template_id)]
    /// The next available template index
    pub(super) type NextTemplateId<T: Config> =
        StorageValue<_, T::TemplateId, ValueQuery, TemplateIdDefault<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_sequence_id)]
    /// The next available sequence index
    pub(super) type NextSequenceId<T: Config> =
        StorageValue<_, T::SequenceId, ValueQuery, SequenceIdDefault<T>>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add a new registry
        ///
        /// Arguments: none
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_registry(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            //TODO: make a helper function or macro
            let registry_id = Self::next_registry_id();
            let next_id = registry_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextRegistryId<T>>::put(next_id);

            <Registries<T>>::insert(&sender, &registry_id, registry_id);

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

            <Templates<T>>::drain_prefix(registry_id).for_each(|(template_id, _template)| {
                <TemplateSteps<T>>::remove_prefix((registry_id, template_id));
                <Sequences<T>>::drain_prefix((registry_id, template_id)).for_each(
                    |(sequence_id, _sequence)| {
                        <SequenceSteps<T>>::remove_prefix((registry_id, template_id, sequence_id));
                    },
                );
            });

            <Registries<T>>::remove(sender, registry_id);

            Self::deposit_event(Event::RegistryRemoved(registry_id));
            Ok(().into())
        }

        /// Add a new template
        ///
        /// Arguments: none
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_template(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            name: Vec<u8>,
            template_steps: Vec<(TemplateStep, Vec<Attestor>)>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
                Error::<T>::NotFound
            );

            //TODO: make a helper function or macro
            let template_id = Self::next_template_id();
            let next_id = template_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextTemplateId<T>>::put(next_id);

            let template = Template { name };

            <Templates<T>>::insert(registry_id, template_id, template);

            template_steps.into_iter().enumerate().for_each(
                |(template_step_index, (template_step, attestors))| {
                    let template_step_index =
                        UniqueSaturatedInto::<TemplateStepIndex>::unique_saturated_into(
                            template_step_index,
                        );
                    Self::create_template_step(
                        registry_id,
                        template_id,
                        template_step_index,
                        template_step,
                        attestors,
                    );
                },
            );

            Self::deposit_event(Event::TemplateCreated(registry_id, template_id));
            Ok(().into())
        }

        /// Remove a template
        ///
        /// Arguments:
        /// - `template_id` Template to be removed
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_template(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            template_id: T::TemplateId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
                Error::<T>::NotFound
            );
            ensure!(
                Self::is_template_in_registry(registry_id, template_id),
                Error::<T>::NotFound
            );

            <TemplateSteps<T>>::remove_prefix((registry_id, template_id));
            <Sequences<T>>::drain_prefix((registry_id, template_id)).for_each(
                |(sequence_id, _sequence)| {
                    <SequenceSteps<T>>::remove_prefix((registry_id, template_id, sequence_id));
                },
            );

            <Templates<T>>::remove(registry_id, template_id);

            Self::deposit_event(Event::TemplateRemoved(registry_id, template_id));
            Ok(().into())
        }

        /// Update a new template_step
        ///
        /// Arguments: none
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_template_step(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            template_id: T::TemplateId,
            template_step_index: TemplateStepIndex,
            add_attestors: Option<Vec<Attestor>>,
            remove_attestors: Option<Vec<Attestor>>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
                Error::<T>::NotFound
            );
            ensure!(
                Self::is_template_step_in_template(registry_id, template_id, template_step_index),
                Error::<T>::NotFound
            );

            if let Some(remove_attestors) = remove_attestors {
                remove_attestors.iter().for_each(|attestor| {
                    <Attestors<T>>::remove(
                        (registry_id, template_id, template_step_index),
                        attestor.did,
                    );
                });
            }

            if let Some(add_attestors) = add_attestors {
                add_attestors.iter().for_each(|attestor| {
                    <Attestors<T>>::insert(
                        (registry_id, template_id, template_step_index),
                        attestor.did,
                        attestor,
                    );
                });
            }

            Self::deposit_event(Event::TemplateStepUpdated(
                registry_id,
                template_id,
                template_step_index,
            ));
            Ok(().into())
        }

        /// Add a new sequence
        ///
        /// Arguments: none
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_sequence(
            origin: OriginFor<T>,
            did: Did,
            registry_id: T::RegistryId,
            template_id: T::TemplateId,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let _sender = ensure_signed(origin)?;

            //TODO: verify sender owns DID

            ensure!(
                Self::is_template_in_registry(registry_id, template_id),
                Error::<T>::NotFound
            );
            ensure!(
                Self::is_attestor_template_step(registry_id, template_id, 0, did),
                Error::<T>::NotAttestor
            );

            //TODO: make a helper function or macro
            let sequence_id = Self::next_sequence_id();
            let next_id = sequence_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextSequenceId<T>>::put(next_id);

            let sequence = Sequence {
                name,
                status: SequenceStatus::InProgress,
            };

            <Sequences<T>>::insert((registry_id, template_id), sequence_id, sequence);

            Self::deposit_event(Event::SequenceCreated(
                registry_id,
                template_id,
                sequence_id,
            ));
            Ok(().into())
        }

        /// Update a sequence
        ///
        /// Arguments: none
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn update_sequence(origin,registry_id: T::RegistryId,template_id: T::TemplateId,sequence_id: T::SequenceId) {
        //     let sender = ensure_signed(origin)?;

        //     //TODO:is attestor
        //     ensure!(Self::is_sequence_in_template(registry_id,template_id,sequence_id), Error::<T>::NotFound);

        //     //TODO:update

        //     Self::deposit_event(Event::SequenceUpdated(registry_id,template_id,sequence_id));
        //    Ok(().into())
        // }

        /// Remove a sequence
        ///
        /// Arguments:
        /// - `sequence_id` Sequence to be removed
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_sequence(
            origin: OriginFor<T>,
            registry_id: T::RegistryId,
            template_id: T::TemplateId,
            sequence_id: T::SequenceId,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                Self::is_registry_owner(&sender, registry_id),
                Error::<T>::NotFound
            );
            ensure!(
                Self::is_sequence_in_template(registry_id, template_id, sequence_id),
                Error::<T>::NotFound
            );

            <Sequences<T>>::remove((registry_id, template_id), sequence_id);
            <SequenceSteps<T>>::remove_prefix((registry_id, template_id, sequence_id));

            Self::deposit_event(Event::SequenceRemoved(
                registry_id,
                template_id,
                sequence_id,
            ));
            Ok(().into())
        }

        /// Add a new sequence_step
        ///
        /// Arguments: none
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_sequence_step(
            origin: OriginFor<T>,
            did: Did,
            registry_id: T::RegistryId,
            template_id: T::TemplateId,
            template_step_index: TemplateStepIndex,
            sequence_id: T::SequenceId,
            attributes: Vec<Attribute>,
        ) -> DispatchResultWithPostInfo {
            let _sender = ensure_signed(origin)?;

            //TODO: verify sender owns DID

            ensure!(
                Self::is_sequence_in_template(registry_id, template_id, sequence_id),
                Error::<T>::NotFound
            );
            //this also ensures template step exists
            ensure!(
                Self::is_attestor_template_step(registry_id, template_id, template_step_index, did),
                Error::<T>::NotAttestor
            );
            ensure!(
                Self::is_valid_template_step(
                    registry_id,
                    template_id,
                    template_step_index,
                    sequence_id
                ),
                Error::<T>::NotAttestor
            );

            let sequence_step = SequenceStep {
                attested_by: did,
                attributes,
            };

            <SequenceSteps<T>>::insert(
                (registry_id, template_id, sequence_id),
                template_step_index,
                sequence_step,
            );

            Self::deposit_event(Event::SequenceStepCreated(
                registry_id,
                template_id,
                sequence_id,
                template_step_index,
            ));
            Ok(().into())
        }

        // /// Update a new sequence_step
        // ///
        // /// Arguments: none
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn update_sequence_step(origin,registry_id: T::RegistryId,template_id: T::TemplateId,sequence_id:T::SequenceId,sequence_step_id: T::SequenceStepId) {
        //     let sender = ensure_signed(origin)?;

        //     //TODO:is attestor
        //     ensure!(Self::is_sequence_step_in_sequence(registry_id,template_id,sequence_id,sequence_step_id), Error::<T>::NotFound);

        //     //TODO:update

        //     Self::deposit_event(Event::SequenceStepUpdated(registry_id,template_id,sequence_id,sequence_step_id));
        // }

        // /// Remove a sequence_step
        // ///
        // /// Arguments:
        // /// - `sequence_step_id` SequenceStep to be removed
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn remove_sequence_step(origin, registry_id: T::RegistryId,template_id: T::TemplateId,sequence_id:T::SequenceId,sequence_step_id: T::SequenceStepId) {
        //     let sender = ensure_signed(origin)?;

        //    //TODO:is attestor
        //     ensure!(Self::is_sequence_step_in_sequence(registry_id,template_id,sequence_id,sequence_step_id), Error::<T>::NotFound);

        //     <SequenceSteps<T>>::remove((registry_id, template_id,sequence_id),sequence_step_id);

        //     Self::deposit_event(Event::SequenceStepRemoved(registry_id,template_id,sequence_id,sequence_step_id));
        // }
    }

    impl<T: Config> Module<T> {
        // -- private functions --

        fn is_registry_owner(account: &T::AccountId, registry_id: T::RegistryId) -> bool {
            <Registries<T>>::contains_key(account, registry_id)
        }

        fn is_template_in_registry(registry_id: T::RegistryId, template_id: T::TemplateId) -> bool {
            <Templates<T>>::contains_key(registry_id, template_id)
        }

        fn is_template_step_in_template(
            registry_id: T::RegistryId,
            template_id: T::TemplateId,
            template_step_index: TemplateStepIndex,
        ) -> bool {
            <TemplateSteps<T>>::contains_key((registry_id, template_id), template_step_index)
        }
        fn is_sequence_in_template(
            registry_id: T::RegistryId,
            template_id: T::TemplateId,
            sequence_id: T::SequenceId,
        ) -> bool {
            <Sequences<T>>::contains_key((registry_id, template_id), sequence_id)
        }

        // fn is_sequence_step_in_sequence(
        //     registry_id: T::RegistryId,
        //     template_id: T::TemplateId,
        //     sequence_id: T::SequenceId,
        //     sequence_step_id: TemplateStepIndex,
        // ) -> bool {
        //     <SequenceSteps<T>>::contains_key((registry_id, template_id, sequence_id), sequence_step_id)
        // }

        fn is_attestor_template_step(
            registry_id: T::RegistryId,
            template_id: T::TemplateId,
            template_step_index: TemplateStepIndex,
            did: Did,
        ) -> bool {
            <Attestors<T>>::contains_key((registry_id, template_id, template_step_index), did)
        }

        pub fn create_template_step(
            registry_id: T::RegistryId,
            template_id: T::TemplateId,
            template_step_index: TemplateStepIndex,
            template_step: TemplateStep,
            attestors: Vec<Attestor>,
        ) {
            <TemplateSteps<T>>::insert(
                (registry_id, template_id),
                template_step_index,
                template_step,
            );

            attestors.iter().for_each(|attestor| {
                <Attestors<T>>::insert(
                    (registry_id, template_id, template_step_index),
                    attestor.did,
                    attestor,
                )
            });
        }
        pub fn is_valid_template_step(
            registry_id: T::RegistryId,
            template_id: T::TemplateId,
            template_step_index: TemplateStepIndex,
            sequence_id: T::SequenceId,
        ) -> bool {
            //must not already exist
            if <SequenceSteps<T>>::contains_key(
                (registry_id, template_id, sequence_id),
                template_step_index,
            ) {
                return false;
            }
            template_step_index==0 ||
        //previous step must exist
        <SequenceSteps<T>>::contains_key(
            (registry_id, template_id, sequence_id),
            template_step_index - 1,
        )
        }
    }
}
