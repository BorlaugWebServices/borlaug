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
//! DATE: 2022-03-21, STEPS: `[50, ]`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
    fn create_registry(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn update_registry(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn delete_registry() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn create_asset(_a: u32, _b: u32, _c: u32, _d: u32, _e: u32, _f: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn update_asset(_a: u32, _b: u32, _c: u32, _d: u32, _e: u32, _f: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn delete_asset() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn new_lease(_a: u32, _b: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn void_lease(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create_registry(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn update_registry(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn delete_registry() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn create_asset(_a: u32, _b: u32, _c: u32, _d: u32, _e: u32, _f: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn update_asset(_a: u32, _b: u32, _c: u32, _d: u32, _e: u32, _f: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn delete_asset() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn new_lease(_a: u32, _b: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn void_lease(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
}
