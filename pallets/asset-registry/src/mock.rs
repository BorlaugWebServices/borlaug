//! Mocks for the module.

use crate as pallet_asset_registry;
use frame_support::parameter_types;
use frame_system::{self as system, EnsureOneOf, EnsureSigned};
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
        Groups: groups::{Module, Call, Storage, Event<T>, Origin<T>},
        Identity: identity::{Module, Call, Storage, Event<T>},
        AssetRegistry: pallet_asset_registry::{Module, Call, Storage, Event<T>},
    }
);

type AccountId = u64;
type Balance = u64;

parameter_types! {
    pub const NameLimit: u32 = 100;
    pub const FactStringLimit: u32 = 100;
}

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
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
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

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
}
impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

impl settings::Config for Test {
    type Event = Event;
    type ChangeSettingOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type ModuleIndex = u8;
    type ExtrinsicIndex = u8;
    type Currency = Balances;
    type Balance = Balance;
}

parameter_types! {
    pub const GroupMaxProposals: u32 = 100;
    pub const GroupMaxMembers: u32 = 100;
}

impl groups::Config for Test {
    type Origin = Origin;
    type GroupsOriginByGroupThreshold = groups::EnsureThreshold<Test>;
    type GroupsOriginByCallerThreshold = groups::EnsureApproved<Test>;
    type GroupsOriginAccountOrGroup =
        EnsureOneOf<AccountId, EnsureSigned<AccountId>, groups::EnsureApproved<Test>>;
    type GetExtrinsicExtraSource = Settings;
    type Proposal = Call;
    type GroupId = u32;
    type ProposalId = u32;
    type MemberCount = u32;
    type Currency = Balances;
    type Event = Event;
    type MaxProposals = GroupMaxProposals;
    type MaxMembers = GroupMaxMembers;
    type WeightInfo = ();
    type NameLimit = NameLimit;
}
parameter_types! {
    pub const PropertyLimit: u32 = 100;
    pub const StatementLimit: u32 = 100;
    pub const ControllerLimit: u32 = 100;
    pub const ClaimConsumerLimit: u32 = 100;
    pub const ClaimIssuerLimit: u32 = 100;
    pub const CatalogDidLimit: u32 = 100;
}
impl identity::Config for Test {
    type CatalogId = u32;
    type ClaimId = u32;
    type Event = Event;
    type WeightInfo = ();
    type NameLimit = NameLimit;
    type FactStringLimit = FactStringLimit;
    type PropertyLimit = PropertyLimit;
    type StatementLimit = StatementLimit;
    type ControllerLimit = ControllerLimit;
    type ClaimConsumerLimit = ClaimConsumerLimit;
    type ClaimIssuerLimit = ClaimIssuerLimit;
    type CatalogDidLimit = CatalogDidLimit;
}

parameter_types! {
    pub const AssetPropertyLimit: u32 = 100;
    pub const LeaseAssetLimit: u32 = 100;
}

impl pallet_asset_registry::Config for Test {
    type RegistryId = u32;
    type AssetId = u32;
    type LeaseId = u32;
    type Balance = u64;
    type Event = Event;
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
