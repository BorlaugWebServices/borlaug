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
pub mod primitives;
use codec::{Decode, Encode, MaxEncodedLen};
pub use frame_support::{
    construct_runtime,
    dispatch::DispatchClass,
    parameter_types,
    traits::{
        ConstU128, ConstU32, ConstU64, Currency, Imbalance, InstanceFilter, KeyOwnerProofSystem,
        OnUnbalanced, Randomness, U128CurrencyToVote,
    },
    traits::{EitherOfDiverse, Get},
    weights::{
        constants::{
            BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
        },
        ConstantMultiplier, IdentityFee, Weight, WeightToFeeCoefficient,
    },
    BoundedVec, PalletId, RuntimeDebug, StorageValue,
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot, EnsureSigned, EnsureWithSuccess,
};
#[cfg(any(feature = "grandpa_babe", feature = "grandpa_aura"))]
use pallet_grandpa::{fg_primitives, AuthorityId as GrandpaId};
#[cfg(feature = "grandpa_babe")]
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_primitives::*;
#[cfg(feature = "grandpa_babe")]
use pallet_session::historical as pallet_session_historical;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
use primitives::{
    AccountId, AssetId, AuditId, Balance, BlockNumber, BoundedStringFact, BoundedStringName,
    CatalogId, ClaimId, ControlPointId, DefinitionId, DefinitionStepIndex, EvidenceId,
    ExtrinsicIndex, FactStringLimit, GroupId, Hash, Index, LeaseId, MemberCount, ModuleIndex,
    Moment, NameLimit, ObservationId, ProcessId, ProposalId, RegistryId, Signature,
};
use scale_info::TypeInfo;
use sp_api::impl_runtime_apis;
#[cfg(feature = "grandpa_babe")]
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
#[cfg(feature = "grandpa_aura")]
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::OpaqueMetadata;
use sp_inherents::{CheckInherentsResult, InherentData};
#[cfg(feature = "grandpa_babe")]
use sp_runtime::traits::{self, OpaqueKeys, SaturatedConversion};
#[cfg(feature = "grandpa_aura")]
use sp_runtime::traits::{self, SaturatedConversion};
use sp_runtime::{
    create_runtime_str, generic,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, Bounded},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, FixedPointNumber, KeyTypeId, Perbill, Percent, Permill, Perquintill,
};
#[cfg(feature = "grandpa_babe")]
use sp_runtime::{
    curve::PiecewiseLinear, impl_opaque_keys, traits::NumberFor,
    transaction_validity::TransactionPriority,
};
#[cfg(feature = "grandpa_aura")]
use sp_runtime::{impl_opaque_keys, traits::NumberFor};
use sp_std::prelude::*;
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(any(feature = "std", test))]
pub use frame_system::Call as SystemCall;
#[cfg(any(feature = "std", test))]
pub use pallet_balances::Call as BalancesCall;
#[cfg(any(feature = "std", test))]
pub use pallet_staking::StakerStatus;
#[cfg(any(feature = "std", test))]
pub use pallet_sudo::Call as SudoCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

/// Constant values used within the runtime.
use constants::{currency::*, time::*};
#[cfg(any(feature = "grandpa_babe", feature = "grandpa_aura"))]
use sp_runtime::generic::Era;

/// Digest item type.
pub type DigestItem = generic::DigestItem;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
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
    #[cfg(feature = "grandpa_aura")]
    impl_opaque_keys! {
        pub struct SessionKeys {
            pub aura: Aura,
            pub grandpa: Grandpa,
        }
    }
}

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        if let Some(author) = Authorship::author() {
            Balances::resolve_creating(&author, amount);
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

            Author::on_unbalanced(split.1);
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
    spec_version: 23,
    impl_version: 0,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 2,
    state_version: 1,
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
const MAXIMUM_BLOCK_WEIGHT: Weight =
    Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), u64::MAX);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
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
    type BaseCallFilter = frame_support::traits::Everything;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type RuntimeCall = RuntimeCall;
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
    type RuntimeEvent = RuntimeEvent;
    /// The ubiquitous origin type.
    type RuntimeOrigin = RuntimeOrigin;
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
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
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

#[cfg(feature = "grandpa_aura")]
impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<32>;
}

#[cfg(feature = "grandpa_babe")]
parameter_types! {
    pub const SessionDuration: BlockNumber = EPOCH_DURATION_IN_SLOTS as _;
    pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
    /// We prioritize im-online heartbeats over election solution submission.
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
}
#[cfg(any(feature = "grandpa_babe", feature = "grandpa_aura"))]
impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    RuntimeCall: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: RuntimeCall,
        public: <Signature as traits::Verify>::Signer,
        account: AccountId,
        nonce: Index,
    ) -> Option<(
        RuntimeCall,
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
            frame_system::CheckNonZeroSender::<Runtime>::new(),
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
                log::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        //TODO: is this correct?
        let address = sp_runtime::MultiAddress::Id(account);
        let (call, extra, _) = raw_payload.deconstruct();
        Some((call, (address, signature, extra)))
    }
}

