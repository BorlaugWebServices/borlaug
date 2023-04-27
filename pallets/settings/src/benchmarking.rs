//! Settings pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::sp_runtime::Perbill;
use frame_system::{self, RawOrigin as SystemOrigin};
use sp_std::{prelude::*, vec};

#[allow(unused)]
use crate::Pallet as SettingsPallet;

benchmarks! {
    set_weight_to_fee_coefficients {
        let a in 1 .. 10;

        let caller:T::AccountId = whitelisted_caller();

        let mut new_coefficents=vec![];
        for i in 0..a {
            new_coefficents.push((10u32.into(),Perbill::from_percent(50),false,a as u8));
        }

    }: _(SystemOrigin::Root,new_coefficents.clone())

    verify {
        let stored_coefficents=<WeightToFeePolinomialCoefficients<T>>::get();
        assert_eq!(stored_coefficents.len(),new_coefficents.len());
    }

    set_transaction_byte_fee {

        let caller:T::AccountId = whitelisted_caller();

        let new_fee:T::Balance=10u32.into();

    }: _(SystemOrigin::Root,new_fee)

    verify {
        let stored_fee=<TransactionByteFee<T>>::get();
        assert_eq!(stored_fee,new_fee);
    }

    set_fee_split_ratio {

        let caller:T::AccountId = whitelisted_caller();

        let new_ratio=10u32;

    }: _(SystemOrigin::Root,new_ratio)

    verify {
        let stored_ratio=<FeeSplitRatio<T>>::get();
        assert_eq!(stored_ratio,new_ratio);
    }

    set_extrinsic_extra {

        let caller:T::AccountId = whitelisted_caller();

        let new_extra:T::Balance=10u32.into();

        let module_index:T::ModuleIndex=1u8.into();
        let extrinsic_index:T::ExtrinsicIndex=1u8.into();

    }: _(SystemOrigin::Root,module_index,extrinsic_index,new_extra)

    verify {
        let stored_extra=<ExtrinsicExtra<T>>::get(module_index,extrinsic_index);
        assert!(stored_extra.is_some());
        let stored_extra=stored_extra.unwrap();
        assert_eq!(stored_extra,new_extra);
    }

    remove_extrinsic_extra {

        let caller:T::AccountId = whitelisted_caller();

        let new_extra:T::Balance=10u32.into();

        let module_index:T::ModuleIndex=1u8.into();
        let extrinsic_index:T::ExtrinsicIndex=1u8.into();

        SettingsPallet::<T>::set_extrinsic_extra(SystemOrigin::Root.into(),module_index,extrinsic_index,new_extra)?;

    }: _(SystemOrigin::Root,module_index,extrinsic_index)

    verify {
        assert!(!<ExtrinsicExtra<T>>::contains_key(module_index,extrinsic_index));
    }
}

impl_benchmark_test_suite!(
    SettingsPallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
