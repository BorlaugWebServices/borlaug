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

//! Autogenerated weights for pallet_settings
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-08-09, STEPS: `[50, ]`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 128

// Executed Command:
// E:\qlikchain\borlaug\target\release\borlaug.exe
// benchmark
// --dev
// --pallet
// pallet_settings
// --extrinsic
// *
// --steps=50
// --repeat=5
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/settings/src/weights.rs
// --template=./.maintain/frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_settings.
pub trait WeightInfo {
    fn set_weight_to_fee_coefficients(a: u32) -> Weight;
    fn set_transaction_byte_fee() -> Weight;
    fn set_fee_split_ratio() -> Weight;
    fn set_extrinsic_extra() -> Weight;
    fn remove_extrinsic_extra() -> Weight;
}

/// Weights for pallet_settings using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn set_weight_to_fee_coefficients(_a: u32) -> Weight {
        (18_498_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_transaction_byte_fee() -> Weight {
        (18_500_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_fee_split_ratio() -> Weight {
        (18_200_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_extrinsic_extra() -> Weight {
        (21_400_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn remove_extrinsic_extra() -> Weight {
        (18_800_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn set_weight_to_fee_coefficients(_a: u32) -> Weight {
        (18_498_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_transaction_byte_fee() -> Weight {
        (18_500_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_fee_split_ratio() -> Weight {
        (18_200_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_extrinsic_extra() -> Weight {
        (21_400_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn remove_extrinsic_extra() -> Weight {
        (18_800_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
}
