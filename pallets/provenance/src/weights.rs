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
//! DATE: 2021-07-27, STEPS: `[50, ]`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 128

// Executed Command:
// E:\qlikchain\borlaug\target\release\borlaug.exe
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
// --output=./pallets/provenance/src/weights.rs
// --template=./.maintain/frame-weight-template.hbs


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_provenance.
pub trait WeightInfo {
	fn create_registry(a: u32, ) -> Weight;
	fn update_registry(a: u32, ) -> Weight;
	fn remove_registry() -> Weight;
	fn create_definition(a: u32, ) -> Weight;
	fn update_definition(a: u32, ) -> Weight;
	fn set_definition_active(a: u32, ) -> Weight;
	fn set_definition_inactive() -> Weight;
	fn remove_definition(a: u32, ) -> Weight;
	fn create_definition_step(a: u32, ) -> Weight;
	fn update_definition_step(a: u32, ) -> Weight;
	fn delete_definition_step(a: u32, ) -> Weight;
	fn create_process(a: u32, ) -> Weight;
	fn update_process(a: u32, ) -> Weight;
	fn remove_process(a: u32, ) -> Weight;
	fn update_process_step(a: u32, b: u32, c: u32, ) -> Weight;
	fn attest_process_step() -> Weight;
}

/// Weights for pallet_provenance using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn create_registry(_a: u32, ) -> Weight {
		(39_076_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn update_registry(_a: u32, ) -> Weight {
		(34_516_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_registry() -> Weight {
		(38_800_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn create_definition(a: u32, ) -> Weight {
		(43_572_000 as Weight)
			// Standard Error: 18_000
			.saturating_add((17_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn update_definition(a: u32, ) -> Weight {
		(39_811_000 as Weight)
			// Standard Error: 16_000
			.saturating_add((10_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_definition_active(a: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 74_000
			.saturating_add((12_133_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_definition_inactive() -> Weight {
		(36_900_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_definition(a: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 80_000
			.saturating_add((13_416_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
	}
	fn create_definition_step(a: u32, ) -> Weight {
		(42_357_000 as Weight)
			// Standard Error: 15_000
			.saturating_add((10_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn update_definition_step(_a: u32, ) -> Weight {
		(51_849_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn delete_definition_step(a: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 42_000
			.saturating_add((13_299_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
	}
	fn create_process(a: u32, ) -> Weight {
		(58_328_000 as Weight)
			// Standard Error: 16_000
			.saturating_add((13_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn update_process(_a: u32, ) -> Weight {
		(46_505_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_process(a: u32, ) -> Weight {
		(63_304_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((26_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn update_process_step(a: u32, b: u32, c: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 10_000
			.saturating_add((995_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 112_000
			.saturating_add((1_288_000 as Weight).saturating_mul(b as Weight))
			// Standard Error: 10_000
			.saturating_add((754_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn attest_process_step() -> Weight {
		(52_700_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_registry(_a: u32, ) -> Weight {
		(39_076_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn update_registry(_a: u32, ) -> Weight {
		(34_516_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn remove_registry() -> Weight {
		(38_800_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn create_definition(a: u32, ) -> Weight {
		(43_572_000 as Weight)
			// Standard Error: 18_000
			.saturating_add((17_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn update_definition(a: u32, ) -> Weight {
		(39_811_000 as Weight)
			// Standard Error: 16_000
			.saturating_add((10_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn set_definition_active(a: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 74_000
			.saturating_add((12_133_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn set_definition_inactive() -> Weight {
		(36_900_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn remove_definition(a: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 80_000
			.saturating_add((13_416_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(RocksDbWeight::get().reads(4 as Weight))
			.saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
	}
	fn create_definition_step(a: u32, ) -> Weight {
		(42_357_000 as Weight)
			// Standard Error: 15_000
			.saturating_add((10_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn update_definition_step(_a: u32, ) -> Weight {
		(51_849_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn delete_definition_step(a: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 42_000
			.saturating_add((13_299_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
	}
	fn create_process(a: u32, ) -> Weight {
		(58_328_000 as Weight)
			// Standard Error: 16_000
			.saturating_add((13_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(RocksDbWeight::get().reads(4 as Weight))
			.saturating_add(RocksDbWeight::get().writes(3 as Weight))
	}
	fn update_process(_a: u32, ) -> Weight {
		(46_505_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn remove_process(a: u32, ) -> Weight {
		(63_304_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((26_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(RocksDbWeight::get().reads(4 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
	fn update_process_step(a: u32, b: u32, c: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 10_000
			.saturating_add((995_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 112_000
			.saturating_add((1_288_000 as Weight).saturating_mul(b as Weight))
			// Standard Error: 10_000
			.saturating_add((754_000 as Weight).saturating_mul(c as Weight))
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn attest_process_step() -> Weight {
		(52_700_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
}
