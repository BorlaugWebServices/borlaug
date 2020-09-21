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

use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure, Parameter, StorageMap,
};
use frame_system::{self as system, ensure_signed};
use primitives::did::Did;
#[cfg(not(feature = "std"))]
use sp_io::hashing::blake2_256;
use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, One};
use sp_std::prelude::*;

pub trait Trait: frame_system::Trait + timestamp::Trait {
    type RegistryId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
    type TemplateId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
    type TemplateStepId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
    type SequenceId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
    type SequenceStepId: Parameter + AtLeast32Bit + Default + Copy + PartialEq;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        // <T as frame_system::Trait>::AccountId,
        // <T as timestamp::Trait>::Moment,
        <T as Trait>::RegistryId,
        <T as Trait>::TemplateId,
        <T as Trait>::TemplateStepId,
        // <T as Trait>::SequenceId,
        // <T as Trait>::SequenceStepId,
    {
     /// A new Registry was created (RegistryId)
     RegistryCreated(RegistryId),
     /// A Registry was Removed (RegistryId)
     RegistryRemoved(RegistryId),
     /// A new Template was created (TemplateId)
     TemplateCreated(TemplateId),
     /// A Template was Removed (TemplateId)
     TemplateRemoved(TemplateId),
    /// A new Template was created (TemplateStepId)
    TemplateStepCreated(TemplateStepId),
    /// A Template was Removed (TemplateStepId)
    TemplateStepUpdated(TemplateStepId),
    /// A Template was Removed (TemplateStepId)
    TemplateStepRemoved(TemplateStepId),


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
        pub Registry get(fn registries):
            double_map hasher(blake2_128_concat)  T::AccountId, hasher(blake2_128_concat) T::RegistryId => T::RegistryId;

        /// A Registry can have multiple process Templates
        /// (T::RegistryId,T::TemplateId) => T::TemplateId
        pub Template get(fn templates):
            double_map hasher(blake2_128_concat) T::RegistryId,  hasher(blake2_128_concat) T::TemplateId => T::TemplateId;

        /// A Process has multiple steps
        /// (T::RegistryId,T::TemplateId), T::TemplateStepId => T::TemplateStepId
        pub TemplateStep get(fn template_steps):
        double_map hasher(blake2_128_concat) (T::RegistryId, T::TemplateId), hasher(blake2_128_concat) T::TemplateStepId => T::TemplateStepId;

        /// The next available registry index
        pub NextRegistryId get(fn next_registry_id) config(): T::RegistryId;

         /// The next available template index
         pub NextTemplateId get(fn next_template_id) config(): T::TemplateId;

         /// The next available template step index
         pub NextTemplateStepId get(fn next_template_step_id) config(): T::TemplateStepId;

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

            <Registry<T>>::insert(&sender, &registry_id, registry_id);

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

            <Registry<T>>::remove(sender, registry_id);

            Self::deposit_event(RawEvent::RegistryRemoved(registry_id));
        }

        /// Add a new template
        ///
        /// Arguments: none
        #[weight = 100_000]
        pub fn create_template(origin,registry_id: T::RegistryId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_registry_owner(&sender,registry_id), Error::<T>::NotFound);

            let template_id = Self::next_template_id();
            let next_id = template_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextTemplateId<T>>::put(next_id);

            <Template<T>>::insert(registry_id, template_id, template_id);

            Self::deposit_event(RawEvent::TemplateCreated(template_id));
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

            <Template<T>>::remove(registry_id, template_id);

            Self::deposit_event(RawEvent::TemplateRemoved(template_id));
        }

        /// Add a new template_step
        ///
        /// Arguments: none
        #[weight = 100_000]
        pub fn create_template_step(origin,registry_id: T::RegistryId,template_id: T::TemplateId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_registry_owner(&sender,registry_id), Error::<T>::NotFound);
            ensure!(Self::is_template_in_registry(registry_id,template_id), Error::<T>::NotFound);

            let template_step_id = Self::next_template_step_id();
            let next_id = template_step_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextTemplateStepId<T>>::put(next_id);

            <TemplateStep<T>>::insert((registry_id,template_id), template_step_id, template_step_id);

            //TODO:attestors

            Self::deposit_event(RawEvent::TemplateStepCreated(template_step_id));
        }

        /// Update a new template_step
        ///
        /// Arguments: none
        #[weight = 100_000]
        pub fn update_template_step(origin,registry_id: T::RegistryId,template_id: T::TemplateId,template_step_id: T::TemplateStepId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_registry_owner(&sender,registry_id), Error::<T>::NotFound);
            ensure!(Self::is_template_in_registry(registry_id,template_id), Error::<T>::NotFound);
            ensure!(Self::is_template_step_in_template(registry_id,template_id,template_step_id), Error::<T>::NotFound);

            //TODO:attestors

            Self::deposit_event(RawEvent::TemplateStepUpdated(template_step_id));
        }

        /// Remove a template_step
        ///
        /// Arguments:
        /// - `template_step_id` TemplateStep to be removed
        #[weight = 100_000]
        pub fn remove_template_step(origin, registry_id: T::RegistryId,template_id: T::TemplateId,template_step_id: T::TemplateStepId) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_registry_owner(&sender,registry_id), Error::<T>::NotFound);
            ensure!(Self::is_template_in_registry(registry_id,template_id), Error::<T>::NotFound);
            ensure!(Self::is_template_step_in_template(registry_id,template_id,template_step_id), Error::<T>::NotFound);

            <TemplateStep<T>>::remove((registry_id, template_id),template_step_id);

            Self::deposit_event(RawEvent::TemplateStepRemoved(template_step_id));
        }

    }
}

impl<T: Trait> Module<T> {
    // -- private functions --

    fn is_registry_owner(account: &T::AccountId, registry_id: T::RegistryId) -> bool {
        <Registry<T>>::contains_key(account, registry_id)
    }

    fn is_template_in_registry(registry_id: T::RegistryId, template_id: T::TemplateId) -> bool {
        <Template<T>>::contains_key(registry_id, template_id)
    }
    fn is_template_step_in_template(
        registry_id: T::RegistryId,
        template_id: T::TemplateId,
        template_step_id: T::TemplateStepId,
    ) -> bool {
        <TemplateStep<T>>::contains_key((registry_id, template_id), template_step_id)
    }
}
