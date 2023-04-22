// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for pallet_provenance
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2022-03-21, STEPS: `[50, ]`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 128

// Executed Command:
// ./borlaug
// benchmark
// --dev
// --pallet
// pallet_provenance
// --extrinsic
// *
// --steps=50
// --repeat=5
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./weights/provenance/weights.rs
// --template=./frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_provenance.
pub trait WeightInfo {
    fn create_registry(a: u32) -> Weight;
    fn update_registry(a: u32) -> Weight;
    fn remove_registry() -> Weight;
    fn create_definition(a: u32, b: u32, c: u32) -> Weight;
    fn set_definition_active() -> Weight;
    fn set_definition_inactive() -> Weight;
    fn remove_definition(a: u32) -> Weight;
    fn update_definition_step() -> Weight;
    fn create_process(a: u32) -> Weight;
    fn update_process(a: u32) -> Weight;
    fn remove_process(a: u32) -> Weight;
    fn add_child_definition() -> Weight;
    fn remove_child_definition() -> Weight;
    fn attest_process_step(a: u32, b: u32, c: u32) -> Weight;
    fn complete_process(a: u32) -> Weight;
}

/// Weights for pallet_provenance using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_registry(_a: u32) -> Weight {
        Weight::from_ref_time(78_623_000 as u64)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    fn update_registry(a: u32) -> Weight {
        Weight::from_ref_time(73_259_000 as u64)
            // Standard Error: 3_000
            .saturating_add(Weight::from_ref_time(4_000 as u64).saturating_mul(a as u64))
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn remove_registry() -> Weight {
        Weight::from_ref_time(86_710_000 as u64)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn create_definition(_a: u32, _b: u32, c: u32) -> Weight {
        Weight::from_ref_time(212_030_000 as u64)
            // Standard Error: 32_000
            .saturating_add(Weight::from_ref_time(12_336_000 as u64).saturating_mul(c as u64))
            .saturating_add(T::DbWeight::get().reads(3 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
            .saturating_add(T::DbWeight::get().writes((2 as u64).saturating_mul(c as u64)))
    }
    fn set_definition_active() -> Weight {
        Weight::from_ref_time(82_682_000 as u64)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn set_definition_inactive() -> Weight {
        Weight::from_ref_time(77_616_000 as u64)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn remove_definition(a: u32) -> Weight {
        Weight::from_ref_time(0 as u64)
            // Standard Error: 304_000
            .saturating_add(Weight::from_ref_time(40_502_000 as u64).saturating_mul(a as u64))
            .saturating_add(T::DbWeight::get().reads(6 as u64))
            .saturating_add(T::DbWeight::get().reads((1 as u64).saturating_mul(a as u64)))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(a as u64)))
    }
    fn update_definition_step() -> Weight {
        Weight::from_ref_time(112_441_000 as u64)
            .saturating_add(T::DbWeight::get().reads(3 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    fn create_process(_a: u32) -> Weight {
        Weight::from_ref_time(114_075_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    fn update_process(_a: u32) -> Weight {
        Weight::from_ref_time(88_524_000 as u64)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn remove_process(a: u32) -> Weight {
        Weight::from_ref_time(118_246_000 as u64)
            // Standard Error: 6_000
            .saturating_add(Weight::from_ref_time(36_000 as u64).saturating_mul(a as u64))
            .saturating_add(T::DbWeight::get().reads(3 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn add_child_definition() -> Weight {
        Weight::from_ref_time(121_169_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    fn remove_child_definition() -> Weight {
        Weight::from_ref_time(92_507_000 as u64)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    fn attest_process_step(a: u32, b: u32, c: u32) -> Weight {
        Weight::from_ref_time(19_146_000 as u64)
            // Standard Error: 2_000
            .saturating_add(Weight::from_ref_time(643_000 as u64).saturating_mul(a as u64))
            // Standard Error: 11_000
            .saturating_add(Weight::from_ref_time(586_000 as u64).saturating_mul(b as u64))
            // Standard Error: 11_000
            .saturating_add(Weight::from_ref_time(546_000 as u64).saturating_mul(c as u64))
            .saturating_add(T::DbWeight::get().reads(5 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn complete_process(a: u32) -> Weight {
        Weight::from_ref_time(0 as u64)
            // Standard Error: 147_000
            .saturating_add(Weight::from_ref_time(58_951_000 as u64).saturating_mul(a as u64))
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().reads((2 as u64).saturating_mul(a as u64)))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create_registry(_a: u32) -> Weight {
        Weight::from_ref_time(78_623_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    fn update_registry(a: u32) -> Weight {
        Weight::from_ref_time(73_259_000 as u64)
            // Standard Error: 3_000
            .saturating_add(Weight::from_ref_time(4_000 as u64).saturating_mul(a as u64))
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn remove_registry() -> Weight {
        Weight::from_ref_time(86_710_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn create_definition(_a: u32, _b: u32, c: u32) -> Weight {
        Weight::from_ref_time(212_030_000 as u64)
            // Standard Error: 32_000
            .saturating_add(Weight::from_ref_time(12_336_000 as u64).saturating_mul(c as u64))
            .saturating_add(RocksDbWeight::get().reads(3 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
            .saturating_add(RocksDbWeight::get().writes((2 as u64).saturating_mul(c as u64)))
    }
    fn set_definition_active() -> Weight {
        Weight::from_ref_time(82_682_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn set_definition_inactive() -> Weight {
        Weight::from_ref_time(77_616_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn remove_definition(a: u32) -> Weight {
        Weight::from_ref_time(0 as u64)
            // Standard Error: 304_000
            .saturating_add(Weight::from_ref_time(40_502_000 as u64).saturating_mul(a as u64))
            .saturating_add(RocksDbWeight::get().reads(6 as u64))
            .saturating_add(RocksDbWeight::get().reads((1 as u64).saturating_mul(a as u64)))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(a as u64)))
    }
    fn update_definition_step() -> Weight {
        Weight::from_ref_time(112_441_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(3 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    fn create_process(_a: u32) -> Weight {
        Weight::from_ref_time(114_075_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    fn update_process(_a: u32) -> Weight {
        Weight::from_ref_time(88_524_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn remove_process(a: u32) -> Weight {
        Weight::from_ref_time(118_246_000 as u64)
            // Standard Error: 6_000
            .saturating_add(Weight::from_ref_time(36_000 as u64).saturating_mul(a as u64))
            .saturating_add(RocksDbWeight::get().reads(3 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn add_child_definition() -> Weight {
        Weight::from_ref_time(121_169_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    fn remove_child_definition() -> Weight {
        Weight::from_ref_time(92_507_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    fn attest_process_step(a: u32, b: u32, c: u32) -> Weight {
        Weight::from_ref_time(19_146_000 as u64)
            // Standard Error: 2_000
            .saturating_add(Weight::from_ref_time(643_000 as u64).saturating_mul(a as u64))
            // Standard Error: 11_000
            .saturating_add(Weight::from_ref_time(586_000 as u64).saturating_mul(b as u64))
            // Standard Error: 11_000
            .saturating_add(Weight::from_ref_time(546_000 as u64).saturating_mul(c as u64))
            .saturating_add(RocksDbWeight::get().reads(5 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn complete_process(a: u32) -> Weight {
        Weight::from_ref_time(0 as u64)
            // Standard Error: 147_000
            .saturating_add(Weight::from_ref_time(58_951_000 as u64).saturating_mul(a as u64))
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().reads((2 as u64).saturating_mul(a as u64)))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
}
