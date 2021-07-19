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

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    dispatch::Vec,
    traits::{Currency, Get},
};
// use frame_system::Call as SystemCall;
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::*;
use sp_runtime::traits::Bounded;
use sp_std::{prelude::*, vec};

#[allow(unused)]
use crate::Pallet as IdentityPallet;

//TODO: compare with collective pallet in substrate and see if we need to set maximums.

type BalanceOf<T> =
    <<T as groups::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

fn create_properties(
    property_count: u32,
    property_name_len: u32,
    property_fact_len: u32,
) -> Vec<DidProperty<Vec<u8>, Vec<u8>>> {
    let mut properties = Vec::new();
    for _ in 1..property_count {
        properties.push(DidProperty {
            name: vec![42u8; property_name_len as usize],
            fact: Fact::Text(vec![42u8; property_fact_len as usize]),
        });
    }
    properties
}

benchmarks! {
    register_did {
        let a in 1 .. (<T as Config>::NameLimit::get() -1);//short_name length
        let b in 1 .. (<T as Config>::NameLimit::get()-1);//property name length
        let c in 1 .. (<T as Config>::FactStringLimit::get()-1);//property fact length
        let d in 1 .. (<T as Config>::PropertyLimit::get()-1);//property count

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let name = vec![42u8; a as usize];

        let properties=create_properties(d,b,c);

    }: _(SystemOrigin::Signed(caller.clone()), Some(name), Some(properties))

    verify {
        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let mut dids_by_subject=Vec::new();
        <DidBySubject<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_subject.push(did);
        });
        assert_eq!(dids_by_subject.len(), 1);
    }

    register_did_for {
        let a in 1 .. (<T as Config>::NameLimit::get() -1);//short_name length
        let b in 1 .. (<T as Config>::NameLimit::get()-1);//property name length
        let c in 1 .. (<T as Config>::FactStringLimit::get()-1);//property fact length
        let d in 1 .. (<T as Config>::PropertyLimit::get()-1);//property count

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let subject:<T as frame_system::Config>::AccountId = whitelisted_caller();

        let name = vec![42u8; a as usize];

        let properties=create_properties(d,b,c);

    }: _(SystemOrigin::Signed(caller.clone()),subject.clone(), Some(name), Some(properties))

    verify {
        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let mut dids_by_subject=Vec::new();
        <DidByController<T>>::iter_prefix(&subject).for_each(|(did, _)| {
            dids_by_subject.push(did);
        });
        assert_eq!(dids_by_subject.len(), 1);
    }

    update_did {
        let a in 1 .. (<T as Config>::NameLimit::get() -1); //short_name length
        let b in 1 .. (<T as Config>::NameLimit::get()-1); //add property name length
        let c in 1 .. (<T as Config>::FactStringLimit::get()-1); //add property fact length
        let d in 1 .. (<T as Config>::PropertyLimit::get()-1); //add property count
        let e in 1 .. (<T as Config>::NameLimit::get()-1); //remove_keys key length
        let f in 1 .. (<T as Config>::PropertyLimit::get()-1); //remove_keys count


        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origional_name = vec![42u8; <T as Config>::NameLimit::get() as usize];
        //these will be removed
        let origional_properties=create_properties(f,e,<T as Config>::FactStringLimit::get()-1);

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::register_did(origin,Some(origional_name), Some(origional_properties.clone()))?;

        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        let add_properties=create_properties(d,b,c);

        let remove_keys=origional_properties.into_iter().map(|property|property.name).collect();

        let name = vec![42u8; a as usize];
    }: _(SystemOrigin::Signed(caller.clone()),did, Some(name.clone()),  Some(add_properties.clone()),Some(remove_keys))

    verify {
        let did_document=<DidDocuments<T>>::get(&did);
        assert!(did_document.is_some());
        let did_document=did_document.unwrap();
        assert!(did_document.short_name.is_some());
        assert_eq!(did_document.short_name.unwrap().len(),name.len());

        let mut stored_properties=Vec::new();
        <DidDocumentProperties<T>>::iter_prefix(&did).for_each(|(_, property)| {
            stored_properties.push(property);
        });
        assert_eq!(stored_properties.len(), add_properties.len());
    }

}

impl_benchmark_test_suite!(
    IdentityPallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