#[cfg(any(feature = "grandpa_babe", feature = "grandpa_aura"))]
impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as traits::Verify>::Signer;
    type Signature = Signature;
}
#[cfg(any(feature = "grandpa_babe", feature = "grandpa_aura"))]
impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    RuntimeCall: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = RuntimeCall;
}

#[cfg(feature = "grandpa_babe")]
parameter_types! {
    pub const MaxAuthorities: u32 = 100;
    pub const MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
}

#[cfg(feature = "grandpa_babe")]
impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = MaxAuthorities;
    type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
    type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
    type EquivocationReportSystem =
        pallet_grandpa::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

#[cfg(feature = "grandpa_aura")]
impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = Moment;
    #[cfg(feature = "grandpa_babe")]
    type OnTimestampSet = Babe;
    #[cfg(feature = "grandpa_aura")]
    type OnTimestampSet = Aura;
    #[cfg(feature = "instant_seal")]
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

parameter_types! {
    pub const UncleGenerations: BlockNumber = 5;
}

#[cfg(feature = "grandpa_babe")]
impl pallet_authorship::Config for Runtime {
    type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
    type EventHandler = (Staking, ImOnline);
}

#[cfg(any(feature = "grandpa_aura", feature = "instant_seal"))]
impl pallet_authorship::Config for Runtime {
    type FindAuthor = ();
    type EventHandler = ();
}

pub const EXISTENTIAL_DEPOSIT: u128 = 500;

/// A reason for placing a hold on funds.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, MaxEncodedLen, Debug, TypeInfo,
)]
pub enum HoldReason {
    /// The NIS Pallet has reserved it for a non-fungible receipt.
    Nis,
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = frame_system::Pallet<Runtime>;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type HoldIdentifier = HoldReason;
    type MaxHolds = ConstU32<1>;
}

parameter_types! {
    pub const TransactionByteFee: Balance = 10 * MILLIGRAM;
    pub const OperationalFeeMultiplier: u8 = 5;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
    pub MaximumMultiplier: Multiplier = Bounded::max_value();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = TargetedFeeAdjustment<
        Self,
        TargetBlockFullness,
        AdjustmentVariable,
        MinimumMultiplier,
        MaximumMultiplier,
    >;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
}
// parameter_types! {
//     pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
//     pub const CouncilMaxProposals: u32 = 100;
// }

// type GeneralCouncilInstance = collective::Instance1;

// impl collective::Trait<GeneralCouncilInstance> for Runtime {
//     type Origin = Origin;
//     type Proposal = Call;
//     type RuntimeEvent = RuntimeEvent;
//     type MotionDuration = CouncilMotionDuration;
//     type MaxProposals = CouncilMaxProposals;
//     type WeightInfo = ();
// }

// type GeneralCouncilMembershipInstance = membership::Instance1;

// impl membership::Trait<GeneralCouncilMembershipInstance> for Runtime {
//     type RuntimeEvent = RuntimeEvent;
//     type AddOrigin =
//         collective::EnsureProportionMoreThan<3, 4, AccountId, GeneralCouncilInstance>;
//     type RemoveOrigin =
//         collective::EnsureProportionMoreThan<3, 4, AccountId, GeneralCouncilInstance>;
//     type SwapOrigin =
//         collective::EnsureProportionMoreThan<3, 4, AccountId, GeneralCouncilInstance>;
//     type ResetOrigin =
//         collective::EnsureProportionMoreThan<3, 4, AccountId, GeneralCouncilInstance>;
//     type PrimeOrigin =
//         collective::EnsureProportionMoreThan<1, 2, AccountId, GeneralCouncilInstance>;
//     type MembershipInitialized = GeneralCouncil;
//     type MembershipChanged = GeneralCouncil;
// }

#[cfg(feature = "grandpa_babe")]
impl pallet_session::Config for Runtime {
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type ShouldEndSession = Babe;
    type NextSessionRotation = Babe;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
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
    type RuntimeEvent = RuntimeEvent;
    type Slash = Treasury; // send the slashed funds to the treasury.
    type Reward = (); // rewards are minted from the void
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    /// A super-majority of the council can cancel the slash.
    type SlashCancelOrigin = EitherOfDiverse<
        AccountId,
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>,
    >;
    type SessionInterface = Self;
    type RewardCurve = RewardCurve;
    type NextNewSession = Session;
    type ElectionLookahead = ElectionLookahead;
    type RuntimeCall = RuntimeCall;
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
    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
    pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
    pub const MaximumReasonLength: u32 = 16384;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyValueMinimum: Balance = 5 * GRAM;
    pub const MaxApprovals: u32 = 100;
    pub const MaxBalance: Balance = Balance::max_value();
}

