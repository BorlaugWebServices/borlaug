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

    use codec::Codec;
    use extrinsic_extra::GetExtrinsicExtra;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::AtLeast32BitUnsigned;
    use sp_std::fmt::Debug;
    use sp_std::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The origin which can change settings
        type ChangeSettingOrigin: EnsureOrigin<Self::Origin>;

        /// Unique identifier for each module
        type ModuleIndex: Parameter + Member + PartialEq + MaybeSerializeDeserialize;
        /// A Unique identifier for each extrinsic within a module
        type ExtrinsicIndex: Parameter + Member + PartialEq + MaybeSerializeDeserialize;

        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug;
    }

    #[pallet::event]
    #[pallet::metadata(
        T::Moment = "Moment",
        T::Hash = "Hash",
        T::AccountId = "AccountId",
        //TODO: are these names safe?
        T::ModuleIndex = "ModuleIndex",
        T::ExtrinsicIndex = "ExtrinsicIndex",
        T::Balance = "Balance"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Fee split ratio was updated
        FeeSplitRatioUpdated(u32),
        /// Extrinsic extra was updated
        ExtrinsicExtraUpdated(T::ModuleIndex, T::ExtrinsicIndex, T::Balance),
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
        pub extrinisic_extra: Vec<(T::ModuleIndex, Vec<(T::ExtrinsicIndex, T::Balance)>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                fee_split_ratio: 80,
                extrinisic_extra: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::initialize_fee_split_ratio(self.fee_split_ratio);
            Pallet::<T>::initialize_extrinisic_extra(self.extrinisic_extra.clone());
        }
    }

    /// Ratio of fees to be split between Treasury and Author  value stored is percentage to go to Treasury    
    #[pallet::storage]
    #[pallet::getter(fn fee_split_ratio)]
    pub(super) type FeeSplitRatio<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Cost to be added to extrinsics
    #[pallet::storage]
    #[pallet::getter(fn extrinsic_extra)]
    pub(super) type ExtrinsicExtra<T: Config> = StorageDoubleMap<
        _,
        Identity,
        T::ModuleIndex,
        Identity,
        T::ExtrinsicIndex,
        T::Balance,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Change the fee split ratio (specify percentage of fee to go to Treasury. Remaining goes to Author)
        ///
        /// # <weight>
        /// TODO:
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_fee_split_ratio(
            origin: OriginFor<T>,
            new_ratio: u32,
        ) -> DispatchResultWithPostInfo {
            T::ChangeSettingOrigin::ensure_origin(origin)?;

            assert!(new_ratio <= 100, "Invalid fee_split_ratio");

            <FeeSplitRatio<T>>::put(new_ratio);

            Self::deposit_event(Event::FeeSplitRatioUpdated(new_ratio));

            Ok(().into())
        }

        /// Change the fee split ratio (specify percentage of fee to go to Treasury. Remaining goes to Author)
        ///
        /// # <weight>
        /// TODO:
        /// # </weight>
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_extrinsic_extra(
            origin: OriginFor<T>,
            module_index: T::ModuleIndex,
            extrinsic_index: T::ExtrinsicIndex,
            extra: T::Balance,
        ) -> DispatchResultWithPostInfo {
            T::ChangeSettingOrigin::ensure_origin(origin)?;

            <ExtrinsicExtra<T>>::insert(&module_index, &extrinsic_index, extra);

            Self::deposit_event(Event::ExtrinsicExtraUpdated(
                module_index,
                extrinsic_index,
                extra,
            ));

            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --

        pub fn get_fee_split_ratio() -> u32 {
            <FeeSplitRatio<T>>::get()
        }

        pub fn get_extrinsic_extras() -> Vec<(T::ModuleIndex, Vec<(T::ExtrinsicIndex, T::Balance)>)>
        {
            let mut extrinsic_extras: Vec<(T::ModuleIndex, Vec<(T::ExtrinsicIndex, T::Balance)>)> =
                Vec::new();
            <ExtrinsicExtra<T>>::iter().for_each(|(module_index, extrinsic_index, extra)| {
                if let Some((_, extrinsics)) = extrinsic_extras
                    .iter_mut()
                    .find(|(m, _)| *m == module_index)
                {
                    extrinsics.push((extrinsic_index, extra));
                } else {
                    extrinsic_extras.push((module_index, vec![(extrinsic_index, extra)]));
                }
            });
            extrinsic_extras
        }

        // -- private functions --
        fn initialize_fee_split_ratio(fee_split_ratio: u32) {
            assert!(fee_split_ratio <= 100, "Invalid fee_split_ratio");
            <FeeSplitRatio<T>>::put(fee_split_ratio);
        }

        fn initialize_extrinisic_extra(
            extrinisic_extra: Vec<(T::ModuleIndex, Vec<(T::ExtrinsicIndex, T::Balance)>)>,
        ) {
            extrinisic_extra
                .into_iter()
                .for_each(|(module, extrinsic_extras)| {
                    extrinsic_extras.into_iter().for_each(|(extrinsic, extra)| {
                        <ExtrinsicExtra<T>>::insert(&module, &extrinsic, extra)
                    })
                })
        }

        fn get_extrinsic_extra(
            module_index: &T::ModuleIndex,
            extrinsic_index: &T::ExtrinsicIndex,
        ) -> T::Balance {
            <ExtrinsicExtra<T>>::get(module_index, extrinsic_index).unwrap_or_else(|| 0u32.into())
        }
    }

    impl<T: Config> GetExtrinsicExtra for Module<T> {
        type ModuleIndex = T::ModuleIndex;
        type ExtrinsicIndex = T::ExtrinsicIndex;
        type Balance = T::Balance;

        fn get_extrinsic_extra(
            module_index: &Self::ModuleIndex,
            extrinsic_index: &Self::ExtrinsicIndex,
        ) -> T::Balance {
            Self::get_extrinsic_extra(module_index, extrinsic_index)
        }
    }
}
