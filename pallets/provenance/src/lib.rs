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

mod mock;
mod tests;

// use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure, IterableStorageDoubleMap, Parameter,
    StorageDoubleMap,
};
use frame_system::ensure_signed;
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

pub trait Trait: frame_system::Trait + timestamp::Trait {
    type RegistryId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
    type TemplateId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
    type SequenceId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

pub type TemplateStepIndex = u8;

decl_event!(
    pub enum Event<T>
    where
        <T as Trait>::RegistryId,
        <T as Trait>::TemplateId,
        <T as Trait>::SequenceId,
    {
     /// A new Registry was created (RegistryId)
     RegistryCreated(RegistryId),
     /// A Registry was Removed (RegistryId)
     RegistryRemoved(RegistryId),
     /// A new Template was created (RegistryId)
     TemplateCreated(RegistryId,TemplateId),
     /// A Template was Removed (RegistryId,TemplateId)
     TemplateRemoved(RegistryId,TemplateId),
    /// A TemplateStep was Removed (RegistryId,TemplateId,TemplateStepIndex)
    TemplateStepUpdated(RegistryId,TemplateId,TemplateStepIndex),
    /// A new Sequence was created (RegistryId,TemplateId,SequenceId)
    SequenceCreated(RegistryId,TemplateId,SequenceId),
    /// A Sequence was Removed (RegistryId,TemplateId,SequenceId)
    SequenceUpdated(RegistryId,TemplateId,SequenceId),
    /// A Sequence was Removed (RegistryId,TemplateId,SequenceId)
    SequenceRemoved(RegistryId,TemplateId,SequenceId),
    /// A new SequenceStep was created (RegistryId,TemplateId,TemplateStepIndex,SequenceId,TemplateStepIndex)
    SequenceStepCreated(RegistryId,TemplateId,TemplateStepIndex,SequenceId,TemplateStepIndex),

    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
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
        NoIdAvailable

    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Provenance {

        /// Incrementing nonce
        pub Nonce get(fn nonce) build(|_| 1u64): u64;

        /// An account can have multiple Regitries of process templates
        /// (T::AccountId,T::RegistryId) => T::RegistryId
        pub Registries get(fn registries):
            double_map hasher(blake2_128_concat)  T::AccountId, hasher(blake2_128_concat) T::RegistryId => T::RegistryId;

        /// A Registry can have multiple process Templates
        /// (T::RegistryId,T::TemplateId) => Template
        pub Templates get(fn templates):
            double_map hasher(blake2_128_concat) T::RegistryId,  hasher(blake2_128_concat) T::TemplateId => Template;

        /// A Process has multiple steps
        /// (T::RegistryId,T::TemplateId), u8 => TemplateStep
        pub TemplateSteps get(fn template_steps):
            double_map hasher(blake2_128_concat) (T::RegistryId, T::TemplateId), hasher(blake2_128_concat) u8 => TemplateStep;


        /// A Template step may have multiple attestors
        /// (T::RegistryId,T::TemplateId,TemplateStepIndex,Did)=> Attestor
        pub Attestors get(fn attestors):
            double_map hasher(blake2_128_concat) (T::RegistryId, T::TemplateId,TemplateStepIndex), hasher(blake2_128_concat) Did => Attestor;

        /// A process Template can have multiple process Sequences
        /// (T::RegistryId,T::TemplateId), T::SequenceId => T::SequenceId
        pub Sequences get(fn sequences):
            double_map hasher(blake2_128_concat) (T::RegistryId, T::TemplateId), hasher(blake2_128_concat) T::SequenceId => Sequence;

        /// A Sequence can have multiple process Sequence Steps
        /// (T::RegistryId,T::TemplateId,T::SequenceId), TemplateStepIndex => SequenceStep
        pub SequenceSteps get(fn sequence_steps):
            double_map hasher(blake2_128_concat) (T::RegistryId,T::TemplateId,T::SequenceId), hasher(blake2_128_concat) TemplateStepIndex => SequenceStep;

        /// The next available registry index
        pub NextRegistryId get(fn next_registry_id) config(): T::RegistryId;

         /// The next available template index
         pub NextTemplateId get(fn next_template_id) config(): T::TemplateId;

        /// The next available sequence index
        pub NextSequenceId get(fn next_sequence_id) config(): T::SequenceId;

    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Add a new registry
        ///
        /// Arguments: none
        #[weight = 100_000]
        pub fn create_registry(origin) {
            let sender = ensure_signed(origin)?;

            let registry_id = Self::next_registry_id();
            let next_id = registry_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextRegistryId<T>>::put(next_id);

            <Registries<T>>::insert(&sender, &registry_id, registry_id);

            Self::deposit_event(RawEvent::RegistryCreated(registry_id));
        }

        /// Remove a registry
        ///
        /// Arguments:
        /// - `registry_id` Registry to be removed
        #[weight = 100_000]
        pub fn remove_registry(origin, registry_id: T::RegistryId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_registry_owner(&sender,registry_id), Error::<T>::NotFound);

            <Templates<T>>::drain_prefix(registry_id).for_each(|(template_id,_template)|{
                <TemplateSteps<T>>::remove_prefix((registry_id,template_id));
                <Sequences<T>>::drain_prefix((registry_id,template_id)).for_each(|(sequence_id,_sequence)|{
                    <SequenceSteps<T>>::remove_prefix((registry_id,template_id,sequence_id));
                });
            });

            <Registries<T>>::remove(sender, registry_id);

            Self::deposit_event(RawEvent::RegistryRemoved(registry_id));
        }

        /// Add a new template
        ///
        /// Arguments: none
        #[weight = 100_000]
        pub fn create_template(origin,registry_id: T::RegistryId,name:Vec<u8>,template_steps:Vec<(TemplateStep,Vec<Attestor>)>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_registry_owner(&sender,registry_id), Error::<T>::NotFound);

            let template_id = Self::next_template_id();
            let next_id = template_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextTemplateId<T>>::put(next_id);

            let template=Template{
                name
            };

            <Templates<T>>::insert(registry_id, template_id, template);


            template_steps.into_iter().enumerate().for_each(|(template_step_index,(template_step,attestors))|{
                let template_step_index= UniqueSaturatedInto::<TemplateStepIndex>::unique_saturated_into(template_step_index);
                Self::create_template_step(registry_id,template_id,template_step_index,template_step,attestors);
            });

            Self::deposit_event(RawEvent::TemplateCreated(registry_id,template_id));
        }

        /// Remove a template
        ///
        /// Arguments:
        /// - `template_id` Template to be removed
        #[weight = 100_000]
        pub fn remove_template(origin, registry_id: T::RegistryId,template_id: T::TemplateId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_registry_owner(&sender,registry_id), Error::<T>::NotFound);
            ensure!(Self::is_template_in_registry(registry_id,template_id), Error::<T>::NotFound);

            <TemplateSteps<T>>::remove_prefix((registry_id,template_id));
            <Sequences<T>>::drain_prefix((registry_id,template_id)).for_each(|(sequence_id,_sequence)|{
                <SequenceSteps<T>>::remove_prefix((registry_id,template_id,sequence_id));
            });

            <Templates<T>>::remove(registry_id,template_id);

            Self::deposit_event(RawEvent::TemplateRemoved(registry_id,template_id));
        }

