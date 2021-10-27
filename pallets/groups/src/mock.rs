//! Mocks for the module.

use crate as pallet_groups;
use frame_support::{ord_parameter_types, parameter_types};
use frame_system::{self as system, EnsureOneOf, EnsureSigned};
use runtime::{
    primitives::NameLimit, GroupChainLimit, GroupMaxMembers, GroupMaxProposalLength,
    GroupMaxProposals,
};
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
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Groups: pallet_groups::{Module, Call, Storage, Event<T>, Origin<T>},
        Settings: settings::{Module, Call, Config<T>,Storage, Event<T>},
    }
);

type AccountId = u64;
type ModuleIndex = u8;
type ExtrinsicIndex = u8;
type Balance = u64;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
    pub const ExistentialDeposit: u128 = 1;
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
    type AccountId = AccountId;
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

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = u128;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

impl pallet_groups::Config for Test {
    type Origin = Origin;
    type GroupsOriginByGroupThreshold = pallet_groups::EnsureThreshold<Test>;
    type GroupsOriginByCallerThreshold = pallet_groups::EnsureApproved<Test>;
    type GroupsOriginExecuted = pallet_groups::EnsureExecuted<Test>;
    type GroupsOriginAccountOrThreshold =
        EnsureOneOf<AccountId, EnsureSigned<AccountId>, pallet_groups::EnsureThreshold<Test>>;
    type GroupsOriginAccountOrApproved =
        EnsureOneOf<AccountId, EnsureSigned<AccountId>, pallet_groups::EnsureApproved<Test>>;
    type GroupsOriginAccountOrExecuted =
        EnsureOneOf<AccountId, EnsureSigned<AccountId>, pallet_groups::EnsureExecuted<Test>>;
    type Proposal = Call;
    type GroupId = u32;
    type ProposalId = u32;
    type MemberCount = u32;
    type Currency = Balances;
    type Event = Event;
    type MaxProposals = GroupMaxProposals;
    type MaxProposalLength = GroupMaxProposalLength;
    type MaxMembers = GroupMaxMembers;
    type WeightInfo = ();
    type GetExtrinsicExtraSource = Settings;
    type NameLimit = NameLimit;
    type GroupChainLimit = GroupChainLimit;
}

ord_parameter_types! {
    pub const One: AccountId = 1;

}

impl settings::Config for Test {
    type Event = Event;
    type ChangeSettingOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type ModuleIndex = ModuleIndex;
    type Currency = Balances;
    type Balance = Balance;
    type ExtrinsicIndex = ExtrinsicIndex;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 2_000_000_000u128)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
