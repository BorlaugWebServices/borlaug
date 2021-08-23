//! # Settings Module
//!
//! ## Overview
//!
//! This module allows the Council to change various chain settings including:
//! * The **Transaction Byte Fee**. This is the fee in GRAM per byte charged for extrinsics (in addition to other fees).
//! * The **Fee Split Ratio**. This is the proportion of fees that go to the Treasury vs the block author (validator owner).
//! * **Extrinsic Extras**. These are special fees that are charged for specific extrinsics.
//!   Typicaly they involve creating objects on chain such as Audits, Process Definitions etc.
//! * **Weight to Fee Coefficients**. These are the coeffients used for the Weight to Fee Polinomial.
//!   For more detail see: https://substrate.dev/recipes/fees.html
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `set_weight_to_fee_coefficients` - Set the **Weight to Fee Coefficients**.
//! * `set_transaction_byte_fee` - Set the **Transaction Byte Fee**.
//! * `set_fee_split_ratio` - Set the **Fee Split Ratio**.
//! * `set_extrinsic_extra` - Set an **Extrinsic Extra**.
//! * `remove_extrinsic_extra` - Set an **Extrinsic Extra**.
//!
//! ### RPC Methods
//!
//! * `get_weight_to_fee_coefficients` - Get the current **Weight to Fee Coefficients**.
//! * `get_transaction_byte_fee` - Get the current **Transaction Byte Fee**.
//! * `get_fee_split_ratio` - Get the current **Fee Split Ratio**.
//! * `get_extrinsic_extra` - Get a specific **Extrinsic Extra**.
//! * `get_extrinsic_extras` - Get all the current **Extrinsic Extras**.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    pub use super::weights::WeightInfo;
    use codec::Codec;
    use extrinsic_extra::GetExtrinsicExtra;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*,
        traits::{Currency, Get},
        weights::{WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial},
    };
    use frame_system::pallet_prelude::*;
    use smallvec::SmallVec;
    use sp_runtime::{
        traits::{AtLeast32BitUnsigned, UniqueSaturatedInto},
        Perbill,
    };
    use sp_std::{fmt::Debug, marker::PhantomData, prelude::*};

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The origin which can change settings
        type ChangeSettingOrigin: EnsureOrigin<Self::Origin>;

        type Currency: Currency<Self::AccountId>;

        /// Unique identifier for each module
        type ModuleIndex: Parameter
            + Member
            + PartialEq
            + MaybeSerializeDeserialize
            + From<u8>
            + Copy;
        /// A Unique identifier for each extrinsic within a module
        type ExtrinsicIndex: Parameter
            + Member
            + PartialEq
            + MaybeSerializeDeserialize
            + From<u8>
            + Copy;

        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + Into<<Self::Currency as Currency<Self::AccountId>>::Balance>;
    }

    #[pallet::event]
    #[pallet::metadata(
        T::Moment = "Moment",
        T::Hash = "Hash",
        T::AccountId = "AccountId",
        T::ModuleIndex = "ModuleIndex",
        T::ExtrinsicIndex = "ExtrinsicIndex",
        T::Balance = "Balance"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// WeightToFeePolinomialCoefficients were updated
        ///
        WeightToFeePolinomialCoefficientsUpdated(),
        /// Transaction byte fee was updated
        /// (byte_fee)
        TransactionByteFeeUpdated(T::Balance),
        /// Fee split ratio was updated
        /// (ratio)
        FeeSplitRatioUpdated(u32),
        /// Extrinsic Extra was updated
        /// (module_index, extrinsic_index, fee)
        ExtrinsicExtraUpdated(T::ModuleIndex, T::ExtrinsicIndex, T::Balance),
        /// Extrinsic Extra was removed
        /// (module_index, extrinsic_index)
        ExtrinsicExtraRemoved(T::ModuleIndex, T::ExtrinsicIndex),
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
        pub transaction_byte_fee: T::Balance,
        pub fee_split_ratio: u32,
        pub extrinisic_extra: Vec<(T::ModuleIndex, Vec<(T::ExtrinsicIndex, T::Balance)>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                transaction_byte_fee: 10_000u32.into(),
                fee_split_ratio: 80,
                extrinisic_extra: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::initialize_weight_to_fee_coefficients(vec![WeightToFeeCoefficient {
                coeff_integer: 1u32.into(),
                coeff_frac: Perbill::zero(),
                negative: false,
                degree: 1,
            }]);
            Pallet::<T>::initialize_transaction_byte_fee(self.transaction_byte_fee);
            Pallet::<T>::initialize_fee_split_ratio(self.fee_split_ratio);
            Pallet::<T>::initialize_extrinisic_extra(self.extrinisic_extra.clone());
        }
    }

    /// The coefficients used for the WeightToFeePolynomial when calculating fees from weights
    #[pallet::storage]
    #[pallet::getter(fn weight_to_fee_coefficients)]
    pub(super) type WeightToFeePolinomialCoefficients<T: Config> =
        StorageValue<_, Vec<WeightToFeeCoefficient<T::Balance>>, ValueQuery>;

    /// The fee charged per byte for extrinsics (added to weight and fixed fees)   
    #[pallet::storage]
    #[pallet::getter(fn transaction_byte_fee)]
    pub(super) type TransactionByteFee<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

    /// Ratio of fees to be split between Treasury and Author  value stored is percentage to go to Treasury    
    #[pallet::storage]
    #[pallet::getter(fn fee_split_ratio)]
    pub(super) type FeeSplitRatio<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Special fees to be added to specific extrinsics
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
        /// Change the weight to fee coefficents used to build the polynomial for calcualting weight to fee
        ///
        /// Arguments:
        /// - `new_coefficents` new polinomial coefficients
        #[pallet::weight(<T as Config>::WeightInfo::set_weight_to_fee_coefficients(
            new_coefficents.len() as u32
        ))]
        pub fn set_weight_to_fee_coefficients(
            origin: OriginFor<T>,
            new_coefficents: Vec<(T::Balance, Perbill, bool, u8)>,
        ) -> DispatchResultWithPostInfo {
            T::ChangeSettingOrigin::ensure_origin(origin)?;

            let new_coefficents: Vec<WeightToFeeCoefficient<T::Balance>> = new_coefficents
                .into_iter()
                .map(
                    |(coeff_integer, coeff_frac, negative, degree)| WeightToFeeCoefficient {
                        coeff_integer,
                        coeff_frac,
                        negative,
                        degree,
                    },
                )
                .collect();

            <WeightToFeePolinomialCoefficients<T>>::put(new_coefficents);

            Self::deposit_event(Event::WeightToFeePolinomialCoefficientsUpdated());

            Ok(().into())
        }

        /// Change the Transaction Byte Fee. The fee in GRAM per byte charged for extrinsics (in addition to other fees).
        ///
        /// Arguments:
        /// - `new_fee` new transaction byte fee
        #[pallet::weight(<T as Config>::WeightInfo::set_transaction_byte_fee())]
        pub fn set_transaction_byte_fee(
            origin: OriginFor<T>,
            new_fee: T::Balance,
        ) -> DispatchResultWithPostInfo {
            T::ChangeSettingOrigin::ensure_origin(origin)?;

            <TransactionByteFee<T>>::put(new_fee);

            Self::deposit_event(Event::TransactionByteFeeUpdated(new_fee));

            Ok(().into())
        }

        /// Change the fee split ratio (specify percentage of fee to go to Treasury. Remaining goes to Author)
        ///
        /// Arguments:
        /// - `new_ratio` new Fee Split Ratio as an integer 0..100 inclusive. This number represents the percentage going to the Treasury. Remainder goes to block author.
        #[pallet::weight(<T as Config>::WeightInfo::set_fee_split_ratio())]
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

        /// Set an Extrinsic Extra - a special fee for specific extrinsics in addition to usual extrinsic fees.
        ///
        /// Arguments:
        /// - `module_index` module of the extrinsic. See module code for indicies.
        /// - `extrinsic_index` index of the extrinsic within the module. See module code for indicies.
        /// - `extra` fee to be charged for calling the extrinsic.
        #[pallet::weight(<T as Config>::WeightInfo::set_extrinsic_extra())]
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
        /// Remove an Extrinsic Extra
        ///
        /// Arguments:
        /// - `module_index` module of the extrinsic. See module code for indicies.
        /// - `extrinsic_index` index of the extrinsic within the module. See module code for indicies.
        #[pallet::weight(<T as Config>::WeightInfo::remove_extrinsic_extra())]
        pub fn remove_extrinsic_extra(
            origin: OriginFor<T>,
            module_index: T::ModuleIndex,
            extrinsic_index: T::ExtrinsicIndex,
        ) -> DispatchResultWithPostInfo {
            T::ChangeSettingOrigin::ensure_origin(origin)?;

            <ExtrinsicExtra<T>>::remove(&module_index, &extrinsic_index);

            Self::deposit_event(Event::ExtrinsicExtraRemoved(module_index, extrinsic_index));

            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        // -- rpc api functions --

        pub fn get_weight_to_fee_coefficients() -> Vec<(u64, Perbill, bool, u8)> {
            let weight_to_fee_coefficients = <WeightToFeePolinomialCoefficients<T>>::get();
            weight_to_fee_coefficients
                .into_iter()
                .map(|weight_to_fee_coefficient| {
                    (
                        weight_to_fee_coefficient
                            .coeff_integer
                            .unique_saturated_into(),
                        weight_to_fee_coefficient.coeff_frac,
                        weight_to_fee_coefficient.negative,
                        weight_to_fee_coefficient.degree,
                    )
                })
                .collect()
        }

        pub fn get_transaction_byte_fee() -> T::Balance {
            <TransactionByteFee<T>>::get()
        }

        pub fn get_fee_split_ratio() -> u32 {
            <FeeSplitRatio<T>>::get()
        }

        pub fn get_extrinsic_extra(
            module_index: T::ModuleIndex,
            extrinsic_index: T::ExtrinsicIndex,
        ) -> Option<T::Balance> {
            <ExtrinsicExtra<T>>::get(module_index, extrinsic_index)
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
        fn initialize_weight_to_fee_coefficients(
            weight_to_fee_coefficients: Vec<WeightToFeeCoefficient<T::Balance>>,
        ) {
            <WeightToFeePolinomialCoefficients<T>>::put(weight_to_fee_coefficients);
        }
        fn initialize_transaction_byte_fee(transaction_byte_fee: T::Balance) {
            <TransactionByteFee<T>>::put(transaction_byte_fee);
        }

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

        fn charge_extrinsic_extra(
            module_index: &T::ModuleIndex,
            extrinsic_index: &T::ExtrinsicIndex,
            account: &T::AccountId,
        ) {
            match <ExtrinsicExtra<T>>::get(module_index, extrinsic_index) {
                Some(fee) => {
                    let (_deducted, _) = T::Currency::slash(&account, fee.into());
                }
                None => (),
            }
        }
    }

    impl<T: Config> GetExtrinsicExtra for Module<T> {
        type ModuleIndex = T::ModuleIndex;
        type ExtrinsicIndex = T::ExtrinsicIndex;
        type AccountId = T::AccountId;

        fn charge_extrinsic_extra(
            module_index: &Self::ModuleIndex,
            extrinsic_index: &Self::ExtrinsicIndex,
            account: &T::AccountId,
        ) {
            Self::charge_extrinsic_extra(module_index, extrinsic_index, account)
        }
    }

    pub struct TransactionByteFeeGet<T: Config>(PhantomData<T>);

    impl<T: Config> Get<T::Balance> for TransactionByteFeeGet<T> {
        fn get() -> T::Balance {
            <TransactionByteFee<T>>::get()
        }
    }

    /// Implementor of `WeightToFeePolynomial` that can be changed in the settings pallet.
    pub struct CustomizableFee<T: Config>(sp_std::marker::PhantomData<T>);

    impl<T: Config> WeightToFeePolynomial for CustomizableFee<T> {
        type Balance = <T as Config>::Balance;

        fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
            let weight_to_fee_coefficients = <WeightToFeePolinomialCoefficients<T>>::get();

            let mut vec = SmallVec::new();
            weight_to_fee_coefficients
                .into_iter()
                .for_each(|coefficient| {
                    vec.push(coefficient);
                });
            vec
        }
    }
}
