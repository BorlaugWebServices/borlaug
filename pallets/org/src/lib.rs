//! # Org Module
//!
//! ## Overview
//!
//! TODO:
//!
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

    use codec::Encode;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Randomness,
    };
    use frame_system::pallet_prelude::*;
    use primitives::org::OrgGroup;
    use sp_runtime::traits::{AtLeast32Bit, CheckedAdd, One};
    use sp_std::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + timestamp::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::event]
    #[pallet::metadata(T::Moment = "Moment")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new Org was created (Controller)
        OrgCreated(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value was None
        NoneValue,
        /// A non-controller account attempted to
        NotController,
        /// Not authorized
        NotAuthorized,
        /// Id out of bounds
        NoIdAvailable,
    }

    #[pallet::type_value]
    pub fn UnitDefault<T: Config>() -> u64 {
        1u64
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    /// Incrementing nonce
    pub type Nonce<T> = StorageValue<_, u64, ValueQuery, UnitDefault<T>>;

    /// An account can have multiple DIDs
    /// AccountId => Vec<Did>
    #[pallet::storage]
    #[pallet::getter(fn dids)]
    pub type Org<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<OrgGroup>, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new Org
        ///
        /// # <weight>
        /// - O(1).
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_org(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            Ok(().into())
        }
    }
}
