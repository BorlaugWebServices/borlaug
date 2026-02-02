// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use crate as pallet_transaction_payment;
use pallet_balances::Call as BalancesCall;
use polkadot_sdk::frame_support::traits::EnsureOrigin;
use polkadot_sdk::frame_support::{
    derive_impl,
    dispatch::DispatchClass,
    parameter_types,
    traits::{fungible, Imbalance, OnUnbalanced},
    weights::{Weight, WeightToFee as WeightToFeeT},
};
use polkadot_sdk::frame_system as system;

use polkadot_sdk::sp_runtime::Either; // Ensure this is imported

pub struct EnsureRootToGroupsThree;
impl EnsureOrigin<RuntimeOrigin> for EnsureRootToGroupsThree {
    type Success = (u32, u64, u64);

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        // Explicitly type the result of the into() conversion
        let result: Result<system::RawOrigin<u64>, RuntimeOrigin> = o.into();
        result.and_then(|o| match o {
            system::RawOrigin::Root => Ok((0, 0, 0)),
            r => Err(RuntimeOrigin::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Ok(RuntimeOrigin::root())
    }
}

pub struct OriginFive;
impl EnsureOrigin<RuntimeOrigin> for OriginFive {
    type Success = (u32, u32, Option<u32>, Option<u32>, u64);

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        let result: Result<system::RawOrigin<u64>, RuntimeOrigin> = o.into();
        result.and_then(|o| match o {
            system::RawOrigin::Root => Ok((0, 0, None, None, 0)),
            r => Err(RuntimeOrigin::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Ok(RuntimeOrigin::root())
    }
}

pub struct OriginFiveEither;
impl EnsureOrigin<RuntimeOrigin> for OriginFiveEither {
    type Success = Either<u64, (u32, u32, Option<u32>, Option<u32>, u64)>;

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        let result: Result<system::RawOrigin<u64>, RuntimeOrigin> = o.into();
        result.and_then(|o| match o {
            system::RawOrigin::Root => Ok(Either::Left(0)),
            r => Err(RuntimeOrigin::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Ok(RuntimeOrigin::root())
    }
}

pub struct OriginThreeEither;
impl EnsureOrigin<RuntimeOrigin> for OriginThreeEither {
    type Success = Either<u64, (u32, u64, u64)>;

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        let result: Result<system::RawOrigin<u64>, RuntimeOrigin> = o.into();
        result.and_then(|o| match o {
            system::RawOrigin::Root => Ok(Either::Left(0)),
            r => Err(RuntimeOrigin::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Ok(RuntimeOrigin::root())
    }
}

type Balance = u64;

impl settings::Config for Runtime {
    type ChangeSettingOrigin = polkadot_sdk::frame_system::EnsureRoot<Self::AccountId>;
    type ModuleIndex = u8;
    type ExtrinsicIndex = u8;
    type Currency = Balances;
    type Balance = Balance;
    type WeightInfo = ();
}
type Block = polkadot_sdk::frame_system::mocking::MockBlock<Runtime>;

impl groups::Config for Runtime {
    type GroupId = u32;
    type ProposalId = u32;
    type MemberCount = u32;
    type Currency = Balances;
    type Origin = RuntimeOrigin;
    type GroupsOriginByGroupThreshold = OriginFive;
    type GroupsOriginByCallerThreshold = OriginFive;
    type GroupsOriginExecuted = EnsureRootToGroupsThree;
    type GroupsOriginAccountOrThreshold = OriginFiveEither;
    type GroupsOriginAccountOrApproved = OriginFiveEither;
    type GroupsOriginAccountOrExecuted = OriginThreeEither;
    type Proposal = RuntimeCall;
    type MaxProposals = ConstU32<100>;
    type MaxMembers = ConstU32<100>;
    type MaxProposalLength = ();
    type WeightInfo = ();
    type GetExtrinsicExtraSource = settings::Pallet<Runtime>;
    type NameLimit = ();
    type GroupChainLimit = ();
}

polkadot_sdk::frame_support::construct_runtime!(
    pub struct Runtime
    {
        System: system,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>},
        Groups: groups,
        Settings: settings,
    }
);

pub(crate) const CALL: &<Runtime as frame_system::Config>::RuntimeCall =
    &RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 69 });

parameter_types! {
    pub(crate) static ExtrinsicBaseWeight: Weight = Weight::zero();
}

pub struct BlockWeights;
impl Get<frame_system::limits::BlockWeights> for BlockWeights {
    fn get() -> frame_system::limits::BlockWeights {
        frame_system::limits::BlockWeights::builder()
            .base_block(Weight::zero())
            .for_class(DispatchClass::all(), |weights| {
                weights.base_extrinsic = ExtrinsicBaseWeight::get().into();
            })
            .for_class(DispatchClass::non_mandatory(), |weights| {
                weights.max_total = Weight::from_parts(1024, u64::MAX).into();
            })
            .build_or_panic()
    }
}

parameter_types! {
    pub static WeightToFee: u64 = 1;
    pub static TransactionByteFee: u64 = 1;
    pub static OperationalFeeMultiplier: u8 = 5;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Runtime {
    type BlockWeights = BlockWeights;
    type Block = Block;
    type AccountData = pallet_balances::AccountData<Self::AccountId>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Runtime {
    type AccountStore = System;
}

impl WeightToFeeT for WeightToFee {
    type Balance = u64;

    fn weight_to_fee(weight: &Weight) -> Self::Balance {
        Self::Balance::saturated_from(weight.ref_time())
            .saturating_mul(WEIGHT_TO_FEE.with(|v| *v.borrow()))
    }
}

impl WeightToFeeT for TransactionByteFee {
    type Balance = u64;

    fn weight_to_fee(weight: &Weight) -> Self::Balance {
        Self::Balance::saturated_from(weight.ref_time())
            .saturating_mul(TRANSACTION_BYTE_FEE.with(|v| *v.borrow()))
    }
}

parameter_types! {
    pub(crate) static TipUnbalancedAmount: u64 = 0;
    pub(crate) static FeeUnbalancedAmount: u64 = 0;
}

pub struct DealWithFees;
impl OnUnbalanced<fungible::Credit<<Runtime as frame_system::Config>::AccountId, Balances>>
    for DealWithFees
{
    fn on_unbalanceds(
        mut fees_then_tips: impl Iterator<
            Item = fungible::Credit<<Runtime as frame_system::Config>::AccountId, Balances>,
        >,
    ) {
        if let Some(fees) = fees_then_tips.next() {
            FeeUnbalancedAmount::mutate(|a| *a += fees.peek());
            if let Some(tips) = fees_then_tips.next() {
                TipUnbalancedAmount::mutate(|a| *a += tips.peek());
            }
        }
    }
}

/// Weights used in testing.
pub struct MockWeights;

impl WeightInfo for MockWeights {
    fn charge_transaction_payment() -> Weight {
        Weight::from_parts(10, 0)
    }
}

impl Config for Runtime {
    type OnChargeTransaction = FungibleAdapter<Balances, DealWithFees>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type WeightToFee = WeightToFee;
    type LengthToFee = TransactionByteFee;
    type FeeMultiplierUpdate = ();
    type WeightInfo = MockWeights;
}

#[cfg(feature = "runtime-benchmarks")]
pub fn new_test_ext() -> sp_io::TestExternalities {
    crate::tests::ExtBuilder::default()
        .base_weight(Weight::from_parts(100, 0))
        .byte_fee(10)
        .balance_factor(0)
        .build()
}
