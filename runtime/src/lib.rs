//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
    WASM_BINARY.expect(
        "Development wasm binary is not available. This means the client is \
						built with `SKIP_WASM_BUILD` flag and it is only usable for \
						production chains. Please rebuild with the flag disabled.",
    )
}

pub mod constants;
mod payment;
pub mod primitives;
#[cfg(feature = "grandpa_babe")]
use codec::Encode;
pub use frame_support::{
    construct_runtime, debug, parameter_types,
    traits::{
        Currency, Imbalance, KeyOwnerProofSystem, OnUnbalanced, Randomness, U128CurrencyToVote,
    },
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        DispatchClass, IdentityFee, Weight,
    },
    StorageValue,
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureOneOf, EnsureRoot,
};
pub use pallet_balances::Call as BalancesCall;
#[cfg(feature = "grandpa_babe")]
use pallet_grandpa::{
    fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
#[cfg(feature = "grandpa_babe")]
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_primitives::*;
#[cfg(feature = "grandpa_babe")]
use pallet_session::historical as pallet_session_historical;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use primitives::{
    AccountId, Balance, BlockNumber, CatalogId, DefinitionId, DefinitionStepIndex, ExtrinsicIndex,
    GroupId, Hash, Index, MemberCount, ModuleIndex, Moment, ProcessId, ProposalId, RegistryId,
    Signature,
};
use sp_api::impl_runtime_apis;
#[cfg(feature = "grandpa_babe")]
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::{
    crypto::KeyTypeId,
    u32_trait::{_1, _2, _3, _4, _5},
    OpaqueMetadata,
};
use sp_inherents::{CheckInherentsResult, InherentData};
#[cfg(feature = "grandpa_babe")]
use sp_runtime::traits::{self, OpaqueKeys, SaturatedConversion};
use sp_runtime::{
    create_runtime_str, generic,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, FixedPointNumber, ModuleId, Perbill, Percent, Permill, Perquintill,
};
#[cfg(feature = "grandpa_babe")]
use sp_runtime::{
    curve::PiecewiseLinear, impl_opaque_keys, traits::NumberFor,
    transaction_validity::TransactionPriority,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

#[cfg(any(feature = "std", test))]
pub use pallet_staking::StakerStatus;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;

/// Constant values used within the runtime.
use constants::{currency::*, time::*};
#[cfg(feature = "grandpa_babe")]
use sp_runtime::generic::Era;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    #[cfg(feature = "grandpa_babe")]
    impl_opaque_keys! {
        pub struct SessionKeys {
            pub babe: Babe,
            pub grandpa: Grandpa,
            pub im_online: ImOnline,
            pub authority_discovery: AuthorityDiscovery,
        }
    }
}

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(fees) = fees_then_tips.next() {
            let treasury_share = Settings::fee_split_ratio();
            let author_share = 100 - treasury_share;

            let mut split = fees.ration(treasury_share, author_share);
            if let Some(tips) = fees_then_tips.next() {
                // for tips, if any,  (though this can be anything)
                tips.ration_merge_into(treasury_share, author_share, &mut split);
            }
            Treasury::on_unbalanced(split.0);
            //TODO: Do we need the Author pallet?
            // Author::on_unbalanced(split.1);
        }
    }
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("borlaug-chain"),
    impl_name: create_runtime_str!("borlaug-chain"),
    authoring_version: 2,
    // Per convention: if the runtime behavior changes, increment spec_version
    // and set impl_version to 0. If only runtime
    // implementation changes and behavior does not, then leave spec_version as
    // is and increment impl_version.
    spec_version: 3,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 2,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub const BlockHashCount: BlockNumber = 2400;
    pub RuntimeBlockLength: BlockLength =
    BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
    .base_block(BlockExecutionWeight::get())
    .for_class(DispatchClass::all(), |weights| {
        weights.base_extrinsic = ExtrinsicBaseWeight::get();
    })
    .for_class(DispatchClass::Normal, |weights| {
        weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
    })
    .for_class(DispatchClass::Operational, |weights| {
        weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
        // Operational transactions have some extra reserved space, so that they
        // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
        weights.reserved = Some(
            MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
        );
    })
    .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
    .build_or_panic();
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = ();
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
}
#[cfg(feature = "grandpa_babe")]
parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
    pub const ReportLongevity: u64 =
        BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}
