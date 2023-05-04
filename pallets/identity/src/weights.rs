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

//! Autogenerated weights for pallet_identity
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2022-03-21, STEPS: `[50, ]`, REPEAT: 5, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 128

// Executed Command:
// ./borlaug
// benchmark
// --dev
// --pallet
// pallet_identity
// --extrinsic
// *
// --steps=50
// --repeat=5
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./weights/identity/weights.rs
// --template=./frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_identity.
pub trait WeightInfo {
    fn register_did(a: u32, b: u32, c: u32) -> Weight;
    fn register_did_for(a: u32, b: u32, c: u32) -> Weight;
    fn add_did_properties(a: u32, b: u32, c: u32) -> Weight;
    fn remove_did_properties(a: u32, b: u32) -> Weight;
    fn manage_controllers(a: u32, b: u32) -> Weight;
    fn authorize_claim_consumers(a: u32) -> Weight;
    fn revoke_claim_consumers(a: u32) -> Weight;
    fn authorize_claim_issuers(a: u32) -> Weight;
    fn revoke_claim_issuers(a: u32) -> Weight;
    fn make_claim(_a: u32, _b: u32, _c: u32, _d: u32) -> Weight;
    fn attest_claim(_a: u32, _b: u32, _c: u32, _d: u32) -> Weight;
    fn revoke_attestation(a: u32, b: u32, c: u32) -> Weight;
    fn create_catalog() -> Weight;
    fn remove_catalog() -> Weight;
    fn add_dids_to_catalog(a: u32) -> Weight;
    fn remove_dids_from_catalog(a: u32) -> Weight;
}

/// Weights for pallet_identity using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_did(_a: u32, _b: u32, _c: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn register_did_for(_a: u32, _b: u32, _c: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn add_did_properties(_a: u32, _b: u32, _c: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn remove_did_properties(_a: u32, _b: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn manage_controllers(_a: u32, _b: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn authorize_claim_consumers(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn revoke_claim_consumers(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn authorize_claim_issuers(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn revoke_claim_issuers(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn make_claim(_a: u32, _b: u32, _c: u32, _d: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn attest_claim(_a: u32, _b: u32, _c: u32, _d: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn revoke_attestation(_a: u32, _b: u32, _c: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn create_catalog() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn remove_catalog() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn add_dids_to_catalog(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn remove_dids_from_catalog(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn register_did(_a: u32, _b: u32, _c: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn register_did_for(_a: u32, _b: u32, _c: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn add_did_properties(_a: u32, _b: u32, _c: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn remove_did_properties(_a: u32, _b: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn manage_controllers(_a: u32, _b: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn authorize_claim_consumers(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn revoke_claim_consumers(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn authorize_claim_issuers(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn revoke_claim_issuers(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn make_claim(_a: u32, _b: u32, _c: u32, _d: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn attest_claim(_a: u32, _b: u32, _c: u32, _d: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn revoke_attestation(_a: u32, _b: u32, _c: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn create_catalog() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn remove_catalog() -> Weight {
        Weight::from_parts(0, 0)
    }
    fn add_dids_to_catalog(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
    fn remove_dids_from_catalog(_a: u32) -> Weight {
        Weight::from_parts(0, 0)
    }
}
