//! Mocks for the module.

use crate as pallet_asset_registry;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64, EitherOfDiverse},
};
use frame_system::{self as system, EnsureSigned};
use runtime::{
    primitives::{FactStringLimit, NameLimit},
    AssetPropertyLimit, BulkDidLimit, BulkDidPropertyLimit, CatalogDidLimit, ClaimConsumerLimit,
    ClaimIssuerLimit, ControllerLimit, GroupChainLimit, GroupMaxMembers, GroupMaxProposalLength,
    GroupMaxProposals, LeaseAssetLimit, PropertyLimit, StatementLimit,
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
        Groups: groups,
        Identity: identity,
        Timestamp: timestamp,
        AssetRegistry: pallet_asset_registry,
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

impl groups::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Origin = RuntimeOrigin;
    type GroupsOriginByGroupThreshold = groups::EnsureThreshold<Test>;
    type GroupsOriginByCallerThreshold = groups::EnsureApproved<Test>;
    type GroupsOriginExecuted = groups::EnsureExecuted<Test>;
    type GroupsOriginAccountOrThreshold =
        EitherOfDiverse<EnsureSigned<AccountId>, groups::EnsureThreshold<Test>>;
    type GroupsOriginAccountOrApproved =
        EitherOfDiverse<EnsureSigned<AccountId>, groups::EnsureApproved<Test>>;
    type GroupsOriginAccountOrExecuted =
        EitherOfDiverse<EnsureSigned<AccountId>, groups::EnsureExecuted<Test>>;
    type Proposal = RuntimeCall;
    type GroupId = u32;
    type ProposalId = u32;
    type MemberCount = u32;
    type Currency = Balances;
    type MaxProposals = GroupMaxProposals;
    type MaxProposalLength = GroupMaxProposalLength;
    type MaxMembers = GroupMaxMembers;
    type WeightInfo = groups::weights::SubstrateWeight<Test>;
    type GetExtrinsicExtraSource = Settings;
    type NameLimit = NameLimit;
    type GroupChainLimit = GroupChainLimit;
}

impl identity::Config for Test {
    type CatalogId = u32;
    type ClaimId = u32;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type NameLimit = NameLimit;
    type FactStringLimit = FactStringLimit;
    type PropertyLimit = PropertyLimit;
    type StatementLimit = StatementLimit;
    type ControllerLimit = ControllerLimit;
    type ClaimConsumerLimit = ClaimConsumerLimit;
    type ClaimIssuerLimit = ClaimIssuerLimit;
    type CatalogDidLimit = CatalogDidLimit;
    type BulkDidLimit = BulkDidLimit;
    type BulkDidPropertyLimit = BulkDidPropertyLimit;
}

impl pallet_asset_registry::Config for Test {
    type RegistryId = u32;
    type AssetId = u32;
    type LeaseId = u32;
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type NameLimit = NameLimit;
    type FactStringLimit = FactStringLimit;
    type AssetPropertyLimit = AssetPropertyLimit;
    type LeaseAssetLimit = LeaseAssetLimit;
}
// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
