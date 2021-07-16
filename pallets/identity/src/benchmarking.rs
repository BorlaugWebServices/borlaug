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

//! Identity pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    dispatch::Vec,
    traits::{Currency, EnsureOrigin, Get, UnfilteredDispatchable},
};
use frame_system::Call as SystemCall;
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::*;
use sp_runtime::traits::Bounded;
use sp_std::{prelude::*, vec};

#[allow(unused)]
use crate::Pallet as IdentityPallet;

const SEED: u32 = 0;

//TODO: compare with collective pallet in substrate and see if we need to set maximums.

type BalanceOf<T> =
    <<T as groups::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

benchmarks! {
    register_did {
        let a in 1 .. (<T as Config>::NameLimit::get() -1);
        let b in 1 .. (<T as Config>::NameLimit::get()-1);
        let c in 1 .. (<T as Config>::FactStringLimit::get()-1);
        let d in 1 .. (<T as Config>::PropertyLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());


        let name = Some(vec![42u8; a as usize]);

        let mut properties=Vec::new();
        for x in 1..d {
            properties.push(DidProperty{
                name:vec![42u8; b as usize],
                fact:Fact::Text(vec![42u8; c as usize])
            });
        }


    }: _(SystemOrigin::Signed(caller.clone()), name, Some(properties))

    verify {

        assert_eq!(true, true);
    }


}

impl_benchmark_test_suite!(
    IdentityPallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
