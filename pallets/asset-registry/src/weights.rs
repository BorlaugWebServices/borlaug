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
//! DATE: 2021-10-25, STEPS: `[50, ]`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 128

// Executed Command:
// ./borlaug
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
// --output=./weights/asset-registry/weights.rs
// --template=./frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::many_single_char_names)]

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
    fn void_lease(a: u32) -> Weight;
}

/// Weights for pallet_asset_registry using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_registry(a: u32) -> Weight {
        (122_214_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((12_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn update_registry(_a: u32) -> Weight {
        (116_343_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn delete_registry() -> Weight {
        (142_919_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn create_asset(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 36_000
            .saturating_add((207_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 36_000
            .saturating_add((304_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 36_000
            .saturating_add((304_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 36_000
            .saturating_add((2_209_000 as Weight).saturating_mul(d as Weight))
            // Standard Error: 36_000
            .saturating_add((1_846_000 as Weight).saturating_mul(e as Weight))
            // Standard Error: 36_000
            .saturating_add((6_079_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn update_asset(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 37_000
            .saturating_add((304_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 37_000
            .saturating_add((319_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 37_000
            .saturating_add((336_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 37_000
            .saturating_add((2_179_000 as Weight).saturating_mul(d as Weight))
            // Standard Error: 37_000
            .saturating_add((1_822_000 as Weight).saturating_mul(e as Weight))
            // Standard Error: 37_000
            .saturating_add((6_068_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn delete_asset() -> Weight {
        (94_675_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn new_lease(a: u32, b: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 89_000
            .saturating_add((820_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 89_000
            .saturating_add((51_734_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
    }
    fn void_lease(a: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 74_000
            .saturating_add((27_770_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create_registry(a: u32) -> Weight {
        (122_214_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((12_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn update_registry(_a: u32) -> Weight {
        (116_343_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn delete_registry() -> Weight {
        (142_919_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn create_asset(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 36_000
            .saturating_add((207_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 36_000
            .saturating_add((304_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 36_000
            .saturating_add((304_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 36_000
            .saturating_add((2_209_000 as Weight).saturating_mul(d as Weight))
            // Standard Error: 36_000
            .saturating_add((1_846_000 as Weight).saturating_mul(e as Weight))
            // Standard Error: 36_000
            .saturating_add((6_079_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn update_asset(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 37_000
            .saturating_add((304_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 37_000
            .saturating_add((319_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 37_000
            .saturating_add((336_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 37_000
            .saturating_add((2_179_000 as Weight).saturating_mul(d as Weight))
            // Standard Error: 37_000
            .saturating_add((1_822_000 as Weight).saturating_mul(e as Weight))
            // Standard Error: 37_000
            .saturating_add((6_068_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn delete_asset() -> Weight {
        (94_675_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn new_lease(a: u32, b: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 89_000
            .saturating_add((820_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 89_000
            .saturating_add((51_734_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().reads((2 as Weight).saturating_mul(b as Weight)))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
    }
    fn void_lease(a: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 74_000
            .saturating_add((27_770_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
    }
}
