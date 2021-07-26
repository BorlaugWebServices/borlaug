// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Provenance pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    dispatch::Vec,
    traits::{Currency, Get},
};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::*;
use sp_runtime::traits::{Bounded, UniqueSaturatedFrom};
use sp_std::{prelude::*, vec};

#[allow(unused)]
use crate::Pallet as ProvenancePallet;

//TODO: compare with collective pallet in substrate and see if we need to set maximums.

type BalanceOf<T> =
    <<T as groups::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

benchmarks! {
    create_registry {
        let a in 1 .. (<T as Config>::NameLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(caller.clone()),name.clone())

    verify {
        let mut registrys=Vec::new();
        <Registries<T>>::iter_prefix(&caller).for_each(|( registry_id,_)| {
            registrys.push(registry_id);
        });
        assert_eq!(registrys.len(),1 as usize);
        let registry_id=registrys[0];

        let registry=<Registries<T>>::get(registry_id);
        assert!(registry.is_some());
        assert_eq!(registry.unwrap().name.len(),name.len());
    }


}

impl_benchmark_test_suite!(
    ProvenancePallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
