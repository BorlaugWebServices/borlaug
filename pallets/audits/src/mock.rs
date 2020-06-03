//! Mocks for the module.

#![cfg(test)]

use frame_support::{impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

use super::*;

impl_outer_origin! {
    pub enum Origin for Test  where system = frame_system {}
}

// For testing the module, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
    pub const MinimumPeriod: u64 = 1;
    pub const EpochDuration: u64 = 3;
    pub const ExpectedBlockTime: u64 = 1;
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(16);
}

impl frame_system::Trait for Test {
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<u64>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
}

impl timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

impl Trait for Test {
    type AuditId = u32;
    type ControlPointId = u32;
    type ObservationId = u32;
    type EvidenceId = u32;
    type Event = TestEvent;
}

mod audits {
    pub use crate::Event;
}

use frame_system as system;
impl_outer_event! {
    pub enum TestEvent for Test {
        system<T>,
        audits<T>,
    }
}
#[allow(dead_code)]
pub type System = frame_system::Module<Test>;
pub type Audits = Module<Test>;

pub struct ExtBuilder {
    #[allow(dead_code)]
    catalog_id: u32,
    #[allow(dead_code)]
    next_catalog_id: u32,
}

// Returns default values for genesis config
impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            catalog_id: 0,
            next_catalog_id: 1000,
        }
    }
}

impl ExtBuilder {
    // builds genesis config
    pub fn build(self) -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        t.into()
    }
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