#[cfg(feature = "grandpa_babe")]
impl pallet_babe::Config for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;

    type KeyOwnerProofSystem = Historical;

    type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::IdentificationTuple;

    type HandleEquivocation =
        pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;

    type WeightInfo = ();
}
#[cfg(feature = "grandpa_babe")]
parameter_types! {
    pub const SessionDuration: BlockNumber = EPOCH_DURATION_IN_SLOTS as _;
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
    /// We prioritize im-online heartbeats over election solution submission.
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
}
#[cfg(feature = "grandpa_babe")]
impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as traits::Verify>::Signer,
        account: AccountId,
        nonce: Index,
    ) -> Option<(
        Call,
        <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload,
    )> {
        let tip = 0;
        // take the biggest period possible.
        let period = BlockHashCount::get()
            .checked_next_power_of_two()
            .map(|c| c / 2)
            .unwrap_or(2) as u64;
        let current_block = System::block_number()
            .saturated_into::<u64>()
            // The `System::block_number` is initialized with `n+1`,
            // so the actual block number is `n`.
            .saturating_sub(1);
        let era = Era::mortal(period, current_block);
        let extra = (
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(era),
            frame_system::CheckNonce::<Runtime>::from(nonce),
            frame_system::CheckWeight::<Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
        );
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                debug::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        //TODO: is this correct?
        let address = sp_runtime::MultiAddress::Id(account);
        let (call, extra, _) = raw_payload.deconstruct();
        Some((call, (address, signature, extra)))
    }
}
#[cfg(feature = "grandpa_babe")]
impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as traits::Verify>::Signer;
    type Signature = Signature;
}
#[cfg(feature = "grandpa_babe")]
impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}
#[cfg(feature = "grandpa_babe")]
impl pallet_grandpa::Config for Runtime {
    type Event = Event;
    type Call = Call;

    type KeyOwnerProofSystem = Historical;

    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    type HandleEquivocation = pallet_grandpa::EquivocationHandler<
        Self::KeyOwnerIdentification,
        Offences,
        ReportLongevity,
    >;

    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = Moment;
    #[cfg(feature = "grandpa_babe")]
    type OnTimestampSet = Babe;
    #[cfg(feature = "instant_seal")]
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const UncleGenerations: BlockNumber = 5;
}

impl pallet_authorship::Config for Runtime {
    #[cfg(feature = "grandpa_babe")]
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    #[cfg(feature = "instant_seal")]
    type FindAuthor = ();
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    #[cfg(feature = "grandpa_babe")]
    type EventHandler = (Staking, ImOnline);
    #[cfg(feature = "instant_seal")]
    type EventHandler = ();
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TransactionByteFee: Balance = 10 * MILLIGRAM;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_transaction_payment::Config for Runtime {
    // type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees>;
    type OnChargeTransaction = payment::GroupsCurrencyAdapter<Balances, DealWithFees>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
    //TODO: put this back. Removed temporarily to make fee analysis easier
    // type FeeMultiplierUpdate =
    //     TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}
// parameter_types! {
//     pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
//     pub const CouncilMaxProposals: u32 = 100;
// }

// type GeneralCouncilInstance = collective::Instance1;

// impl collective::Trait<GeneralCouncilInstance> for Runtime {
//     type Origin = Origin;
//     type Proposal = Call;
//     type Event = Event;
//     type MotionDuration = CouncilMotionDuration;
//     type MaxProposals = CouncilMaxProposals;
//     type WeightInfo = ();
// }

// type GeneralCouncilMembershipInstance = membership::Instance1;

// impl membership::Trait<GeneralCouncilMembershipInstance> for Runtime {
//     type Event = Event;
//     type AddOrigin =
//         collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
//     type RemoveOrigin =
//         collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
//     type SwapOrigin =
//         collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
//     type ResetOrigin =
//         collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
//     type PrimeOrigin =
//         collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>;
//     type MembershipInitialized = GeneralCouncil;
//     type MembershipChanged = GeneralCouncil;
// }
#[cfg(feature = "grandpa_babe")]
impl_opaque_keys! {
    pub struct SessionKeys {
        pub grandpa: Grandpa,
        pub babe: Babe,
        pub im_online: ImOnline,
        pub authority_discovery: AuthorityDiscovery,
    }
}
#[cfg(feature = "grandpa_babe")]
parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}
#[cfg(feature = "grandpa_babe")]
impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}
#[cfg(feature = "grandpa_babe")]
impl pallet_session::historical::Config for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}
#[cfg(feature = "grandpa_babe")]
pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_025_000,
        max_inflation: 0_100_000,
        ideal_stake: 0_500_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}
