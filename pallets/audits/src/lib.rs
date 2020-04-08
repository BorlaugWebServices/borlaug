//! # Audit Module
//!
//! ## Overview
//!
//! An audit
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For general users
//! * `create_audit` - Creates a new audit

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod tests;

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure, weights::SimpleDispatchInfo,
    Parameter,
};
use frame_system::{self as system, ensure_signed};

use sp_runtime::{
    traits::{AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One},
    DispatchResult,
};
use sp_std::prelude::*;

pub trait Trait: frame_system::Trait + timestamp::Trait {
    type AuditId: Parameter
        + Member
        + AtLeast32Bit
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + PartialEq;

   
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
        where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::AuditId,
     
    {
        /// New registry created (owner, registry id)
        AuditCreated(AccountId, AuditId),
        
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Value was None
        NoneValue,
        NoIdAvailable
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Audits {

        /// Incrementing nonce
        pub Nonce get(fn nonce) build(|_| 1u64): u64;

        /// The next available audit index
        pub NextAuditId get(fn next_audit_id) config(): T::AuditId;

        /// P
        pub Audits get(fn audits):
            map hasher(blake2_128_concat) T::AccountId => Vec<T::AuditId>;



    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Create a new registry
        ///
        /// Arguments: None

        #[weight = SimpleDispatchInfo::FixedNormal(100_000)]
        fn create_audit(origin) {
            let sender = ensure_signed(origin)?;

            let audit_id = Self::next_audit_id();
            let next_id = audit_id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoIdAvailable)?;
            <NextAuditId<T>>::put(next_id);

            <Audits<T>>::append_or_insert(&sender, &[&audit_id][..]);

            Self::deposit_event(RawEvent::AuditCreated(sender, audit_id));
        }
    }
}

// private functions
impl<T: Trait> Module<T> {}
