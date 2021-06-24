//! Mocks for the module.

use crate as pallet_provenance;
use frame_support::parameter_types;
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Settings: settings::{Module, Call, Config<T>,Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Identity: identity::{Module, Call, Storage, Event<T>},
        Groups: groups::{Module, Call, Storage, Event<T>},
        Provenance: pallet_provenance::{Module, Call, Storage, Event<T>},
    }
);
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
}

pub const MILLISECS_PER_BLOCK: u64 = 5000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl identity::Config for Test {
    type CatalogId = u32;
    type Event = Event;
}

parameter_types! {
    pub const GroupMaxProposals: u32 = 100;
    pub const GroupMaxMembers: u32 = 100;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = u128;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

impl groups::Config for Test {
    type Proposal = Call;
    type GroupId = u32;
    type ProposalId = u32;
    type Currency = Balances;
    type Event = Event;
    type MaxProposals = GroupMaxProposals;
    type MaxMembers = GroupMaxMembers;
    type WeightInfo = groups::weights::SubstrateWeight<Test>;
}

impl pallet_provenance::Config for Test {
    type RegistryId = u32;
    type DefinitionId = u32;
    type ProcessId = u32;
    type Event = Event;
    type GroupId = u32;
    type GroupInfoSource = Groups;
    type GetExtrinsicExtraSource = Settings;
}

impl settings::Config for Test {
    type Event = Event;
    type ChangeSettingOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type ModuleIndex = u8;
    type ExtrinsicIndex = u8;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