#[cfg(feature = "grandpa_babe")]
parameter_types! {
    pub const SessionsPerEra: sp_staking::SessionIndex = 6;
    pub const BondingDuration: pallet_staking::EraIndex = 24 * 28;
    pub const SlashDeferDuration: pallet_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MaxNominatorRewardedPerValidator: u32 = 256;
    pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
    pub const MaxIterations: u32 = 10;
    // 0.05%. The higher the value, the more strict solution acceptance becomes.
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
    pub OffchainSolutionWeightLimit: Weight = RuntimeBlockWeights::get()
        .get(DispatchClass::Normal)
        .max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
        .saturating_sub(BlockExecutionWeight::get());
}
#[cfg(feature = "grandpa_babe")]
impl pallet_staking::Config for Runtime {
    type Currency = Balances;
    type UnixTime = Timestamp;
    type CurrencyToVote = U128CurrencyToVote;
    type RewardRemainder = Treasury;
    type Event = Event;
    type Slash = Treasury; // send the slashed funds to the treasury.
    type Reward = (); // rewards are minted from the void
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    /// A super-majority of the council can cancel the slash.
    type SlashCancelOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>,
    >;
    type SessionInterface = Self;
    type RewardCurve = RewardCurve;
    type NextNewSession = Session;
    type ElectionLookahead = ElectionLookahead;
    type Call = Call;
    type MaxIterations = MaxIterations;
    type MinSolutionScoreBump = MinSolutionScoreBump;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type UnsignedPriority = StakingUnsignedPriority;
    // The unsigned solution weight targeted by the OCW. We set it to the maximum possible value of
    // a single extrinsic.
    type OffchainSolutionWeightLimit = OffchainSolutionWeightLimit;
    type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 1 * GRAM;
    pub const SpendPeriod: BlockNumber = 1 * DAYS;
    pub const Burn: Permill = Permill::from_percent(50);
    pub const TipCountdown: BlockNumber = 1 * DAYS;
    pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: Balance = 1 * GRAM;
    pub const DataDepositPerByte: Balance = 10 * MILLIGRAM;
    pub const BountyDepositBase: Balance = 1 * GRAM;
    pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
    pub const TreasuryModuleId: ModuleId = ModuleId(*b"py/trsry");
    pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
    pub const MaximumReasonLength: u32 = 16384;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyValueMinimum: Balance = 5 * GRAM;
}

impl pallet_treasury::Config for Runtime {
    type ModuleId = TreasuryModuleId;
    type Currency = Balances;
    type ApproveOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
    >;
    type RejectOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
    >;
    type Event = Event;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = ();
    //TODO: is this correct?
    type SpendFunds = ();
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
}

// parameter_types! {
//     pub const TombstoneDeposit: Balance = deposit(
//         1,
//         sp_std::mem::size_of::<pallet_contracts::ContractInfo<Runtime>>() as u32
//     );
//     pub const DepositPerContract: Balance = TombstoneDeposit::get();
//     pub const DepositPerStorageByte: Balance = deposit(0, 1);
//     pub const DepositPerStorageItem: Balance = deposit(1, 0);
//     pub RentFraction: Perbill = Perbill::from_rational_approximation(1u32, 30 * DAYS);
//     pub const SurchargeReward: Balance = 150 * MILLICENTS;
//     pub const SignedClaimHandicap: u32 = 2;
//     pub const MaxDepth: u32 = 32;
//     pub const MaxValueSize: u32 = 16 * 1024;
//     // The lazy deletion runs inside on_initialize.
//     pub DeletionWeightLimit: Weight = AVERAGE_ON_INITIALIZE_RATIO *
//         RuntimeBlockWeights::get().max_block;
//     // The weight needed for decoding the queue should be less or equal than a fifth
//     // of the overall weight dedicated to the lazy deletion.
//     pub DeletionQueueDepth: u32 = ((DeletionWeightLimit::get() / (
//             <Runtime as pallet_contracts::Config>::WeightInfo::on_initialize_per_queue_item(1) -
//             <Runtime as pallet_contracts::Config>::WeightInfo::on_initialize_per_queue_item(0)
//         )) / 5) as u32;
// }

