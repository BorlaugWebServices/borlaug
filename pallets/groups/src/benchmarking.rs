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
    traits::{Currency, EnsureOrigin, UnfilteredDispatchable},
};
use frame_system::Call as SystemCall;
use frame_system::{self, RawOrigin as SystemOrigin};
use sp_runtime::traits::Bounded;
use sp_std::{prelude::*, vec};

#[allow(unused)]
use crate::Pallet as GroupPallet;

const SEED: u32 = 0;
const MAX_BYTES: u32 = 2;
const MAX_MEMBERS: u32 = 3;

//TODO: compare with collective pallet in substrate and see if we need to set maximums.

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

benchmarks! {
    create_group {
        let n in 1 .. MAX_BYTES;
        let m in 1 .. MAX_MEMBERS;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let mut members = vec![];
        for i in 0 .. m - 1 {
            let member = account("member", i, SEED);
            members.push(member);
        }
        let name = vec![42u8; n as usize];


    }: _(SystemOrigin::Signed(caller.clone()), name,members,1u32.into(),1_000_000_000u32.into())

    verify {
        let group_id:T::GroupId=1u32.into();
        assert_eq!(Groups::<T>::contains_key(group_id), true);
    }

    create_sub_group {
        let n in 1 .. MAX_BYTES;
        let m in 2 .. MAX_MEMBERS;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();

        GroupPallet::<T>::create_group(origin,vec![42u8; 2 as usize],vec![], 1u32.into(),1_000_000_000u32.into())?;

        let group_id:T::GroupId=1u32.into();
        assert_eq!(Groups::<T>::contains_key(group_id), true);

        let mut members = vec![];
        for i in 0 .. m - 1 {
            let member = account("member", i, SEED);
            members.push(member);
        }
        let name = vec![42u8; n as usize];

        let origin=<T as Config>::GroupsOriginByGroupThreshold::successful_origin();
        let call = Call::<T>::create_sub_group(name,members,1u32.into(),1_000u32.into());

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let group_id:T::GroupId=1u32.into();
        assert_eq!(Groups::<T>::contains_key(group_id), true);
    }

// This tests when execution would happen immediately after proposal
    propose_execute {
        let b in 1 .. MAX_BYTES;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],vec![], 1u32.into(),1_000_000u32.into())?;

      let proposal: T::Proposal = SystemCall::<T>::remark(vec![1; b as usize]).into();
        let threshold = 1u32.into();

    }: propose(SystemOrigin::Signed(caller.clone()), 1u32.into(),Box::new(proposal.clone()),threshold)

    // This tests when proposal is created and queued as "proposed"
    propose_proposed {
        let b in 1 .. MAX_BYTES;
        let m in 1 .. MAX_MEMBERS;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let mut members = vec![];
        for i in 0 .. m - 1 {
            let member = account("member", i, SEED);
            members.push(member);
        }

        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],members, m.into(),1_000_000u32.into())?;

        let proposal: T::Proposal = SystemCall::<T>::remark(vec![1; b as usize]).into();
        let threshold = m.into();

    }: propose(SystemOrigin::Signed(caller.clone()), 1u32.into(),Box::new(proposal.clone()),threshold)

// //TODO: handle case where vote results in approval and extrinsic is executed.

    vote {
        // let b in 1 .. MAX_BYTES;
        let m in 2 .. MAX_MEMBERS;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let mut members = vec![];
        for i in 0 .. m  {
            let member = account("member", i, SEED);
            members.push(member);
        }

        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],members.clone(), m.into(),1_000_000u32.into())?;

        let proposal: T::Proposal = SystemCall::<T>::remark(vec![1; 20u32 as usize]).into();
        let threshold = m.into();

        GroupPallet::<T>::propose(SystemOrigin::Signed(caller.clone()).into(), 1u32.into(),Box::new(proposal.clone()),threshold)?;


    }: _(SystemOrigin::Signed(members[0 as usize].clone()), 1u32.into(),1u32.into(),true)
}

impl_benchmark_test_suite!(GroupPallet, crate::mock::new_test_ext(), crate::mock::Test,);
