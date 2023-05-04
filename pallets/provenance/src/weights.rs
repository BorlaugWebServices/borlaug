
//! Autogenerated weights for pallet_provenance
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 25.0.0
//! DATE: 2023-05-04, STEPS: `50`, REPEAT: `5`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Tims-PC`, CPU: `12th Gen Intel(R) Core(TM) i9-12900K`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/borlaug
// benchmark
// pallet
// --pallet
// pallet_provenance
// --extrinsic
// *
// --steps=50
// --repeat=5
// --heap-pages=4096
// --output=./pallets/provenance/src/weights.rs
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

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
fn add_child_definition() -> Weight;
fn remove_child_definition() -> Weight;
fn attest_process_step(a: u32, b: u32, c: u32, ) -> Weight;
fn complete_process(a: u32, ) -> Weight;
}

/// Weights for pallet_provenance using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
				impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
						/// Storage: Settings ExtrinsicExtra (r:1 w:0)
						/// Proof: Settings ExtrinsicExtra (max_values: None, max_size: Some(18), added: 2493, mode: MaxEncodedLen)
						/// Storage: Provenance NextRegistryId (r:1 w:1)
						/// Proof: Provenance NextRegistryId (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
						/// Storage: Provenance Registries (r:0 w:1)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 100]`.
						fn create_registry(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `180`
						// Estimated: `4972`
						// Minimum execution time: 7_249_000 picoseconds.
						Weight::from_parts(8_111_025, 4972)
						// Standard Error: 6_368
						.saturating_add(Weight::from_parts(7_627, 0).saturating_mul(a.into()))
						.saturating_add(T::DbWeight::get().reads(2_u64))
						.saturating_add(T::DbWeight::get().writes(2_u64))
						}
						/// Storage: Provenance Registries (r:1 w:1)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 100]`.
						fn update_registry(_a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `125`
						// Estimated: `3635`
						// Minimum execution time: 6_004_000 picoseconds.
						Weight::from_parts(6_927_986, 3635)
						.saturating_add(T::DbWeight::get().reads(1_u64))
						.saturating_add(T::DbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:1 w:1)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:1 w:0)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						fn remove_registry() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `125`
						// Estimated: `7243`
						// Minimum execution time: 7_204_000 picoseconds.
						Weight::from_parts(7_717_000, 7243)
						.saturating_add(T::DbWeight::get().reads(2_u64))
						.saturating_add(T::DbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Settings ExtrinsicExtra (r:1 w:0)
						/// Proof: Settings ExtrinsicExtra (max_values: None, max_size: Some(18), added: 2493, mode: MaxEncodedLen)
						/// Storage: Provenance NextDefinitionId (r:1 w:1)
						/// Proof: Provenance NextDefinitionId (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionSteps (r:0 w:500)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:0 w:1)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionStepsByAttestor (r:0 w:500)
						/// Proof: Provenance DefinitionStepsByAttestor (max_values: None, max_size: Some(76), added: 2551, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 100]`.
						/// The range of component `b` is `[1, 100]`.
						/// The range of component `c` is `[1, 500]`.
						fn create_definition(_a: u32, _b: u32, c: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `299`
						// Estimated: `8607`
						// Minimum execution time: 11_145_000 picoseconds.
						Weight::from_parts(28_881_975, 8607)
						// Standard Error: 14_923
						.saturating_add(Weight::from_parts(1_745_666, 0).saturating_mul(c.into()))
						.saturating_add(T::DbWeight::get().reads(3_u64))
						.saturating_add(T::DbWeight::get().writes(2_u64))
						.saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(c.into())))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:1 w:1)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						fn set_definition_active() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `281`
						// Estimated: `7243`
						// Minimum execution time: 8_517_000 picoseconds.
						Weight::from_parts(9_026_000, 7243)
						.saturating_add(T::DbWeight::get().reads(2_u64))
						.saturating_add(T::DbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:1 w:1)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						fn set_definition_inactive() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `281`
						// Estimated: `7243`
						// Minimum execution time: 8_941_000 picoseconds.
						Weight::from_parts(9_875_000, 7243)
						.saturating_add(T::DbWeight::get().reads(2_u64))
						.saturating_add(T::DbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:1 w:1)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						/// Storage: Provenance Processes (r:1 w:0)
						/// Proof: Provenance Processes (max_values: None, max_size: Some(147), added: 2622, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionChildren (r:1 w:0)
						/// Proof: Provenance DefinitionChildren (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionParents (r:1 w:0)
						/// Proof: Provenance DefinitionParents (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionSteps (r:501 w:500)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 500]`.
						fn remove_definition(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `266 + a * (166 ±0)`
						// Estimated: `21529 + a * (2658 ±0)`
						// Minimum execution time: 17_164_000 picoseconds.
						Weight::from_parts(17_182_000, 21529)
						// Standard Error: 14_807
						.saturating_add(Weight::from_parts(2_634_226, 0).saturating_mul(a.into()))
						.saturating_add(T::DbWeight::get().reads(6_u64))
						.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
						.saturating_add(T::DbWeight::get().writes(1_u64))
						.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(a.into())))
						.saturating_add(Weight::from_parts(0, 2658).saturating_mul(a.into()))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:1 w:0)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionSteps (r:1 w:1)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionStepsByAttestor (r:0 w:1)
						/// Proof: Provenance DefinitionStepsByAttestor (max_values: None, max_size: Some(76), added: 2551, mode: MaxEncodedLen)
						fn update_definition_step() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `456`
						// Estimated: `10891`
						// Minimum execution time: 12_386_000 picoseconds.
						Weight::from_parts(12_942_000, 10891)
						.saturating_add(T::DbWeight::get().reads(3_u64))
						.saturating_add(T::DbWeight::get().writes(2_u64))
						}
						/// Storage: Provenance Definitions (r:1 w:0)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionSteps (r:1 w:0)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// Storage: Settings ExtrinsicExtra (r:1 w:0)
						/// Proof: Settings ExtrinsicExtra (max_values: None, max_size: Some(18), added: 2493, mode: MaxEncodedLen)
						/// Storage: Provenance NextProcessId (r:1 w:1)
						/// Proof: Provenance NextProcessId (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
						/// Storage: Provenance Processes (r:0 w:1)
						/// Proof: Provenance Processes (max_values: None, max_size: Some(147), added: 2622, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 100]`.
						fn create_process(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `609`
						// Estimated: `12228`
						// Minimum execution time: 12_521_000 picoseconds.
						Weight::from_parts(13_980_393, 12228)
						// Standard Error: 4_132
						.saturating_add(Weight::from_parts(1_704, 0).saturating_mul(a.into()))
						.saturating_add(T::DbWeight::get().reads(4_u64))
						.saturating_add(T::DbWeight::get().writes(2_u64))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Processes (r:1 w:1)
						/// Proof: Provenance Processes (max_values: None, max_size: Some(147), added: 2622, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 100]`.
						fn update_process(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `260`
						// Estimated: `7247`
						// Minimum execution time: 8_252_000 picoseconds.
						Weight::from_parts(9_399_331, 7247)
						// Standard Error: 1_857
						.saturating_add(Weight::from_parts(1_155, 0).saturating_mul(a.into()))
						.saturating_add(T::DbWeight::get().reads(2_u64))
						.saturating_add(T::DbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Processes (r:1 w:1)
						/// Proof: Provenance Processes (max_values: None, max_size: Some(147), added: 2622, mode: MaxEncodedLen)
						/// Storage: Provenance ProcessSteps (r:1 w:0)
						/// Proof Skipped: Provenance ProcessSteps (max_values: None, max_size: None, mode: Measured)
						/// The range of component `a` is `[1, 500]`.
						fn remove_process(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `260`
						// Estimated: `10972`
						// Minimum execution time: 10_451_000 picoseconds.
						Weight::from_parts(12_448_091, 10972)
						// Standard Error: 2_259
						.saturating_add(Weight::from_parts(32_715, 0).saturating_mul(a.into()))
						.saturating_add(T::DbWeight::get().reads(3_u64))
						.saturating_add(T::DbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:2 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:2 w:0)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionChildren (r:0 w:1)
						/// Proof: Provenance DefinitionChildren (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionParents (r:0 w:1)
						/// Proof: Provenance DefinitionParents (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						fn add_child_definition() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `363`
						// Estimated: `12506`
						// Minimum execution time: 13_277_000 picoseconds.
						Weight::from_parts(13_576_000, 12506)
						.saturating_add(T::DbWeight::get().reads(4_u64))
						.saturating_add(T::DbWeight::get().writes(2_u64))
						}
						/// Storage: Provenance Registries (r:2 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionChildren (r:0 w:1)
						/// Proof: Provenance DefinitionChildren (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionParents (r:0 w:1)
						/// Proof: Provenance DefinitionParents (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						fn remove_child_definition() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `232`
						// Estimated: `6280`
						// Minimum execution time: 8_071_000 picoseconds.
						Weight::from_parts(8_558_000, 6280)
						.saturating_add(T::DbWeight::get().reads(2_u64))
						.saturating_add(T::DbWeight::get().writes(2_u64))
						}
						/// Storage: Provenance DefinitionSteps (r:2 w:0)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// Storage: Provenance ProcessSteps (r:2 w:1)
						/// Proof Skipped: Provenance ProcessSteps (max_values: None, max_size: None, mode: Measured)
						/// Storage: Timestamp Now (r:1 w:0)
						/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 500]`.
						/// The range of component `b` is `[1, 100]`.
						/// The range of component `c` is `[1, 100]`.
						fn attest_process_step(a: u32, b: u32, c: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `543`
						// Estimated: `14282`
						// Minimum execution time: 12_937_000 picoseconds.
						Weight::from_parts(13_382_000, 14282)
						// Standard Error: 1_810
						.saturating_add(Weight::from_parts(153_974, 0).saturating_mul(a.into()))
						// Standard Error: 9_075
						.saturating_add(Weight::from_parts(43_782, 0).saturating_mul(b.into()))
						// Standard Error: 9_075
						.saturating_add(Weight::from_parts(24_923, 0).saturating_mul(c.into()))
						.saturating_add(T::DbWeight::get().reads(5_u64))
						.saturating_add(T::DbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance DefinitionSteps (r:501 w:0)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// Storage: Provenance ProcessSteps (r:500 w:0)
						/// Proof Skipped: Provenance ProcessSteps (max_values: None, max_size: None, mode: Measured)
						/// Storage: Provenance Processes (r:1 w:1)
						/// Proof: Provenance Processes (max_values: None, max_size: Some(147), added: 2622, mode: MaxEncodedLen)
						/// The range of component `a` is `[2, 500]`.
						fn complete_process(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `254 + a * (200 ±0)`
						// Estimated: `8510 + a * (5334 ±0)`
						// Minimum execution time: 18_845_000 picoseconds.
						Weight::from_parts(19_058_000, 8510)
						// Standard Error: 19_033
						.saturating_add(Weight::from_parts(3_622_335, 0).saturating_mul(a.into()))
						.saturating_add(T::DbWeight::get().reads(2_u64))
						.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
						.saturating_add(T::DbWeight::get().writes(1_u64))
						.saturating_add(Weight::from_parts(0, 5334).saturating_mul(a.into()))
						}
						}

						// For backwards compatibility and tests
						impl WeightInfo for () {
						/// Storage: Settings ExtrinsicExtra (r:1 w:0)
						/// Proof: Settings ExtrinsicExtra (max_values: None, max_size: Some(18), added: 2493, mode: MaxEncodedLen)
						/// Storage: Provenance NextRegistryId (r:1 w:1)
						/// Proof: Provenance NextRegistryId (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
						/// Storage: Provenance Registries (r:0 w:1)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 100]`.
						fn create_registry(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `180`
						// Estimated: `4972`
						// Minimum execution time: 7_249_000 picoseconds.
						Weight::from_parts(8_111_025, 4972)
						// Standard Error: 6_368
						.saturating_add(Weight::from_parts(7_627, 0).saturating_mul(a.into()))
						.saturating_add(RocksDbWeight::get().reads(2_u64))
						.saturating_add(RocksDbWeight::get().writes(2_u64))
						}
						/// Storage: Provenance Registries (r:1 w:1)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 100]`.
						fn update_registry(_a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `125`
						// Estimated: `3635`
						// Minimum execution time: 6_004_000 picoseconds.
						Weight::from_parts(6_927_986, 3635)
						.saturating_add(RocksDbWeight::get().reads(1_u64))
						.saturating_add(RocksDbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:1 w:1)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:1 w:0)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						fn remove_registry() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `125`
						// Estimated: `7243`
						// Minimum execution time: 7_204_000 picoseconds.
						Weight::from_parts(7_717_000, 7243)
						.saturating_add(RocksDbWeight::get().reads(2_u64))
						.saturating_add(RocksDbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Settings ExtrinsicExtra (r:1 w:0)
						/// Proof: Settings ExtrinsicExtra (max_values: None, max_size: Some(18), added: 2493, mode: MaxEncodedLen)
						/// Storage: Provenance NextDefinitionId (r:1 w:1)
						/// Proof: Provenance NextDefinitionId (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionSteps (r:0 w:500)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:0 w:1)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionStepsByAttestor (r:0 w:500)
						/// Proof: Provenance DefinitionStepsByAttestor (max_values: None, max_size: Some(76), added: 2551, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 100]`.
						/// The range of component `b` is `[1, 100]`.
						/// The range of component `c` is `[1, 500]`.
						fn create_definition(_a: u32, _b: u32, c: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `299`
						// Estimated: `8607`
						// Minimum execution time: 11_145_000 picoseconds.
						Weight::from_parts(28_881_975, 8607)
						// Standard Error: 14_923
						.saturating_add(Weight::from_parts(1_745_666, 0).saturating_mul(c.into()))
						.saturating_add(RocksDbWeight::get().reads(3_u64))
						.saturating_add(RocksDbWeight::get().writes(2_u64))
						.saturating_add(RocksDbWeight::get().writes((2_u64).saturating_mul(c.into())))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:1 w:1)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						fn set_definition_active() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `281`
						// Estimated: `7243`
						// Minimum execution time: 8_517_000 picoseconds.
						Weight::from_parts(9_026_000, 7243)
						.saturating_add(RocksDbWeight::get().reads(2_u64))
						.saturating_add(RocksDbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:1 w:1)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						fn set_definition_inactive() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `281`
						// Estimated: `7243`
						// Minimum execution time: 8_941_000 picoseconds.
						Weight::from_parts(9_875_000, 7243)
						.saturating_add(RocksDbWeight::get().reads(2_u64))
						.saturating_add(RocksDbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:1 w:1)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						/// Storage: Provenance Processes (r:1 w:0)
						/// Proof: Provenance Processes (max_values: None, max_size: Some(147), added: 2622, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionChildren (r:1 w:0)
						/// Proof: Provenance DefinitionChildren (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionParents (r:1 w:0)
						/// Proof: Provenance DefinitionParents (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionSteps (r:501 w:500)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 500]`.
						fn remove_definition(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `266 + a * (166 ±0)`
						// Estimated: `21529 + a * (2658 ±0)`
						// Minimum execution time: 17_164_000 picoseconds.
						Weight::from_parts(17_182_000, 21529)
						// Standard Error: 14_807
						.saturating_add(Weight::from_parts(2_634_226, 0).saturating_mul(a.into()))
						.saturating_add(RocksDbWeight::get().reads(6_u64))
						.saturating_add(RocksDbWeight::get().reads((1_u64).saturating_mul(a.into())))
						.saturating_add(RocksDbWeight::get().writes(1_u64))
						.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(a.into())))
						.saturating_add(Weight::from_parts(0, 2658).saturating_mul(a.into()))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:1 w:0)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionSteps (r:1 w:1)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionStepsByAttestor (r:0 w:1)
						/// Proof: Provenance DefinitionStepsByAttestor (max_values: None, max_size: Some(76), added: 2551, mode: MaxEncodedLen)
						fn update_definition_step() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `456`
						// Estimated: `10891`
						// Minimum execution time: 12_386_000 picoseconds.
						Weight::from_parts(12_942_000, 10891)
						.saturating_add(RocksDbWeight::get().reads(3_u64))
						.saturating_add(RocksDbWeight::get().writes(2_u64))
						}
						/// Storage: Provenance Definitions (r:1 w:0)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionSteps (r:1 w:0)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// Storage: Settings ExtrinsicExtra (r:1 w:0)
						/// Proof: Settings ExtrinsicExtra (max_values: None, max_size: Some(18), added: 2493, mode: MaxEncodedLen)
						/// Storage: Provenance NextProcessId (r:1 w:1)
						/// Proof: Provenance NextProcessId (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
						/// Storage: Provenance Processes (r:0 w:1)
						/// Proof: Provenance Processes (max_values: None, max_size: Some(147), added: 2622, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 100]`.
						fn create_process(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `609`
						// Estimated: `12228`
						// Minimum execution time: 12_521_000 picoseconds.
						Weight::from_parts(13_980_393, 12228)
						// Standard Error: 4_132
						.saturating_add(Weight::from_parts(1_704, 0).saturating_mul(a.into()))
						.saturating_add(RocksDbWeight::get().reads(4_u64))
						.saturating_add(RocksDbWeight::get().writes(2_u64))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Processes (r:1 w:1)
						/// Proof: Provenance Processes (max_values: None, max_size: Some(147), added: 2622, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 100]`.
						fn update_process(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `260`
						// Estimated: `7247`
						// Minimum execution time: 8_252_000 picoseconds.
						Weight::from_parts(9_399_331, 7247)
						// Standard Error: 1_857
						.saturating_add(Weight::from_parts(1_155, 0).saturating_mul(a.into()))
						.saturating_add(RocksDbWeight::get().reads(2_u64))
						.saturating_add(RocksDbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:1 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Processes (r:1 w:1)
						/// Proof: Provenance Processes (max_values: None, max_size: Some(147), added: 2622, mode: MaxEncodedLen)
						/// Storage: Provenance ProcessSteps (r:1 w:0)
						/// Proof Skipped: Provenance ProcessSteps (max_values: None, max_size: None, mode: Measured)
						/// The range of component `a` is `[1, 500]`.
						fn remove_process(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `260`
						// Estimated: `10972`
						// Minimum execution time: 10_451_000 picoseconds.
						Weight::from_parts(12_448_091, 10972)
						// Standard Error: 2_259
						.saturating_add(Weight::from_parts(32_715, 0).saturating_mul(a.into()))
						.saturating_add(RocksDbWeight::get().reads(3_u64))
						.saturating_add(RocksDbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance Registries (r:2 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance Definitions (r:2 w:0)
						/// Proof: Provenance Definitions (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionChildren (r:0 w:1)
						/// Proof: Provenance DefinitionChildren (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionParents (r:0 w:1)
						/// Proof: Provenance DefinitionParents (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						fn add_child_definition() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `363`
						// Estimated: `12506`
						// Minimum execution time: 13_277_000 picoseconds.
						Weight::from_parts(13_576_000, 12506)
						.saturating_add(RocksDbWeight::get().reads(4_u64))
						.saturating_add(RocksDbWeight::get().writes(2_u64))
						}
						/// Storage: Provenance Registries (r:2 w:0)
						/// Proof: Provenance Registries (max_values: None, max_size: Some(170), added: 2645, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionChildren (r:0 w:1)
						/// Proof: Provenance DefinitionChildren (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						/// Storage: Provenance DefinitionParents (r:0 w:1)
						/// Proof: Provenance DefinitionParents (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
						fn remove_child_definition() -> Weight {
						// Proof Size summary in bytes:
						// Measured: `232`
						// Estimated: `6280`
						// Minimum execution time: 8_071_000 picoseconds.
						Weight::from_parts(8_558_000, 6280)
						.saturating_add(RocksDbWeight::get().reads(2_u64))
						.saturating_add(RocksDbWeight::get().writes(2_u64))
						}
						/// Storage: Provenance DefinitionSteps (r:2 w:0)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// Storage: Provenance ProcessSteps (r:2 w:1)
						/// Proof Skipped: Provenance ProcessSteps (max_values: None, max_size: None, mode: Measured)
						/// Storage: Timestamp Now (r:1 w:0)
						/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
						/// The range of component `a` is `[1, 500]`.
						/// The range of component `b` is `[1, 100]`.
						/// The range of component `c` is `[1, 100]`.
						fn attest_process_step(a: u32, b: u32, c: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `543`
						// Estimated: `14282`
						// Minimum execution time: 12_937_000 picoseconds.
						Weight::from_parts(13_382_000, 14282)
						// Standard Error: 1_810
						.saturating_add(Weight::from_parts(153_974, 0).saturating_mul(a.into()))
						// Standard Error: 9_075
						.saturating_add(Weight::from_parts(43_782, 0).saturating_mul(b.into()))
						// Standard Error: 9_075
						.saturating_add(Weight::from_parts(24_923, 0).saturating_mul(c.into()))
						.saturating_add(RocksDbWeight::get().reads(5_u64))
						.saturating_add(RocksDbWeight::get().writes(1_u64))
						}
						/// Storage: Provenance DefinitionSteps (r:501 w:0)
						/// Proof: Provenance DefinitionSteps (max_values: None, max_size: Some(183), added: 2658, mode: MaxEncodedLen)
						/// Storage: Provenance ProcessSteps (r:500 w:0)
						/// Proof Skipped: Provenance ProcessSteps (max_values: None, max_size: None, mode: Measured)
						/// Storage: Provenance Processes (r:1 w:1)
						/// Proof: Provenance Processes (max_values: None, max_size: Some(147), added: 2622, mode: MaxEncodedLen)
						/// The range of component `a` is `[2, 500]`.
						fn complete_process(a: u32, ) -> Weight {
						// Proof Size summary in bytes:
						// Measured: `254 + a * (200 ±0)`
						// Estimated: `8510 + a * (5334 ±0)`
						// Minimum execution time: 18_845_000 picoseconds.
						Weight::from_parts(19_058_000, 8510)
						// Standard Error: 19_033
						.saturating_add(Weight::from_parts(3_622_335, 0).saturating_mul(a.into()))
						.saturating_add(RocksDbWeight::get().reads(2_u64))
						.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(a.into())))
						.saturating_add(RocksDbWeight::get().writes(1_u64))
						.saturating_add(Weight::from_parts(0, 5334).saturating_mul(a.into()))
						}
						}