// impl pallet_contracts::Config for Runtime {
//     type Time = Timestamp;
//     type Randomness = RandomnessCollectiveFlip;
//     type Currency = Balances;
//     type Event = Event;
//     type RentPayment = ();
//     type SignedClaimHandicap = SignedClaimHandicap;
//     type TombstoneDeposit = TombstoneDeposit;
//     type DepositPerContract = DepositPerContract;
//     type DepositPerStorageByte = DepositPerStorageByte;
//     type DepositPerStorageItem = DepositPerStorageItem;
//     type RentFraction = RentFraction;
//     type SurchargeReward = SurchargeReward;
//     type MaxDepth = MaxDepth;
//     type MaxValueSize = MaxValueSize;
//     type WeightPrice = pallet_transaction_payment::Module<Self>;
//     type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
//     type ChainExtension = ();
//     type DeletionQueueDepth = DeletionQueueDepth;
//     type DeletionWeightLimit = DeletionWeightLimit;
// }
#[cfg(feature = "grandpa_babe")]
impl pallet_im_online::Config for Runtime {
    type AuthorityId = ImOnlineId;
    type Event = Event;
    type ValidatorSet = Historical;
    type SessionDuration = SessionDuration;
    #[cfg(feature = "grandpa_babe")]
    type ReportUnresponsiveness = Offences;
    #[cfg(feature = "instant_seal")]
    type ReportUnresponsiveness = ();
    type UnsignedPriority = ImOnlineUnsignedPriority;
    type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub OffencesWeightSoftLimit: Weight = Perbill::from_percent(60) *
        RuntimeBlockWeights::get().max_block;
}
#[cfg(feature = "grandpa_babe")]
impl pallet_offences::Config for Runtime {
    type Event = Event;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
    type WeightSoftLimit = OffencesWeightSoftLimit;
}
#[cfg(feature = "grandpa_babe")]
impl pallet_authority_discovery::Config for Runtime {}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const GroupMaxProposals: u32 = 100;
    pub const GroupMaxMembers: u32 = 100;
}

impl settings::Config for Runtime {
    type Event = Event;
    type ChangeSettingOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>,
    >;
    type ModuleIndex = ModuleIndex;
    type Balance = Balance;
    type ExtrinsicIndex = ExtrinsicIndex;
}

impl groups::Config for Runtime {
    type Origin = Origin;
    type GroupApprovalOrigin = groups::EnsureThreshold<Runtime>;
    type Proposal = Call;
    type GroupId = GroupId;
    type MemberCount = MemberCount;
    type ProposalId = ProposalId;
    type Currency = Balances;
    type Event = Event;
    type MaxProposals = GroupMaxProposals;
    type MaxMembers = GroupMaxMembers;
    type WeightInfo = groups::weights::SubstrateWeight<Runtime>;
}

impl identity::Config for Runtime {
    type CatalogId = u32;
    type Event = Event;
}

impl asset_registry::Config for Runtime {
    type RegistryId = RegistryId;
    type AssetId = u32;
    type LeaseId = u32;
    type Balance = Balance;
    type Event = Event;
}

impl audits::Config for Runtime {
    type AuditId = u32;
    type ControlPointId = u32;
    type EvidenceId = u32;
    type ObservationId = u32;
    type Event = Event;
}

