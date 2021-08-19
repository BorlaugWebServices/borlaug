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

//! Autogenerated weights for pallet_asset_registry
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-07-28, STEPS: `[50, ]`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 128

// Executed Command:
// E:\qlikchain\borlaug\target\release\borlaug.exe
// benchmark
// --dev
// --pallet
// pallet_asset_registry
// --extrinsic
// *
// --steps=50
// --repeat=5
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/asset-registry/src/weights.rs
// --template=./.maintain/frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_asset_registry.
pub trait WeightInfo {
    fn create_registry(a: u32) -> Weight;
    fn update_registry(a: u32) -> Weight;
    fn delete_registry() -> Weight;
    fn create_asset(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> Weight;
    fn update_asset(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> Weight;
    fn delete_asset() -> Weight;
    fn new_lease(a: u32, b: u32) -> Weight;
}

/// Weights for pallet_asset_registry using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_registry(_a: u32) -> Weight {
        (37_451_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn update_registry(a: u32) -> Weight {
        (39_357_000 as Weight)
            // Standard Error: 51_000
            .saturating_add((1_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn delete_registry() -> Weight {
        (58_700_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn create_asset(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 309_000
            .saturating_add((1_957_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 309_000
            .saturating_add((1_941_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 309_000
            .saturating_add((262_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 309_000
            .saturating_add((2_566_000 as Weight).saturating_mul(d as Weight))
            // Standard Error: 28_000
            .saturating_add((1_244_000 as Weight).saturating_mul(e as Weight))
            // Standard Error: 28_000
            .saturating_add((1_974_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn update_asset(_a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 266_000
            .saturating_add((24_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 266_000
            .saturating_add((386_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 266_000
            .saturating_add((1_352_000 as Weight).saturating_mul(d as Weight))
            // Standard Error: 24_000
            .saturating_add((1_094_000 as Weight).saturating_mul(e as Weight))
            // Standard Error: 24_000
            .saturating_add((1_860_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn delete_asset() -> Weight {
        (31_100_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn new_lease(_a: u32, b: u32) -> Weight {
        (83_850_000 as Weight)
            // Standard Error: 108_000
            .saturating_add((17_955_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create_registry(_a: u32) -> Weight {
        (37_451_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn update_registry(a: u32) -> Weight {
        (39_357_000 as Weight)
            // Standard Error: 51_000
            .saturating_add((1_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn delete_registry() -> Weight {
        (58_700_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn create_asset(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 309_000
            .saturating_add((1_957_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 309_000
            .saturating_add((1_941_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 309_000
            .saturating_add((262_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 309_000
            .saturating_add((2_566_000 as Weight).saturating_mul(d as Weight))
            // Standard Error: 28_000
            .saturating_add((1_244_000 as Weight).saturating_mul(e as Weight))
            // Standard Error: 28_000
            .saturating_add((1_974_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn update_asset(_a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 266_000
            .saturating_add((24_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 266_000
            .saturating_add((386_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 266_000
            .saturating_add((1_352_000 as Weight).saturating_mul(d as Weight))
            // Standard Error: 24_000
            .saturating_add((1_094_000 as Weight).saturating_mul(e as Weight))
            // Standard Error: 24_000
            .saturating_add((1_860_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn delete_asset() -> Weight {
        (31_100_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn new_lease(_a: u32, b: u32) -> Weight {
        (83_850_000 as Weight)
            // Standard Error: 108_000
            .saturating_add((17_955_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().reads((2 as Weight).saturating_mul(b as Weight)))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
    }
}