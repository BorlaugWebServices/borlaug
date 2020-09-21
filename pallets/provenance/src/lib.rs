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
        // <T as Trait>::TemplateId,
        // <T as Trait>::TemplateStepId,
        // <T as Trait>::SequenceId,
        // <T as Trait>::SequenceStepId,
    {
     /// A new Registry was created (RegistryId)
     RegistryCreated(RegistryId),
     /// A Registry was updated (RegistryId)
     RegistryUpdated(RegistryId),
     /// A Registry was Removed (RegistryId)
     RegistryRemoved(RegistryId),


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
        /// AccountId => Vec<RegistryId>
        pub Registry get(fn registries): map hasher(blake2_128_concat) T::AccountId => Vec<T::RegistryId>;

        /// The next available registry index
        pub NextRegistryId get(fn next_registry_id) config(): T::RegistryId;



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

            Self::deposit_event(RawEvent::RegistryCreated(registry_id));
        }

        /// Remove a registry
        ///
        /// Arguments:
        /// - `registry_id` Registry to be removed
        #[weight = 100_000]
        pub fn remove_registry(origin, registry_id: T::RegistryId) {
            let sender = ensure_signed(origin)?;

            ensure!(
                    if <Registry<T>>::contains_key(sender.clone()) {
                        let registryIds = <Registry<T>>::get(&sender);
                        registryIds.contains(&registry_id)
                    } else {
                        false
                    }
                ,
                Error::<T>::NotFound
            );

            <Registry<T>>::mutate(&sender, |registrys| {
                registrys.retain(|cid| *cid != registry_id)
            });

            Self::deposit_event(RawEvent::RegistryRemoved(registry_id));
        }

    }
}

impl<T: Trait> Module<T> {
    // -- private functions --

    fn next_nonce() -> u64 {
        let nonce = <Nonce>::get();
        <Nonce>::mutate(|n| *n += 1u64);
        nonce
    }
}