impl provenance::Config for Runtime {
    type Origin = Origin;
    type GroupApprovalOrigin = groups::EnsureApproved<AccountId, GroupId, MemberCount>;
    type RegistryId = primitives::RegistryId;
    type DefinitionId = primitives::DefinitionId;
    type ProcessId = primitives::ProcessId;
    type Currency = Balances;
    type Event = Event;
    type GroupId = primitives::GroupId;
    type MemberCount = MemberCount;
    type GetExtrinsicExtraSource = Settings;
}
#[cfg(feature = "grandpa_babe")]
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>} ,
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},

        Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
        Historical: pallet_session_historical::{Module},
        Staking: pallet_staking::{Module, Call, Config<T>, Storage, Event<T>, ValidateUnsigned},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Authorship: pallet_authorship::{Module, Call, Storage, Inherent},

        Babe: pallet_babe::{Module, Call, Storage, Config, Inherent, ValidateUnsigned},

        Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event, ValidateUnsigned},
        TransactionPayment: pallet_transaction_payment::{Module, Storage},
        Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
        //BorlaugCommittee
        Council: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        Offences: pallet_offences::{Module, Call, Storage, Event},
        Treasury: pallet_treasury::{Module, Call, Storage, Config, Event<T>},
        ImOnline: pallet_im_online::{Module, Call, Storage, Event<T>, ValidateUnsigned, Config<T>},
        AuthorityDiscovery: pallet_authority_discovery::{Module, Call, Config},

        // Contracts: pallet_contracts::{Module, Call, Config<T>, Storage, Event<T>},

        // // Governance
        // GeneralCouncil: collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        // GeneralCouncilMembership: membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},

        // BWS Modules

        Groups: groups::{Module, Call, Storage, Origin<T>, Event<T>},
        //Borlaug
        Settings: settings::{Module, Call, Config<T>,Storage, Event<T>},
        Identity: identity::{Module, Call, Storage, Event<T>},
        AssetRegistry: asset_registry::{Module, Call, Storage, Event<T>},
        Audits: audits::{Module, Call, Storage, Event<T>},
        Provenance: provenance::{Module, Call,Storage, Event<T>},
    }
);
#[cfg(feature = "instant_seal")]
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},



        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Authorship: pallet_authorship::{Module, Call, Storage, Inherent},

        TransactionPayment: pallet_transaction_payment::{Module, Storage},
        Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
        Council: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},

        Treasury: pallet_treasury::{Module, Call, Storage, Config, Event<T>},



        // Contracts: pallet_contracts::{Module, Call, Config<T>, Storage, Event<T>},

        // // Governance
        // GeneralCouncil: collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        // GeneralCouncilMembership: membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},

        // BWS Modules

        Groups: groups::{Module, Call, Storage, Origin<T>, Event<T>},
        Settings: settings::{Module, Call, Config<T>, Storage, Event<T>},
        Identity: identity::{Module, Call, Storage, Event<T>},
        AssetRegistry: asset_registry::{Module, Call, Storage, Event<T>},
        Audits: audits::{Module, Call, Storage, Event<T>},
        Provenance: provenance::{Module, Call, Storage, Event<T>},
    }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllModules,
>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(source: TransactionSource,tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            Executive::validate_transaction(source,tx)
        }
    }


    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    #[cfg(feature = "grandpa_babe")]
    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((fg_primitives::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(fg_primitives::OpaqueKeyOwnershipProof::new)
        }
    }
    #[cfg(feature = "grandpa_babe")]
    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeGenesisConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: PRIMARY_PROBABILITY,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }
    }
    #[cfg(feature = "grandpa_babe")]
    impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<AuthorityDiscoveryId> {
            AuthorityDiscovery::authorities()
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }





//     impl pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber>
//     for Runtime
// {
//     fn call(
//         origin: AccountId,
//         dest: AccountId,
//         value: Balance,
//         gas_limit: u64,
//         input_data: Vec<u8>,
//     ) -> pallet_contracts_primitives::ContractExecResult {
//         Contracts::bare_call(origin, dest, value, gas_limit, input_data)
//     }

//     fn get_storage(
//         address: AccountId,
//         key: [u8; 32],
//     ) -> pallet_contracts_primitives::GetStorageResult {
//         Contracts::get_storage(address, key)
//     }