        /// Update a new template_step
        ///
        /// Arguments: none
        #[weight = 100_000]
        pub fn update_template_step(origin,registry_id: T::RegistryId,template_id: T::TemplateId,template_step_index: TemplateStepIndex, add_attestors:Option<Vec<Attestor>>, remove_attestors:Option<Vec<Attestor>>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_registry_owner(&sender,registry_id), Error::<T>::NotFound);
            ensure!(Self::is_template_step_in_template(registry_id,template_id,template_step_index), Error::<T>::NotFound);

            remove_attestors.and_then(|remove_attestors|
                {
                    remove_attestors.iter().for_each(|attestor| {
                <Attestors<T>>::remove((registry_id,template_id,template_step_index),attestor.did);
            });Some(())
        });

            add_attestors.and_then(|add_attestors|{add_attestors.iter().for_each(|attestor| {
                <Attestors<T>>::insert((registry_id,template_id,template_step_index),attestor.did,attestor);
            });
            Some(())});

            Self::deposit_event(RawEvent::TemplateStepUpdated(registry_id,template_id,template_step_index));
        }

        /// Add a new sequence
        ///
        /// Arguments: none
        #[weight = 100_000]
        pub fn create_sequence(origin,did:Did,registry_id: T::RegistryId,template_id: T::TemplateId,name:Vec<u8>) {
            let _sender = ensure_signed(origin)?;

            //TODO: verify sender owns DID

            ensure!(Self::is_template_in_registry(registry_id,template_id), Error::<T>::NotFound);
            ensure!(Self::is_attestor_template_step(registry_id,template_id,0,did), Error::<T>::NotAttestor);

            let sequence_id = Self::next_sequence_id();
            let next_id = sequence_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextSequenceId<T>>::put(next_id);

            let sequence=Sequence{
                name,
                status:SequenceStatus::InProgress
            };

            <Sequences<T>>::insert((registry_id,template_id), sequence_id, sequence);


            Self::deposit_event(RawEvent::SequenceCreated(registry_id,template_id,sequence_id));
        }

        /// Update a sequence
        ///
        /// Arguments: none
        // #[weight = 100_000]
        // pub fn update_sequence(origin,registry_id: T::RegistryId,template_id: T::TemplateId,sequence_id: T::SequenceId) {
        //     let sender = ensure_signed(origin)?;

        //     //TODO:is attestor
        //     ensure!(Self::is_sequence_in_template(registry_id,template_id,sequence_id), Error::<T>::NotFound);

        //     //TODO:update

        //     Self::deposit_event(RawEvent::SequenceUpdated(registry_id,template_id,sequence_id));
        // }

        /// Remove a sequence
        ///
        /// Arguments:
        /// - `sequence_id` Sequence to be removed
        #[weight = 100_000]
        pub fn remove_sequence(origin, registry_id: T::RegistryId,template_id: T::TemplateId,sequence_id: T::SequenceId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_registry_owner(&sender,registry_id), Error::<T>::NotFound);
            ensure!(Self::is_sequence_in_template(registry_id,template_id,sequence_id), Error::<T>::NotFound);

            <Sequences<T>>::remove((registry_id, template_id),sequence_id);
            <SequenceSteps<T>>::remove_prefix((registry_id, template_id,sequence_id));

            Self::deposit_event(RawEvent::SequenceRemoved(registry_id,template_id,sequence_id));
        }

        /// Add a new sequence_step
        ///
        /// Arguments: none
        #[weight = 100_000]
        pub fn create_sequence_step(origin,did:Did,registry_id: T::RegistryId,template_id: T::TemplateId,template_step_index: TemplateStepIndex,sequence_id:T::SequenceId, attributes:Vec<Attribute>) {
            let _sender = ensure_signed(origin)?;

            //TODO: verify sender owns DID

            ensure!(Self::is_sequence_in_template(registry_id,template_id,sequence_id), Error::<T>::NotFound);
            //this also ensures template step exists
            ensure!(Self::is_attestor_template_step(registry_id,template_id,template_step_index,did), Error::<T>::NotAttestor);
            ensure!(Self::is_valid_template_step(registry_id,template_id,template_step_index,sequence_id), Error::<T>::NotAttestor);

            let sequence_step=SequenceStep{
                attested_by:did,
                attributes
            };

            <SequenceSteps<T>>::insert((registry_id,template_id,sequence_id), template_step_index, sequence_step);

            Self::deposit_event(RawEvent::SequenceStepCreated(registry_id,template_id,template_step_index,sequence_id,template_step_index));
        }

        // /// Update a new sequence_step
        // ///
        // /// Arguments: none
        // #[weight = 100_000]
        // pub fn update_sequence_step(origin,registry_id: T::RegistryId,template_id: T::TemplateId,sequence_id:T::SequenceId,sequence_step_id: T::SequenceStepId) {
        //     let sender = ensure_signed(origin)?;

        //     //TODO:is attestor
        //     ensure!(Self::is_sequence_step_in_sequence(registry_id,template_id,sequence_id,sequence_step_id), Error::<T>::NotFound);

        //     //TODO:update

        //     Self::deposit_event(RawEvent::SequenceStepUpdated(registry_id,template_id,sequence_id,sequence_step_id));
        // }

        // /// Remove a sequence_step
        // ///
        // /// Arguments:
        // /// - `sequence_step_id` SequenceStep to be removed
        // #[weight = 100_000]
        // pub fn remove_sequence_step(origin, registry_id: T::RegistryId,template_id: T::TemplateId,sequence_id:T::SequenceId,sequence_step_id: T::SequenceStepId) {
        //     let sender = ensure_signed(origin)?;

        //    //TODO:is attestor
        //     ensure!(Self::is_sequence_step_in_sequence(registry_id,template_id,sequence_id,sequence_step_id), Error::<T>::NotFound);

        //     <SequenceSteps<T>>::remove((registry_id, template_id,sequence_id),sequence_step_id);

        //     Self::deposit_event(RawEvent::SequenceStepRemoved(registry_id,template_id,sequence_id,sequence_step_id));
        // }

    }
}

impl<T: Trait> Module<T> {
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
