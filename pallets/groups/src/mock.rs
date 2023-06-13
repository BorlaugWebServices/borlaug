//! Mocks for the module.

use crate as pallet_groups;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64, EitherOfDiverse},
};
use frame_system::{self as system, EnsureSigned};
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
        System: frame_system,
        Settings: settings,
        Balances: pallet_balances,
       GroupsModule: pallet_groups
    }
);

type AccountId = u64;
type Balance = u64;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<AccountId>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
    pub const MaxLocks: u32 = 10;
}
impl pallet_balances::Config for Test {
    type Balance = u64;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type HoldIdentifier = ();
    type MaxHolds = ();
}

impl settings::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type ChangeSettingOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type ModuleIndex = u8;
    type ExtrinsicIndex = u8;
    type Currency = Balances;
    type Balance = Balance;
    type WeightInfo = ();
}

impl pallet_groups::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Origin = RuntimeOrigin;
    type GroupsOriginByGroupThreshold = pallet_groups::EnsureThreshold<Test>;
    type GroupsOriginByCallerThreshold = pallet_groups::EnsureApproved<Test>;
    type GroupsOriginExecuted = pallet_groups::EnsureExecuted<Test>;
    type GroupsOriginAccountOrThreshold =
        EitherOfDiverse<EnsureSigned<AccountId>, pallet_groups::EnsureThreshold<Test>>;
    type GroupsOriginAccountOrApproved =
        EitherOfDiverse<EnsureSigned<AccountId>, pallet_groups::EnsureApproved<Test>>;
    type GroupsOriginAccountOrExecuted =
        EitherOfDiverse<EnsureSigned<AccountId>, pallet_groups::EnsureExecuted<Test>>;
    type Proposal = RuntimeCall;
    type GroupId = u32;
    type ProposalId = u32;
    type MemberCount = u32;
    type Currency = Balances;
    type MaxProposals = GroupMaxProposals;
    type MaxProposalLength = GroupMaxProposalLength;
    type MaxMembers = GroupMaxMembers;
    type WeightInfo = pallet_groups::weights::SubstrateWeight<Test>;
    type GetExtrinsicExtraSource = Settings;
    type NameLimit = NameLimit;
    type GroupChainLimit = GroupChainLimit;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 2_000_000_000u64),
            (2, 2_000_000_000u64),
            (3, 2_000_000_000u64),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