//     fn rent_projection(
//         address: AccountId,
//     ) -> pallet_contracts_primitives::RentProjectionResult<BlockNumber> {
//         Contracts::rent_projection(address)
//     }
// }


    impl groups_runtime_api::GroupsApi<Block,AccountId,GroupId,MemberCount> for Runtime {
        fn member_of(account:AccountId) -> Vec<GroupId>  {
            Groups::member_of(account)
        }
        fn get_group(group:GroupId) -> Option<Group<GroupId, AccountId, MemberCount>>{
            Groups::get_group(group)
        }
        fn get_sub_groups(group:GroupId) -> Option<Vec<(GroupId,Group<GroupId, AccountId, MemberCount>)>>{
            Groups::get_sub_groups(group)
        }
    }

    impl provenance_runtime_api::ProvenanceApi<Block,RegistryId,DefinitionId,ProcessId,GroupId, MemberCount,DefinitionStepIndex> for Runtime {
        fn get_registries(group_id: GroupId) -> Vec<(RegistryId,Registry)>  {
            Provenance::get_registries(group_id)
        }
        fn get_registry(group_id: GroupId,registry_id:RegistryId) ->Option<Registry>  {
            Provenance::get_registry(group_id,registry_id)
        }
        fn get_definitions(registry_id:RegistryId) -> Vec<(DefinitionId,Definition)>  {
            Provenance::get_definitions(registry_id)
        }
        fn get_definition(registry_id:RegistryId,definition_id:DefinitionId) -> Option<Definition>  {
            Provenance::get_definition(registry_id,definition_id)
        }
        fn get_definition_steps(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(DefinitionStepIndex,DefinitionStep<GroupId, MemberCount>)>  {
            Provenance::get_definition_steps(registry_id,definition_id)
        }
        fn get_processes(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(ProcessId,Process)>  {
            Provenance::get_processes(registry_id,definition_id)
        }
        fn get_process(registry_id:RegistryId,definition_id:DefinitionId,process_id:ProcessId) -> Option<Process>  {
            Provenance::get_process(registry_id,definition_id,process_id)
        }
        fn get_process_steps(registry_id:RegistryId,definition_id:DefinitionId,process_id:ProcessId) -> Vec<ProcessStep>  {
            Provenance::get_process_steps(registry_id,definition_id,process_id)
        }
        fn get_process_step(registry_id:RegistryId,definition_id:DefinitionId,process_id:ProcessId,definition_step_index:DefinitionStepIndex) -> Option<ProcessStep>  {
            Provenance::get_process_step(registry_id,definition_id,process_id,definition_step_index)
        }
    }
    impl identity_runtime_api::IdentityApi<Block,AccountId,CatalogId> for Runtime {
        fn get_catalogs(did:Did) -> Vec<(CatalogId,Catalog)> {
            Identity::get_catalogs(did)
        }
        fn get_catalog(owner_did:Did,catalog_id:CatalogId) -> Option<Catalog> {
            Identity::get_catalog(owner_did,catalog_id)
        }
        fn get_dids_in_catalog(catalog_id:CatalogId) -> Vec<(Did,Option<Vec<u8>>)>  {
            Identity::get_dids_in_catalog(catalog_id)
        }
        fn get_did_in_catalog(catalog_id:CatalogId,did:Did) -> Option<(Option<Vec<u8>>, DidDocument<AccountId>)>  {
            Identity::get_did_in_catalog(catalog_id,did)
        }
        fn get_did(did:Did) -> Option<DidDocument<AccountId>>  {
            Identity::get_did(did)
        }
        fn get_dids_by_subject( subject: AccountId) -> Vec<(Did, Option<Vec<u8>>)>  {
            Identity::get_dids_by_subject(subject)
        }
        fn get_dids_by_controller( controller: AccountId) -> Vec<(Did, Option<Vec<u8>>)>  {
            Identity::get_dids_by_controller(controller)
        }
    }

    impl settings_runtime_api::SettingsApi<Block,ModuleIndex,ExtrinsicIndex,Balance> for Runtime {
        fn get_fee_split_ratio() -> u32 {
            Settings::get_fee_split_ratio()
        }
        fn get_extrinsic_extras() ->  Vec<(ModuleIndex,Vec<(ExtrinsicIndex,Balance)>)> {
            Settings::get_extrinsic_extras()
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }


    impl sp_session::SessionKeys<Block> for Runtime {
        #[allow(unused_variables)]
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            #[cfg(feature = "grandpa_babe")]
            {
                opaque::SessionKeys::generate(seed)
            }
            #[cfg(feature = "instant_seal")]
            {
            Vec::new()
            }
        }
        #[allow(unused_variables)]
        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            #[cfg(feature = "grandpa_babe")]
            {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
            }
            #[cfg(feature = "instant_seal")]
            {
            None
            }
    }


    }



    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

            use frame_system_benchmarking::Module as SystemBench;
            impl frame_system_benchmarking::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_balances, Balances);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}
