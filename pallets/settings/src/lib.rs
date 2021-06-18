//! # Settings Module
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

    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;
    use sp_std::marker::PhantomData;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The origin which can change settings
        type ChangeSettingOrigin: EnsureOrigin<Self::Origin>;
    }

    #[pallet::event]
    #[pallet::metadata(T::Moment = "Moment", T::Hash = "Hash", T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A setting was changed
        FeeSplitRatioUpdated(T::AccountId, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        NotAuthorized,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub fee_split_ratio: u32,
        pub _phantom: PhantomData<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                fee_split_ratio: 80,
                _phantom: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::initialize_fee_split_ratio(self.fee_split_ratio);
        }
    }

    /// Ratio of fees to be split between Treasury and Author  value stored is percentage to go to Treasury    
    #[pallet::storage]
    #[pallet::getter(fn fee_split_ratio)]
    pub(super) type FeeSplitRatio<T: Config> = StorageValue<_, u32>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Change a setting
        ///
        /// # <weight>
        /// TODO:
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_fee_split_ratio(
            origin: OriginFor<T>,
            new_ratio: u32,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            assert!(new_ratio <= 100, "Invalid fee_split_ratio");

            <FeeSplitRatio<T>>::put(new_ratio);

            Self::deposit_event(Event::FeeSplitRatioUpdated(sender, new_ratio));

            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --

        // -- private functions --
        fn initialize_fee_split_ratio(fee_split_ratio: u32) {
            assert!(fee_split_ratio <= 100, "Invalid fee_split_ratio");
            <FeeSplitRatio<T>>::put(fee_split_ratio);
        }
    }
}
