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

//! Groups pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    dispatch::Vec,
    traits::{Currency, EnsureOrigin, OnInitialize, UnfilteredDispatchable},
};
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, Zero};
use sp_std::{prelude::*, vec};

use crate::Pallet as GroupPallet;

const SEED: u32 = 0;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

benchmarks! {
    create_group {
        let b in 1 .. 100;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let member: T::AccountId = account("member", 0, SEED);

        let name = vec![42u8; (b%100) as usize];


    }: _(RawOrigin::Signed(caller.clone()), name,vec![member],(1u32%2).into(),1_000_000u32.into())


    verify {
        let group_id:T::GroupId=1u32.into();
        assert_eq!(Groups::<T>::contains_key(group_id), true);
    }


}

impl_benchmark_test_suite!(GroupPallet, crate::mock::new_test_ext(), crate::mock::Test,);
