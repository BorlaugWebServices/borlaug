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
//! DATE: 2021-10-25, STEPS: `[50, ]`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 128

// Executed Command:
// ./borlaug
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
// --output=./weights/settings/weights.rs
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
    fn set_weight_to_fee_coefficients(a: u32) -> Weight {
        (67_451_000 as Weight)
            // Standard Error: 101_000
            .saturating_add((144_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_transaction_byte_fee() -> Weight {
        (65_995_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_fee_split_ratio() -> Weight {
        (63_364_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_extrinsic_extra() -> Weight {
        (70_264_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn remove_extrinsic_extra() -> Weight {
        (65_461_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn set_weight_to_fee_coefficients(a: u32) -> Weight {
        (67_451_000 as Weight)
            // Standard Error: 101_000
            .saturating_add((144_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_transaction_byte_fee() -> Weight {
        (65_995_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_fee_split_ratio() -> Weight {
        (63_364_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_extrinsic_extra() -> Weight {
        (70_264_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn remove_extrinsic_extra() -> Weight {
        (65_461_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
}