impl pallet_treasury::Config for Runtime {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type ApproveOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
    >;
    type RejectOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
    >;
    type RuntimeEvent = RuntimeEvent;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type ProposalBondMaximum = ();
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = ();
    //TODO: is this correct
    type SpendFunds = ();
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
    type MaxApprovals = MaxApprovals;
    type SpendOrigin = EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, MaxBalance>;
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
//     type Randomness = CollectiveFlip;
//     type Currency = Balances;
//     type RuntimeEvent = RuntimeEvent;
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
    type RuntimeEvent = RuntimeEvent;
    type NextSessionRotation = Babe;
    type ValidatorSet = Historical;
    type ReportUnresponsiveness = Offences;
    type UnsignedPriority = ImOnlineUnsignedPriority;
    type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
    type MaxKeys = MaxKeys;
    type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
    type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
}
#[cfg(feature = "grandpa_babe")]
parameter_types! {
    pub OffencesWeightSoftLimit: Weight = Perbill::from_percent(60) *
        RuntimeBlockWeights::get().max_block;
}
#[cfg(feature = "grandpa_babe")]
impl pallet_offences::Config for Runtime {
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    #[cfg(feature = "grandpa_babe")]
    type OnOffenceHandler = Staking;
    #[cfg(feature = "grandpa_aura")]
    type OnOffenceHandler = ();
    type WeightSoftLimit = OffencesWeightSoftLimit;
}
#[cfg(feature = "grandpa_babe")]
impl pallet_authority_discovery::Config for Runtime {}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
    // pub MaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
    type SetMembersOrigin = EnsureRoot<Self::AccountId>;
    // type MaxProposalWeight = MaxCollectivesProposalWeight;
}

parameter_types! {
    // One storage item; key size 32, value size 8; .
    pub const ProxyDepositBase: Balance = deposit(1, 8);
    // Additional storage item size of 33 bytes.
    pub const ProxyDepositFactor: Balance = deposit(0, 33);
    pub const MaxProxies: u16 = 32;
    pub const AnnouncementDepositBase: Balance = deposit(1, 8);
    pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
    pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    RuntimeDebug,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum ProxyType {
    Any,
    NonTransfer,
    Governance,
    #[cfg(feature = "grandpa_babe")]
    Staking,
}

impl Default for ProxyType {
    fn default() -> Self {
        Self::Any
    }
}
impl InstanceFilter<RuntimeCall> for ProxyType {
    fn filter(&self, c: &RuntimeCall) -> bool {
        match self {
            ProxyType::Any => true,
            ProxyType::NonTransfer => !matches!(c, RuntimeCall::Balances(..)),
            ProxyType::Governance => {
                matches!(c, RuntimeCall::Council(..) | RuntimeCall::Treasury(..))
            }
            #[cfg(feature = "grandpa_babe")]
            ProxyType::Staking => matches!(c, RuntimeCall::Staking(..)),
        }
    }
    fn is_superset(&self, o: &Self) -> bool {
        match (self, o) {
            (x, y) if x == y => true,
            (ProxyType::Any, _) => true,
            (_, ProxyType::Any) => false,
            (ProxyType::NonTransfer, _) => true,
            _ => false,
        }
    }
}

impl pallet_proxy::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type ProxyType = ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = MaxProxies;
    type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
    type MaxPending = MaxPending;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

impl settings::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = settings::weights::SubstrateWeight<Runtime>;
    type ChangeSettingOrigin = EitherOfDiverse<
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>,
    >;
    type ModuleIndex = ModuleIndex;
    type Currency = Balances;
    type Balance = Balance;
    type ExtrinsicIndex = ExtrinsicIndex;
}

parameter_types! {
    pub const GroupMaxProposals: u32 = 100;
    pub const GroupMaxProposalLength: u32 = 1000;
    pub const GroupMaxMembers: u32 = 100;
    pub const GroupChainLimit: u32 = 100;
}

impl groups::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Origin = RuntimeOrigin;
    type GroupsOriginByGroupThreshold = groups::EnsureThreshold<Runtime>;
    type GroupsOriginByCallerThreshold = groups::EnsureApproved<Runtime>;
    type GroupsOriginExecuted = groups::EnsureExecuted<Runtime>;
    type GroupsOriginAccountOrThreshold =
        EitherOfDiverse<EnsureSigned<AccountId>, groups::EnsureThreshold<Runtime>>;
    type GroupsOriginAccountOrApproved =
        EitherOfDiverse<EnsureSigned<AccountId>, groups::EnsureApproved<Runtime>>;
    type GroupsOriginAccountOrExecuted =
        EitherOfDiverse<EnsureSigned<AccountId>, groups::EnsureExecuted<Runtime>>;
    type Proposal = RuntimeCall;
    type GroupId = GroupId;
    type ProposalId = ProposalId;
    type MemberCount = MemberCount;
    type Currency = Balances;
    type MaxProposals = GroupMaxProposals;
    type MaxProposalLength = GroupMaxProposalLength;
    type MaxMembers = GroupMaxMembers;
    type WeightInfo = groups::weights::SubstrateWeight<Runtime>;
    type GetExtrinsicExtraSource = Settings;
    type NameLimit = NameLimit;
    type GroupChainLimit = GroupChainLimit;
}

