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
//! DATE: 2021-10-25, STEPS: `[50, ]`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
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

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_provenance.
pub trait WeightInfo {
fn create_registry(a: u32, ) -> Weight;
fn update_registry(a: u32, ) -> Weight;
fn remove_registry() -> Weight;
fn create_definition(a: u32, b: u32, c: u32, ) -> Weight;
fn set_definition_active() -> Weight;
fn set_definition_inactive() -> Weight;
fn remove_definition(a: u32, ) -> Weight;
fn update_definition_step() -> Weight;
fn create_process(a: u32, ) -> Weight;
fn update_process(a: u32, ) -> Weight;
fn remove_process(a: u32, ) -> Weight;
fn attest_process_step(a: u32, b: u32, c: u32, ) -> Weight;
}

/// Weights for pallet_provenance using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
		impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
				fn create_registry(a: u32, ) -> Weight {
				(104_417_000 as Weight)
				// Standard Error: 3_000
				.saturating_add((1_000 as Weight).saturating_mul(a as Weight))
				.saturating_add(T::DbWeight::get().reads(2 as Weight))
				.saturating_add(T::DbWeight::get().writes(2 as Weight))
				}
				fn update_registry(_a: u32, ) -> Weight {
				(96_784_000 as Weight)
				.saturating_add(T::DbWeight::get().reads(1 as Weight))
				.saturating_add(T::DbWeight::get().writes(1 as Weight))
				}
				fn remove_registry() -> Weight {
				(107_797_000 as Weight)
				.saturating_add(T::DbWeight::get().reads(2 as Weight))
				.saturating_add(T::DbWeight::get().writes(1 as Weight))
				}
				fn create_definition(a: u32, b: u32, c: u32, ) -> Weight {
				(0 as Weight)
				// Standard Error: 44_000
				.saturating_add((114_000 as Weight).saturating_mul(a as Weight))
				// Standard Error: 44_000
				.saturating_add((2_213_000 as Weight).saturating_mul(b as Weight))
				// Standard Error: 44_000
				.saturating_add((19_855_000 as Weight).saturating_mul(c as Weight))
				.saturating_add(T::DbWeight::get().reads(3 as Weight))
				.saturating_add(T::DbWeight::get().writes(2 as Weight))
				.saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(c as Weight)))
				}
				fn set_definition_active() -> Weight {
				(104_017_000 as Weight)
				.saturating_add(T::DbWeight::get().reads(2 as Weight))
				.saturating_add(T::DbWeight::get().writes(1 as Weight))
				}
				fn set_definition_inactive() -> Weight {
				(103_916_000 as Weight)
				.saturating_add(T::DbWeight::get().reads(2 as Weight))
				.saturating_add(T::DbWeight::get().writes(1 as Weight))
				}
				fn remove_definition(a: u32, ) -> Weight {
				(0 as Weight)
				// Standard Error: 150_000
				.saturating_add((61_240_000 as Weight).saturating_mul(a as Weight))
				.saturating_add(T::DbWeight::get().reads(4 as Weight))
				.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
				.saturating_add(T::DbWeight::get().writes(1 as Weight))
				.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
				}
				fn update_definition_step() -> Weight {
				(156_973_000 as Weight)
				.saturating_add(T::DbWeight::get().reads(3 as Weight))
				.saturating_add(T::DbWeight::get().writes(2 as Weight))
				}
				fn create_process(a: u32, ) -> Weight {
				(150_088_000 as Weight)
				// Standard Error: 2_000
				.saturating_add((12_000 as Weight).saturating_mul(a as Weight))
				.saturating_add(T::DbWeight::get().reads(4 as Weight))
				.saturating_add(T::DbWeight::get().writes(2 as Weight))
				}
				fn update_process(a: u32, ) -> Weight {
				(115_718_000 as Weight)
				// Standard Error: 1_000
				.saturating_add((9_000 as Weight).saturating_mul(a as Weight))
				.saturating_add(T::DbWeight::get().reads(2 as Weight))
				.saturating_add(T::DbWeight::get().writes(1 as Weight))
				}
				fn remove_process(a: u32, ) -> Weight {
				(150_608_000 as Weight)
				// Standard Error: 5_000
				.saturating_add((26_000 as Weight).saturating_mul(a as Weight))
				.saturating_add(T::DbWeight::get().reads(3 as Weight))
				.saturating_add(T::DbWeight::get().writes(1 as Weight))
				}
				fn attest_process_step(a: u32, b: u32, c: u32, ) -> Weight {
				(0 as Weight)
				// Standard Error: 51_000
				.saturating_add((4_153_000 as Weight).saturating_mul(a as Weight))
				// Standard Error: 51_000
				.saturating_add((1_708_000 as Weight).saturating_mul(b as Weight))
				// Standard Error: 51_000
				.saturating_add((1_712_000 as Weight).saturating_mul(c as Weight))
				.saturating_add(T::DbWeight::get().reads(3 as Weight))
				.saturating_add(T::DbWeight::get().writes(1 as Weight))
				}
				}

				// For backwards compatibility and tests
				impl WeightInfo for () {
				fn create_registry(a: u32, ) -> Weight {
				(104_417_000 as Weight)
				// Standard Error: 3_000
				.saturating_add((1_000 as Weight).saturating_mul(a as Weight))
				.saturating_add(RocksDbWeight::get().reads(2 as Weight))
				.saturating_add(RocksDbWeight::get().writes(2 as Weight))
				}
				fn update_registry(_a: u32, ) -> Weight {
				(96_784_000 as Weight)
				.saturating_add(RocksDbWeight::get().reads(1 as Weight))
				.saturating_add(RocksDbWeight::get().writes(1 as Weight))
				}
				fn remove_registry() -> Weight {
				(107_797_000 as Weight)
				.saturating_add(RocksDbWeight::get().reads(2 as Weight))
				.saturating_add(RocksDbWeight::get().writes(1 as Weight))
				}
				fn create_definition(a: u32, b: u32, c: u32, ) -> Weight {
				(0 as Weight)
				// Standard Error: 44_000
				.saturating_add((114_000 as Weight).saturating_mul(a as Weight))
				// Standard Error: 44_000
				.saturating_add((2_213_000 as Weight).saturating_mul(b as Weight))
				// Standard Error: 44_000
				.saturating_add((19_855_000 as Weight).saturating_mul(c as Weight))
				.saturating_add(RocksDbWeight::get().reads(3 as Weight))
				.saturating_add(RocksDbWeight::get().writes(2 as Weight))
				.saturating_add(RocksDbWeight::get().writes((2 as Weight).saturating_mul(c as Weight)))
				}
				fn set_definition_active() -> Weight {
				(104_017_000 as Weight)
				.saturating_add(RocksDbWeight::get().reads(2 as Weight))
				.saturating_add(RocksDbWeight::get().writes(1 as Weight))
				}
				fn set_definition_inactive() -> Weight {
				(103_916_000 as Weight)
				.saturating_add(RocksDbWeight::get().reads(2 as Weight))
				.saturating_add(RocksDbWeight::get().writes(1 as Weight))
				}
				fn remove_definition(a: u32, ) -> Weight {
				(0 as Weight)
				// Standard Error: 150_000
				.saturating_add((61_240_000 as Weight).saturating_mul(a as Weight))
				.saturating_add(RocksDbWeight::get().reads(4 as Weight))
				.saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
				.saturating_add(RocksDbWeight::get().writes(1 as Weight))
				.saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
				}
				fn update_definition_step() -> Weight {
				(156_973_000 as Weight)
				.saturating_add(RocksDbWeight::get().reads(3 as Weight))
				.saturating_add(RocksDbWeight::get().writes(2 as Weight))
				}
				fn create_process(a: u32, ) -> Weight {
				(150_088_000 as Weight)
				// Standard Error: 2_000
				.saturating_add((12_000 as Weight).saturating_mul(a as Weight))
				.saturating_add(RocksDbWeight::get().reads(4 as Weight))
				.saturating_add(RocksDbWeight::get().writes(2 as Weight))
				}
				fn update_process(a: u32, ) -> Weight {
				(115_718_000 as Weight)
				// Standard Error: 1_000
				.saturating_add((9_000 as Weight).saturating_mul(a as Weight))
				.saturating_add(RocksDbWeight::get().reads(2 as Weight))
				.saturating_add(RocksDbWeight::get().writes(1 as Weight))
				}
				fn remove_process(a: u32, ) -> Weight {
				(150_608_000 as Weight)
				// Standard Error: 5_000
				.saturating_add((26_000 as Weight).saturating_mul(a as Weight))
				.saturating_add(RocksDbWeight::get().reads(3 as Weight))
				.saturating_add(RocksDbWeight::get().writes(1 as Weight))
				}
				fn attest_process_step(a: u32, b: u32, c: u32, ) -> Weight {
				(0 as Weight)
				// Standard Error: 51_000
				.saturating_add((4_153_000 as Weight).saturating_mul(a as Weight))
				// Standard Error: 51_000
				.saturating_add((1_708_000 as Weight).saturating_mul(b as Weight))
				// Standard Error: 51_000
				.saturating_add((1_712_000 as Weight).saturating_mul(c as Weight))
				.saturating_add(RocksDbWeight::get().reads(3 as Weight))
				.saturating_add(RocksDbWeight::get().writes(1 as Weight))
				}
				}