parameter_types! {
    pub const PropertyLimit: u32 = 500;
    pub const StatementLimit: u32 = 500;
    pub const ControllerLimit: u32 = 50;
    pub const ClaimConsumerLimit: u32 = 50;
    pub const ClaimIssuerLimit: u32 = 50;
    pub const CatalogDidLimit: u32 = 1_000;
    pub const BulkDidLimit: u32 = 15;
    pub const BulkDidPropertyLimit: u32 = 50;
}
impl identity::Config for Runtime {
    type CatalogId = CatalogId;
    type ClaimId = ClaimId;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = identity::weights::SubstrateWeight<Runtime>;
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
parameter_types! {
    pub const AssetPropertyLimit: u32 = 500;
    pub const LeaseAssetLimit: u32 = 500;
}
impl asset_registry::Config for Runtime {
    type RegistryId = RegistryId;
    type AssetId = AssetId;
    type LeaseId = LeaseId;
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = asset_registry::weights::SubstrateWeight<Runtime>;
    type NameLimit = NameLimit;
    type FactStringLimit = FactStringLimit;
    type AssetPropertyLimit = AssetPropertyLimit;
    type LeaseAssetLimit = LeaseAssetLimit;
}

parameter_types! {
    pub const MaxLinkRemove: u32 = 50;
    pub const UrlLimit: u32 = 500;
}
pub type BoundedStringUrl = BoundedVec<u8, UrlLimit>;
impl audits::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AuditId = AuditId;
    type ControlPointId = ControlPointId;
    type EvidenceId = EvidenceId;
    type ObservationId = ObservationId;
    type WeightInfo = audits::weights::SubstrateWeight<Runtime>;
    type NameLimit = NameLimit;
    type UrlLimit = UrlLimit;
    type MaxLinkRemove = MaxLinkRemove;
}
parameter_types! {
    pub const DefinitionStepLimit: u32 = 500;
    pub const AttributeLimit: u32 = 500;
}
impl provenance::Config for Runtime {
    type RegistryId = primitives::RegistryId;
    type DefinitionId = primitives::DefinitionId;
    type DefinitionStepIndex = primitives::DefinitionStepIndex;
    type ProcessId = primitives::ProcessId;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = provenance::weights::SubstrateWeight<Runtime>;
    type NameLimit = NameLimit;
    type FactStringLimit = FactStringLimit;
    type DefinitionStepLimit = DefinitionStepLimit;
    type AttributeLimit = AttributeLimit;
    type GetExtrinsicExtraSource = Settings;
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

#[cfg(feature = "grandpa_babe")]
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system ,
        CollectiveFlip: pallet_insecure_randomness_collective_flip,
        Balances: pallet_balances,
        Session: pallet_session,
        Historical: pallet_session_historical,
        Staking: pallet_staking,
        Timestamp: pallet_timestamp,
        Authorship: pallet_authorship,
        Babe: pallet_babe,
        Grandpa: pallet_grandpa,
        TransactionPayment: pallet_transaction_payment,
        Sudo: pallet_sudo,
        //BorlaugCommittee
        Council: pallet_collective::<Instance1>,
        Offences: pallet_offences,
        Treasury: pallet_treasury,
        ImOnline: pallet_im_online,
        AuthorityDiscovery: pallet_authority_discovery,
        Proxy: pallet_proxy,

        // Contracts: pallet_contracts,

        // // Governance
        // GeneralCouncil: collective::<Instance1>,
        // GeneralCouncilMembership: membership::<Instance1>,

        // BWS Modules

        Groups: groups,
        //Borlaug
        Settings: settings,
        Identity: identity,
        AssetRegistry: asset_registry,
        Audits: audits,
        Provenance: provenance,
    }
);
#[cfg(feature = "grandpa_aura")]
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system ,
        CollectiveFlip: pallet_insecure_randomness_collective_flip,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Authorship: pallet_authorship,
        // Session: pallet_session,
        // Historical: pallet_session_historical,
        Aura: pallet_aura,
        Grandpa: pallet_grandpa,
        TransactionPayment: pallet_transaction_payment,
        Sudo: pallet_sudo,
        //BorlaugCommittee
        Council: pallet_collective::<Instance1>,
        // Offences: pallet_offences,
        Treasury: pallet_treasury,
        // ImOnline: pallet_im_online,
        Proxy: pallet_proxy,
        // Contracts: pallet_contracts,

        // // Governance
        // GeneralCouncil: collective::<Instance1>,
        // GeneralCouncilMembership: membership::<Instance1>,

        // BWS Modules

        Groups: groups,
        //Borlaug
        Settings: settings,
        Identity: identity,
        AssetRegistry: asset_registry,
        Audits: audits,
        Provenance: provenance,
    }
);
#[cfg(feature = "instant_seal")]
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system,
        CollectiveFlip: pallet_insecure_randomness_collective_flip,
        Balances: pallet_balances,

        Timestamp: pallet_timestamp,
        Authorship: pallet_authorship,

        TransactionPayment: pallet_transaction_payment,
        Sudo: pallet_sudo,
        Council: pallet_collective::<Instance1>,

        Treasury: pallet_treasury,

        Proxy: pallet_proxy,

        // Contracts: pallet_contracts,

        // // Governance
        // GeneralCouncil: collective::<Instance1>,
        // GeneralCouncilMembership: membership::<Instance1>,

        // BWS Modules

        Groups: groups,
        Settings: settings,
        Identity: identity,
        AssetRegistry: asset_registry,
        Audits: audits,
        Provenance: provenance,
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
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    //TODO: do we need to add Migrations struct for runtime upgrades
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        //TODO: fix this
        // [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_timestamp, Timestamp]
        [pallet_groups, Groups]
        [pallet_identity, Identity]
        [pallet_audits, Audits]
        [pallet_provenance, Provenance]
        [pallet_asset_registry, AssetRegistry]
        [pallet_settings, Settings]
    );
}

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
            OpaqueMetadata::new(Runtime::metadata().into())
        }
        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
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

    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }


    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    #[cfg(feature = "grandpa_aura")]
    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities().into_inner()
        }
    }


    impl sp_session::SessionKeys<Block> for Runtime {
        #[allow(unused_variables)]
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            #[cfg(any(feature = "grandpa_babe",feature = "grandpa_aura"))]
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
            #[cfg(any(feature = "grandpa_babe",feature = "grandpa_aura"))]
            {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
            }
            #[cfg(feature = "instant_seal")]
            {
            None
            }
        }
    }


    #[cfg(any(feature = "grandpa_babe",feature = "grandpa_aura"))]
    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> fg_primitives::AuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> fg_primitives::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            #[cfg(feature = "grandpa_babe")]
            {
                let key_owner_proof = _key_owner_proof.decode()?;
                Grandpa::submit_unsigned_equivocation_report(
                    _equivocation_proof,
                    key_owner_proof,
                )
            }
            #[cfg(feature = "grandpa_aura")]
            {
                None
            }
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            #[cfg(feature = "grandpa_babe")]
            {
                use codec::Encode;

                Historical::prove((fg_primitives::KEY_TYPE, _authority_id))
                    .map(|p| p.encode())
                    .map(fg_primitives::OpaqueKeyOwnershipProof::new)
            }
            #[cfg(feature = "grandpa_aura")]
            {
                None
            }
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



    #[cfg(any(feature = "grandpa_babe"))]
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


    impl groups_runtime_api::GroupsApi<Block,AccountId,GroupId,MemberCount,ProposalId,Hash,BoundedStringName,Balance> for Runtime {
        fn member_of(account_id:AccountId) -> Vec<(GroupId, Group<GroupId, AccountId, MemberCount, BoundedStringName>,Vec<(AccountId, MemberCount)>,Balance)>  {
            Groups::member_of(account_id)
        }
        fn is_member(group_id:GroupId,account_id:AccountId) -> bool  {
            Groups::is_member(group_id,&account_id)
        }
        fn get_group_by_account(account_id:AccountId) -> Option<(GroupId,Group<GroupId, AccountId, MemberCount,BoundedStringName>,Vec<(AccountId, MemberCount)>,Balance)>{
            Groups::get_group_by_account(account_id)
        }
        fn get_group_account(group_id:GroupId) -> Option<AccountId>  {
            Groups::get_group_account(group_id)
        }
        fn get_group(group_id:GroupId) -> Option<(Group<GroupId, AccountId, MemberCount,BoundedStringName>,Vec<(AccountId, MemberCount)>,Balance)>{
            Groups::get_group(group_id)
        }
        fn get_sub_groups(group_id:GroupId) -> Vec<(GroupId,Group<GroupId, AccountId, MemberCount,BoundedStringName>,Vec<(AccountId, MemberCount)>,Balance)>{
            Groups::get_sub_groups(group_id)
        }
        fn get_proposal(proposal_id:ProposalId) ->Option<(ProposalId, GroupId,Vec<(AccountId, MemberCount)>, Option<(Hash,u32)>,Votes<AccountId, MemberCount>)>{
            Groups::get_proposal(proposal_id)
        }
        fn get_proposals_by_group(group_id:GroupId) -> Vec<(ProposalId, GroupId,Vec<(AccountId, MemberCount)>,Option<(Hash,u32)>,Votes<AccountId, MemberCount>)>{
            Groups::get_proposals_by_group(group_id)
        }
        fn get_proposals_by_account(account_id: AccountId) -> Vec<(GroupId, Vec<(ProposalId, GroupId,Vec<(AccountId, MemberCount)>,Option<(Hash,u32)>,Votes<AccountId, MemberCount>)>)>{
            Groups::get_proposals_by_account(account_id)
        }
    }

    impl asset_registry_runtime_api::AssetRegistryApi<Block,AccountId,ProposalId,RegistryId,AssetId,LeaseId,Moment,Balance,BoundedStringName,BoundedStringFact> for Runtime {
        fn get_registries(did: Did) -> Vec<(RegistryId,Registry<BoundedStringName>)>  {
            AssetRegistry::get_registries(did)
        }
        fn get_registry(did: Did,registry_id:RegistryId) -> Option<Registry<BoundedStringName>>{
            AssetRegistry::get_registry(did,registry_id)
        }
        fn get_assets(registry_id:RegistryId) -> Vec<(AssetId,Asset<Moment,Balance,BoundedStringName,BoundedStringFact>)>{
            AssetRegistry::get_assets(registry_id)
        }
        fn get_asset(registry_id:RegistryId, asset_id:AssetId) -> Option<Asset<Moment,Balance,BoundedStringName,BoundedStringFact>>{
            AssetRegistry::get_asset(registry_id,asset_id)
        }
        fn get_leases(lessor: Did) -> Vec<(LeaseId,LeaseAgreement<ProposalId,RegistryId,AssetId,Moment,BoundedStringName>)>{
            AssetRegistry::get_leases(lessor)
        }
        fn get_lease(lessor: Did, lease_id:LeaseId) -> Option<LeaseAgreement<ProposalId,RegistryId,AssetId,Moment,BoundedStringName>>{
            AssetRegistry::get_lease(lessor,lease_id)
        }
        fn get_lease_allocations(registry_id:RegistryId, asset_id:AssetId) -> Option<Vec<(LeaseId, u64, Moment)>>{
            AssetRegistry::get_lease_allocations(registry_id,asset_id)
        }
    }

    impl provenance_runtime_api::ProvenanceApi<Block,AccountId,RegistryId,DefinitionId,ProcessId, ProposalId,Moment,MemberCount,DefinitionStepIndex,BoundedStringName,BoundedStringFact> for Runtime {
        fn get_registries(account_id: AccountId) -> Vec<(RegistryId,Registry<BoundedStringName>)>  {
            Provenance::get_registries(account_id)
        }
        fn get_registry(account_id: AccountId,registry_id:RegistryId) ->Option<Registry<BoundedStringName>>  {
            Provenance::get_registry(account_id,registry_id)
        }
        fn get_definitions(registry_id:RegistryId) -> Vec<(DefinitionId,Definition<BoundedStringName>)>  {
            Provenance::get_definitions(registry_id)
        }
        fn get_definition(registry_id:RegistryId,definition_id:DefinitionId) -> Option<Definition<BoundedStringName>>  {
            Provenance::get_definition(registry_id,definition_id)
        }
        fn get_definition_step(registry_id:RegistryId,definition_id:DefinitionId,step_index: DefinitionStepIndex) -> Option<DefinitionStep<AccountId, MemberCount,BoundedStringName>>  {
            Provenance::get_definition_step(registry_id,definition_id,step_index)
        }
        fn get_definition_steps(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(DefinitionStepIndex,DefinitionStep<AccountId, MemberCount,BoundedStringName>)>  {
            Provenance::get_definition_steps(registry_id,definition_id)
        }
        fn get_available_definitions(account_id:AccountId) -> Vec<(RegistryId,DefinitionId,Definition<BoundedStringName>)>   {
            Provenance::get_available_definitions(account_id)
        }
        fn get_processes(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(ProcessId,Process<BoundedStringName>)>  {
            Provenance::get_processes(registry_id,definition_id)
        }
        fn get_process(registry_id:RegistryId,definition_id:DefinitionId,process_id:ProcessId) -> Option<Process<BoundedStringName>>  {
            Provenance::get_process(registry_id,definition_id,process_id)
        }
        fn get_processes_for_attestor_by_status(account_id: AccountId,status: ProcessStatus) -> Vec<(RegistryId,DefinitionId,ProcessId,Process<BoundedStringName>)>  {
            Provenance::get_processes_for_attestor_by_status(account_id,status)
        }
        fn get_processes_for_attestor_pending(account_id: AccountId) -> Vec<(RegistryId,DefinitionId,ProcessId,Process<BoundedStringName>)>  {
            Provenance::get_processes_for_attestor_pending(account_id)
        }
        fn get_process_steps(registry_id:RegistryId,definition_id:DefinitionId,process_id:ProcessId) -> Vec<(DefinitionStepIndex,ProcessStep<ProposalId,Moment,BoundedStringName,BoundedStringFact>)>  {
            Provenance::get_process_steps(registry_id,definition_id,process_id)
        }
        fn get_process_step(registry_id:RegistryId,definition_id:DefinitionId,process_id:ProcessId,definition_step_index:DefinitionStepIndex) -> Option<(DefinitionStepIndex,ProcessStep<ProposalId,Moment,BoundedStringName,BoundedStringFact>) >  {
            Provenance::get_process_step(registry_id,definition_id,process_id,definition_step_index)
        }
        fn get_definition_children(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(DefinitionId,Definition<BoundedStringName>)>  {
            Provenance::get_definition_children(registry_id,definition_id)
        }
        fn get_definition_parents(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(DefinitionId,Definition<BoundedStringName>)>  {
            Provenance::get_definition_parents(registry_id,definition_id)
        }
        fn can_view_definition(account_id: AccountId,registry_id:RegistryId,definition_id:DefinitionId) -> bool  {
            Provenance::can_view_definition(account_id,registry_id,definition_id)
        }
        fn is_attestor(account_id: AccountId,registry_id:RegistryId,definition_id:DefinitionId,definition_step_index: DefinitionStepIndex ) -> bool  {
            Provenance::is_attestor(account_id,registry_id,definition_id,definition_step_index)
        }
    }
    impl identity_runtime_api::IdentityApi<Block,AccountId,CatalogId,ClaimId,MemberCount,Moment,BoundedStringName,BoundedStringFact> for Runtime {
        fn is_catalog_owner(account_id: AccountId, catalog_id: CatalogId) -> bool {
            Identity::is_catalog_owner(account_id,catalog_id)
        }
        fn get_catalogs(account_id: AccountId) -> Vec<CatalogId> {
            Identity::get_catalogs(account_id)
        }
        fn get_dids_in_catalog(catalog_id: CatalogId) -> Vec<Did> {
            Identity::get_dids_in_catalog(catalog_id)
        }
        fn get_catalogs_by_did(did:Did) -> Vec<CatalogId> {
            Identity::get_catalogs_by_did(did)
        }
        fn get_did_in_catalog(catalog_id: CatalogId, did: Did) ->  Option<( DidDocument<AccountId>,Vec<DidProperty<BoundedStringName,BoundedStringFact>>,Vec<AccountId>)> {
            Identity::get_did_in_catalog(catalog_id, did)
        }
        fn is_controller(account_id: AccountId,did:Did) -> bool {
            Identity::is_controller(account_id, did)
        }
        fn get_did(did:Did) -> Option<(DidDocument<AccountId>,Vec<DidProperty<BoundedStringName,BoundedStringFact>>,Vec<AccountId>)>  {
            Identity::get_did(did)
        }
        fn get_dids_by_subject(subject: AccountId) -> Vec<Did> {
            Identity::get_dids_by_subject(subject)
        }
        fn get_dids_by_controller(controller: AccountId) -> Vec<Did> {
            Identity::get_dids_by_controller(controller)
        }
        fn find_did_by_text_or_did_property(catalog_id: CatalogId, name: Vec<u8>,filter: Vec<u8>) -> Vec<Did>  {
            Identity::find_did_by_text_or_did_property(catalog_id,name,filter)
        }
        fn find_did_by_integer_property(catalog_id: CatalogId, name: Vec<u8>,min: Option<u128>,max: Option<u128>) -> Vec<Did>  {
            Identity::find_did_by_integer_property(catalog_id,name,min,max)
        }
        fn find_did_by_float_property(catalog_id: CatalogId, name: Vec<u8>,min: Option<[u8;8]>,max: Option<[u8;8]>) -> Vec<Did>  {
            Identity::find_did_by_float_property(catalog_id,name,min,max)
        }
        fn find_did_by_date_property(catalog_id: CatalogId, name: Vec<u8>,min: Option<(u16, u8, u8)>,max: Option<(u16, u8, u8)>) -> Vec<Did>  {
            Identity::find_did_by_date_property(catalog_id,name,min,max)
        }
        // fn find_did_by_iso8601_property(catalog_id: CatalogId, name: Vec<u8>,min: Option<(u16, u8, u8, u8, u8, u8, Vec<u8>)>,max: Option<(u16, u8, u8, u8, u8, u8, Vec<u8>)>) -> Vec<Did>  {
        //     Identity::find_did_by_iso8601_property(catalog_id,name,min,max)
        // }
        fn get_claims(did: Did) -> Vec<(ClaimId, Claim<AccountId,MemberCount,Moment,BoundedStringName,BoundedStringFact>)>  {
            Identity::get_claims(did)
        }
        fn get_claim(did: Did, claim_id:ClaimId) -> Option<Claim<AccountId,MemberCount,Moment,BoundedStringName, BoundedStringFact>>{
            Identity::get_claim(did,claim_id)
        }
        fn get_claim_consumers(did: Did) -> Vec<(AccountId,Moment)>{Identity::get_claim_consumers(did)}
        fn get_claim_issuers(did: Did) -> Vec<(AccountId,Moment)>{Identity::get_claim_issuers(did)}
        fn get_dids_by_consumer(consumer:AccountId) -> Vec<(Did,Moment)>{Identity::get_dids_by_consumer(consumer)}
        fn get_dids_by_issuer(issuer:AccountId) -> Vec<(Did,Moment)>{Identity::get_dids_by_issuer(issuer)}
        fn get_outstanding_claims(consumer:AccountId) -> Vec<(Did,Moment)>{Identity::get_outstanding_claims(consumer)}
        fn get_outstanding_attestations(issuer:AccountId) -> Vec<(Did,Moment)>{Identity::get_outstanding_attestations(issuer)}
    }

    impl audits_runtime_api::AuditsApi<Block,AccountId,ProposalId,AuditId,ControlPointId,EvidenceId,ObservationId,BoundedStringName,BoundedStringUrl> for Runtime {
        fn get_audits_by_creator(account_id: AccountId) -> Vec<(AuditId,Audit<AccountId,ProposalId>)>{
            Audits::get_audits_by_creator(account_id)
        }
        fn get_audits_by_auditing_org(account_id: AccountId) -> Vec<(AuditId,Audit<AccountId,ProposalId>)>{
            Audits::get_audits_by_auditing_org(account_id)
        }
        fn get_audits_by_auditors(account_id: AccountId) -> Vec<(AuditId,Audit<AccountId,ProposalId>)>{
            Audits::get_audits_by_auditors(account_id)
        }
        fn get_linked_audits(audit_id:AuditId) -> Vec<(AuditId,Audit<AccountId,ProposalId>)>{
            Audits::get_linked_audits(audit_id)
        }
        fn get_audit(audit_id:AuditId) -> Option<Audit<AccountId,ProposalId>>{
            Audits::get_audit(audit_id)
        }
        fn get_audit_by_proposal(proposal_id:ProposalId) -> Option<(AuditId,Audit<AccountId,ProposalId>)>{
            Audits::get_audit_by_proposal(proposal_id)
        }
        fn get_observation(audit_id:AuditId,control_point_id:ControlPointId,observation_id:ObservationId)->Option<(Observation<ProposalId>,Vec<(EvidenceId,Evidence<ProposalId,BoundedStringName,BoundedStringUrl>)>)>{
            Audits::get_observation(audit_id,control_point_id,observation_id)
        }
        fn get_observation_by_proposal(proposal_id: ProposalId)->Option<(ObservationId,Observation<ProposalId>,Vec<(EvidenceId,Evidence<ProposalId,BoundedStringName,BoundedStringUrl>)>)>{
            Audits::get_observation_by_proposal(proposal_id)
        }
        fn get_observation_by_control_point(audit_id:AuditId,control_point_id:ControlPointId)->Vec<(ObservationId,Observation<ProposalId>,Vec<(EvidenceId,Evidence<ProposalId,BoundedStringName,BoundedStringUrl>)>)>{
            Audits::get_observation_by_control_point(audit_id,control_point_id)
        }
        fn get_evidence(audit_id:AuditId,evidence_id:EvidenceId)->Option<Evidence<ProposalId,BoundedStringName,BoundedStringUrl>>{
            Audits::get_evidence(audit_id,evidence_id)
        }
        fn get_evidence_by_audit(audit_id:AuditId)->Vec<(EvidenceId,Evidence<ProposalId,BoundedStringName,BoundedStringUrl>)>{
            Audits::get_evidence_by_audit(audit_id)
        }
        fn get_evidence_by_proposal(proposal_id:ProposalId)->Option<(EvidenceId,Evidence<ProposalId,BoundedStringName,BoundedStringUrl>)>{
            Audits::get_evidence_by_proposal(proposal_id)
        }
        fn get_evidence_links_by_evidence(evidence_id:EvidenceId)->Vec<ObservationId>{
            Audits::get_evidence_links_by_evidence(evidence_id)
        }
        fn get_evidence_links_by_observation(observation_id:ObservationId)->Vec<EvidenceId>{
            Audits::get_evidence_links_by_observation(observation_id)
        }
    }

    impl settings_runtime_api::SettingsApi<Block,ModuleIndex,ExtrinsicIndex,Balance> for Runtime {
        fn get_weight_to_fee_coefficients() -> Vec<(u64, Perbill, bool, u8)>{
            Settings::get_weight_to_fee_coefficients()
        }
        fn get_transaction_byte_fee() -> Balance{
            Settings::get_transaction_byte_fee()
        }
        fn get_fee_split_ratio() -> u32 {
            Settings::get_fee_split_ratio()
        }
        fn get_extrinsic_extra(module_index:ModuleIndex,extrinsic_index:ExtrinsicIndex) ->   Option<Balance>{
            Settings::get_extrinsic_extra(module_index,extrinsic_index)
        }
        fn get_extrinsic_extras() ->  Vec<(ModuleIndex,Vec<(ExtrinsicIndex,Balance)>)> {
            Settings::get_extrinsic_extras()
        }
    }
    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
        for Runtime
    {
        fn query_call_info(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_call_info(call, len)
        }
        fn query_call_fee_details(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_call_fee_details(call, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }


    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            let mut list = Vec::<BenchmarkList>::new();

            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();

            (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, TrackedStorageKey};

            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            impl frame_system_benchmarking::Config for Runtime {}
            impl baseline::Config for Runtime {}

            use frame_support::traits::WhitelistedStorageKeys;

            let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmarks!(params, batches);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}
