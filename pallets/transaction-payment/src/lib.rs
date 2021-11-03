// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
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

//! # Transaction Payment Pallet
//!
//! This pallet provides the basic logic needed to pay the absolute minimum amount needed for a
//! transaction to be included. This includes:
//!   - _base fee_: This is the minimum amount a user pays for a transaction. It is declared
//!     as a base _weight_ in the runtime and converted to a fee using `WeightToFee`.
//!   - _weight fee_: A fee proportional to amount of weight a transaction consumes.
//!   - _length fee_: A fee proportional to the encoded length of the transaction.
//!   - _tip_: An optional tip. Tip increases the priority of the transaction, giving it a higher
//!     chance to be included by the transaction queue.
//!
//! The base fee and adjusted weight and length fees constitute the _inclusion fee_, which is
//! the minimum fee for a transaction to be included in a block.
//!
//! The formula of final fee:
//!   ```ignore
//!   inclusion_fee = base_fee + length_fee + [targeted_fee_adjustment * weight_fee];
//!   final_fee = inclusion_fee + tip;
//!   ```
//!
//!   - `targeted_fee_adjustment`: This is a multiplier that can tune the final fee based on
//!     the congestion of the network.
//!
//! Additionally, this pallet allows one to configure:
//!   - The mapping between one unit of weight to one unit of fee via [`Config::WeightToFee`].
//!   - A means of updating the fee for the next block, via defining a multiplier, based on the
//!     final state of the chain at the end of the previous block. This can be configured via
//!     [`Config::FeeMultiplierUpdate`]
//!   - How the fees are paid via [`Config::OnChargeTransaction`].

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

use sp_runtime::{
    traits::{
        Convert, DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SaturatedConversion, Saturating,
        SignedExtension,
    },
    transaction_validity::{
        TransactionPriority, TransactionValidity, TransactionValidityError, ValidTransaction,
    },
    FixedPointNumber, FixedPointOperand, FixedU128, Perquintill, RuntimeDebug,
};
use sp_std::prelude::*;

use frame_support::{
    dispatch::DispatchResult,
    traits::{Get, IsSubType},
    weights::{
        DispatchClass, DispatchInfo, GetDispatchInfo, Pays, PostDispatchInfo, Weight,
        WeightToFeeCoefficient, WeightToFeePolynomial,
    },
};

mod payment;
mod types;

pub use pallet::*;
pub use payment::*;
pub use types::{FeeDetails, InclusionFee, RuntimeDispatchInfo};

/// Fee multiplier.
pub type Multiplier = FixedU128;

type BalanceOf<T> = <<T as Config>::OnChargeTransaction as OnChargeTransaction<T>>::Balance;

/// A struct to update the weight multiplier per block. It implements `Convert<Multiplier,
/// Multiplier>`, meaning that it can convert the previous multiplier to the next one. This should
/// be called on `on_finalize` of a block, prior to potentially cleaning the weight data from the
/// system pallet.
///
/// given:
///    s = previous block weight
///    s'= ideal block weight
///    m = maximum block weight
///    diff = (s - s')/m
///    v = 0.00001
///    t1 = (v * diff)
///    t2 = (v * diff)^2 / 2
/// then:
///    next_multiplier = prev_multiplier * (1 + t1 + t2)
///
/// Where `(s', v)` must be given as the `Get` implementation of the `T` generic type. Moreover, `M`
/// must provide the minimum allowed value for the multiplier. Note that a runtime should ensure
/// with tests that the combination of this `M` and `V` is not such that the multiplier can drop to
/// zero and never recover.
///
/// note that `s'` is interpreted as a portion in the _normal transaction_ capacity of the block.
/// For example, given `s' == 0.25` and `AvailableBlockRatio = 0.75`, then the target fullness is
/// _0.25 of the normal capacity_ and _0.1875 of the entire block_.
///
/// This implementation implies the bound:
/// - `v ≤ p / k * (s − s')`
/// - or, solving for `p`: `p >= v * k * (s - s')`
///
/// where `p` is the amount of change over `k` blocks.
///
/// Hence:
/// - in a fully congested chain: `p >= v * k * (1 - s')`.
/// - in an empty chain: `p >= v * k * (-s')`.
///
/// For example, when all blocks are full and there are 28800 blocks per day (default in `substrate-node`)
/// and v == 0.00001, s' == 0.1875, we'd have:
///
/// p >= 0.00001 * 28800 * 0.8125
/// p >= 0.234
///
/// Meaning that fees can change by around ~23% per day, given extreme congestion.
///
/// More info can be found at:
/// <https://w3f-research.readthedocs.io/en/latest/polkadot/overview/2-token-economics.html>
pub struct TargetedFeeAdjustment<T, S, V, M>(sp_std::marker::PhantomData<(T, S, V, M)>);

/// Something that can convert the current multiplier to the next one.
pub trait MultiplierUpdate: Convert<Multiplier, Multiplier> {
    /// Minimum multiplier
    fn min() -> Multiplier;
    /// Target block saturation level
    fn target() -> Perquintill;
    /// Variability factor
    fn variability() -> Multiplier;
}

impl MultiplierUpdate for () {
    fn min() -> Multiplier {
        Default::default()
    }
    fn target() -> Perquintill {
        Default::default()
    }
    fn variability() -> Multiplier {
        Default::default()
    }
}

impl<T, S, V, M> MultiplierUpdate for TargetedFeeAdjustment<T, S, V, M>
where
    T: frame_system::Config,
    S: Get<Perquintill>,
    V: Get<Multiplier>,
    M: Get<Multiplier>,
{
    fn min() -> Multiplier {
        M::get()
    }
    fn target() -> Perquintill {
        S::get()
    }
    fn variability() -> Multiplier {
        V::get()
    }
}

impl<T, S, V, M> Convert<Multiplier, Multiplier> for TargetedFeeAdjustment<T, S, V, M>
where
    T: frame_system::Config,
    S: Get<Perquintill>,
    V: Get<Multiplier>,
    M: Get<Multiplier>,
{
    fn convert(previous: Multiplier) -> Multiplier {
        // Defensive only. The multiplier in storage should always be at most positive. Nonetheless
        // we recover here in case of errors, because any value below this would be stale and can
        // never change.
        let min_multiplier = M::get();
        let previous = previous.max(min_multiplier);

        let weights = T::BlockWeights::get();
        // the computed ratio is only among the normal class.
        let normal_max_weight = weights
            .get(DispatchClass::Normal)
            .max_total
            .unwrap_or(weights.max_block);
        let current_block_weight = <frame_system::Pallet<T>>::block_weight();
        let normal_block_weight = *current_block_weight
            .get(DispatchClass::Normal)
            .min(&normal_max_weight);

        let s = S::get();
        let v = V::get();

        let target_weight = (s * normal_max_weight) as u128;
        let block_weight = normal_block_weight as u128;

        // determines if the first_term is positive
        let positive = block_weight >= target_weight;
        let diff_abs = block_weight.max(target_weight) - block_weight.min(target_weight);

        // defensive only, a test case assures that the maximum weight diff can fit in Multiplier
        // without any saturation.
        let diff = Multiplier::saturating_from_rational(diff_abs, normal_max_weight.max(1));
        let diff_squared = diff.saturating_mul(diff);

        let v_squared_2 = v.saturating_mul(v) / Multiplier::saturating_from_integer(2);

        let first_term = v.saturating_mul(diff);
        let second_term = v_squared_2.saturating_mul(diff_squared);

        if positive {
            let excess = first_term
                .saturating_add(second_term)
                .saturating_mul(previous);
            previous.saturating_add(excess).max(min_multiplier)
        } else {
            // Defensive-only: first_term > second_term. Safe subtraction.
            let negative = first_term
                .saturating_sub(second_term)
                .saturating_mul(previous);
            previous.saturating_sub(negative).max(min_multiplier)
        }
    }
}

/// Storage releases of the pallet.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
enum Releases {
    /// Original version of the pallet.
    V1Ancient,
    /// One that bumps the usage to FixedU128 from FixedI128.
    V2,
}

impl Default for Releases {
    fn default() -> Self {
        Releases::V1Ancient
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + groups::Config {
        /// Handler for withdrawing, refunding and depositing the transaction fee.
        /// Transaction fees are withdrawn before the transaction is executed.
        /// After the transaction was executed the transaction weight can be
        /// adjusted, depending on the used resources by the transaction. If the
        /// transaction weight is lower than expected, parts of the transaction fee
        /// might be refunded. In the end the fees can be deposited.
        type OnChargeTransaction: OnChargeTransaction<Self>;

        /// The fee to be paid for making a transaction; the per-byte portion.
        // #[pallet::constant]
        type TransactionByteFee: Get<BalanceOf<Self>>;

        /// Convert a weight value into a deductible fee based on the currency type.
        type WeightToFee: WeightToFeePolynomial<Balance = BalanceOf<Self>>;

        /// Update the multiplier of the next block, based on the previous block's weight.
        type FeeMultiplierUpdate: MultiplierUpdate;
    }

    #[pallet::extra_constants]
    impl<T: Config> Pallet<T> {
        //TODO: rename to snake case after https://github.com/paritytech/substrate/issues/8826 fixed.
        #[allow(non_snake_case)]
        /// The polynomial that is applied in order to derive fee from weight.
        fn WeightToFee() -> Vec<WeightToFeeCoefficient<BalanceOf<T>>> {
            T::WeightToFee::polynomial().to_vec()
        }
    }

    #[pallet::type_value]
    pub fn NextFeeMultiplierOnEmpty() -> Multiplier {
        Multiplier::saturating_from_integer(1)
    }

    #[pallet::storage]
    #[pallet::getter(fn next_fee_multiplier)]
    pub type NextFeeMultiplier<T: Config> =
        StorageValue<_, Multiplier, ValueQuery, NextFeeMultiplierOnEmpty>;

    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig;

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            StorageVersion::<T>::put(Releases::V2);
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_: T::BlockNumber) {
            <NextFeeMultiplier<T>>::mutate(|fm| {
                *fm = T::FeeMultiplierUpdate::convert(*fm);
            });
        }

        fn integrity_test() {
            // given weight == u64, we build multipliers from `diff` of two weight values, which can
            // at most be maximum block weight. Make sure that this can fit in a multiplier without
            // loss.
            use sp_std::convert::TryInto;
            assert!(
                <Multiplier as sp_runtime::traits::Bounded>::max_value()
                    >= Multiplier::checked_from_integer(
                        T::BlockWeights::get().max_block.try_into().unwrap()
                    )
                    .unwrap(),
            );

            // This is the minimum value of the multiplier. Make sure that if we collapse to this
            // value, we can recover with a reasonable amount of traffic. For this test we assert
            // that if we collapse to minimum, the trend will be positive with a weight value
            // which is 1% more than the target.
            let min_value = T::FeeMultiplierUpdate::min();
            let mut target = T::FeeMultiplierUpdate::target()
                * T::BlockWeights::get()
                    .get(DispatchClass::Normal)
                    .max_total
                    .expect(
                        "Setting `max_total` for `Normal` dispatch class is not compatible with \
					`transaction-payment` pallet.",
                    );
            // add 1 percent;
            let addition = target / 100;
            if addition == 0 {
                // this is most likely because in a test setup we set everything to ().
                return;
            }
            target += addition;

            #[cfg(any(feature = "std", test))]
            sp_io::TestExternalities::new_empty().execute_with(|| {
                <frame_system::Pallet<T>>::set_block_consumed_resources(target, 0);
                let next = T::FeeMultiplierUpdate::convert(min_value);
                assert!(
                    next > min_value,
                    "The minimum bound of the multiplier is too low. When \
					block saturation is more than target by 1% and multiplier is minimal then \
					the multiplier doesn't increase."
                );
            });
        }
    }
    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T>
where
    BalanceOf<T>: FixedPointOperand,
{
    /// Query the data that we know about the fee of a given `call`.
    ///
    /// This pallet is not and cannot be aware of the internals of a signed extension, for example
    /// a tip. It only interprets the extrinsic as some encoded value and accounts for its weight
    /// and length, the runtime's extrinsic base weight, and the current fee multiplier.
    ///
    /// All dispatchables must be annotated with weight and will have some fee info. This function
    /// always returns.
    pub fn query_info<Extrinsic: GetDispatchInfo>(
        unchecked_extrinsic: Extrinsic,
        len: u32,
    ) -> RuntimeDispatchInfo<BalanceOf<T>>
    where
        T::Call: Dispatchable<Info = DispatchInfo>,
    {
        // NOTE: we can actually make it understand `ChargeTransactionPayment`, but would be some
        // hassle for sure. We have to make it aware of the index of `ChargeTransactionPayment` in
        // `Extra`. Alternatively, we could actually execute the tx's per-dispatch and record the
        // balance of the sender before and after the pipeline.. but this is way too much hassle for
        // a very very little potential gain in the future.
        let dispatch_info = <Extrinsic as GetDispatchInfo>::get_dispatch_info(&unchecked_extrinsic);

        let partial_fee = Self::compute_fee(len, &dispatch_info, 0u32.into());
        let DispatchInfo { weight, class, .. } = dispatch_info;

        RuntimeDispatchInfo {
            weight,
            class,
            partial_fee,
        }
    }

    /// Query the detailed fee of a given `call`.
    pub fn query_fee_details<Extrinsic: GetDispatchInfo>(
        unchecked_extrinsic: Extrinsic,
        len: u32,
    ) -> FeeDetails<BalanceOf<T>>
    where
        T::Call: Dispatchable<Info = DispatchInfo>,
    {
        let dispatch_info = <Extrinsic as GetDispatchInfo>::get_dispatch_info(&unchecked_extrinsic);
        Self::compute_fee_details(len, &dispatch_info, 0u32.into())
    }

    /// Compute the final fee value for a particular transaction.
    pub fn compute_fee(len: u32, info: &DispatchInfoOf<T::Call>, tip: BalanceOf<T>) -> BalanceOf<T>
    where
        T::Call: Dispatchable<Info = DispatchInfo>,
    {
        Self::compute_fee_details(len, info, tip).final_fee()
    }

    /// Compute the fee details for a particular transaction.
    pub fn compute_fee_details(
        len: u32,
        info: &DispatchInfoOf<T::Call>,
        tip: BalanceOf<T>,
    ) -> FeeDetails<BalanceOf<T>>
    where
        T::Call: Dispatchable<Info = DispatchInfo>,
    {
        Self::compute_fee_raw(len, info.weight, tip, info.pays_fee, info.class)
    }

    /// Compute the actual post dispatch fee for a particular transaction.
    ///
    /// Identical to `compute_fee` with the only difference that the post dispatch corrected
    /// weight is used for the weight fee calculation.
    pub fn compute_actual_fee(
        len: u32,
        info: &DispatchInfoOf<T::Call>,
        post_info: &PostDispatchInfoOf<T::Call>,
        tip: BalanceOf<T>,
    ) -> BalanceOf<T>
    where
        T::Call: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    {
        Self::compute_actual_fee_details(len, info, post_info, tip).final_fee()
    }

    /// Compute the actual post dispatch fee details for a particular transaction.
    pub fn compute_actual_fee_details(
        len: u32,
        info: &DispatchInfoOf<T::Call>,
        post_info: &PostDispatchInfoOf<T::Call>,
        tip: BalanceOf<T>,
    ) -> FeeDetails<BalanceOf<T>>
    where
        T::Call: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    {
        Self::compute_fee_raw(
            len,
            post_info.calc_actual_weight(info),
            tip,
            post_info.pays_fee(info),
            info.class,
        )
    }

    fn compute_fee_raw(
        len: u32,
        weight: Weight,
        tip: BalanceOf<T>,
        pays_fee: Pays,
        class: DispatchClass,
    ) -> FeeDetails<BalanceOf<T>> {
        if pays_fee == Pays::Yes {
            let len = <BalanceOf<T>>::from(len);
            let per_byte = T::TransactionByteFee::get();

            // length fee. this is not adjusted.
            let fixed_len_fee = per_byte.saturating_mul(len);

            // the adjustable part of the fee.
            let unadjusted_weight_fee = Self::weight_to_fee(weight);
            let multiplier = Self::next_fee_multiplier();
            // final adjusted weight fee.
            let adjusted_weight_fee = multiplier.saturating_mul_int(unadjusted_weight_fee);

            let base_fee = Self::weight_to_fee(T::BlockWeights::get().get(class).base_extrinsic);
            FeeDetails {
                inclusion_fee: Some(InclusionFee {
                    base_fee,
                    len_fee: fixed_len_fee,
                    adjusted_weight_fee,
                }),
                tip,
            }
        } else {
            FeeDetails {
                inclusion_fee: None,
                tip,
            }
        }
    }

    fn weight_to_fee(weight: Weight) -> BalanceOf<T> {
        // cap the weight to the maximum defined in runtime, otherwise it will be the
        // `Bounded` maximum of its data type, which is not desired.
        let capped_weight = weight.min(T::BlockWeights::get().max_block);
        T::WeightToFee::calc(&capped_weight)
    }
}

impl<T> Convert<Weight, BalanceOf<T>> for Pallet<T>
where
    T: Config,
    BalanceOf<T>: FixedPointOperand,
{
    /// Compute the fee for the specified weight.
    ///
    /// This fee is already adjusted by the per block fee adjustment factor and is therefore the
    /// share that the weight contributes to the overall fee of a transaction. It is mainly
    /// for informational purposes and not used in the actual fee calculation.
    fn convert(weight: Weight) -> BalanceOf<T> {
        <NextFeeMultiplier<T>>::get().saturating_mul_int(Self::weight_to_fee(weight))
    }
}

/// Require the transactor pay for themselves and maybe include a tip to gain additional priority
/// in the queue.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct ChargeTransactionPayment<T: Config + groups::Config>(#[codec(compact)] BalanceOf<T>)
where
    <T as frame_system::Config>::Call: IsSubType<groups::Call<T>>;

impl<T: Config + groups::Config> ChargeTransactionPayment<T>
where
    T::Call: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    BalanceOf<T>: Send + Sync + FixedPointOperand,
    <T as frame_system::Config>::Call: IsSubType<groups::Call<T>>,
{
    /// utility constructor. Used only in client/factory code.
    pub fn from(fee: BalanceOf<T>) -> Self {
        Self(fee)
    }

    /// Returns the tip as being choosen by the transaction sender.
    pub fn tip(&self) -> BalanceOf<T> {
        self.0
    }

    fn withdraw_fee(
        &self,
        who: &T::AccountId,
        call: &T::Call,
        info: &DispatchInfoOf<T::Call>,
        len: usize,
    ) -> Result<
        (
            BalanceOf<T>,
            <<T as Config>::OnChargeTransaction as OnChargeTransaction<T>>::LiquidityInfo,
        ),
        TransactionValidityError,
    > {
        let tip = self.0;
        let fee = Pallet::<T>::compute_fee(len as u32, info, tip);

        <<T as Config>::OnChargeTransaction as OnChargeTransaction<T>>::withdraw_fee(
            who, call, info, fee, tip,
        )
        .map(|i| (fee, i))
    }

    /// Get an appropriate priority for a transaction with the given length and info.
    ///
    /// This will try and optimise the `fee/weight` `fee/length`, whichever is consuming more of the
    /// maximum corresponding limit.
    ///
    /// For example, if a transaction consumed 1/4th of the block length and half of the weight, its
    /// final priority is `fee * min(2, 4) = fee * 2`. If it consumed `1/4th` of the block length
    /// and the entire block weight `(1/1)`, its priority is `fee * min(1, 4) = fee * 1`. This means
    ///  that the transaction which consumes more resources (either length or weight) with the same
    /// `fee` ends up having lower priority.
    fn get_priority(
        len: usize,
        info: &DispatchInfoOf<T::Call>,
        final_fee: BalanceOf<T>,
    ) -> TransactionPriority {
        let weight_saturation = T::BlockWeights::get().max_block / info.weight.max(1);
        let max_block_length = *T::BlockLength::get().max.get(DispatchClass::Normal);
        let len_saturation = max_block_length as u64 / (len as u64).max(1);
        let coefficient: BalanceOf<T> = weight_saturation
            .min(len_saturation)
            .saturated_into::<BalanceOf<T>>();
        final_fee
            .saturating_mul(coefficient)
            .saturated_into::<TransactionPriority>()
    }
}

impl<T: Config + groups::Config> sp_std::fmt::Debug for ChargeTransactionPayment<T>
where
    <T as frame_system::Config>::Call: IsSubType<groups::Call<T>>,
{
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "ChargeTransactionPayment<{:?}>", self.0)
    }
    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config + groups::Config> SignedExtension for ChargeTransactionPayment<T>
where
    BalanceOf<T>: Send + Sync + From<u64> + FixedPointOperand,
    T::Call: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    T::Call: IsSubType<groups::Call<T>>,
{
    const IDENTIFIER: &'static str = "ChargeTransactionPayment";
    type AccountId = T::AccountId;
    type Call = T::Call;
    type AdditionalSigned = ();
    type Pre = (
        // tip
        BalanceOf<T>,
        // who paid the fee
        Self::AccountId,
        // imbalance resulting from withdrawing the fee
        <<T as Config>::OnChargeTransaction as OnChargeTransaction<T>>::LiquidityInfo,
    );
    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> TransactionValidity {
        let who = match call.is_sub_type() {
            Some(groups::Call::propose(group_id, ..)) => {
                if let Some(group) = groups::Module::<T>::groups(group_id) {
                    group.anonymous_account
                } else {
                    who.clone()
                }
            }
            Some(groups::Call::execute(group_id, ..)) => {
                if let Some(group) = groups::Module::<T>::groups(group_id) {
                    group.anonymous_account
                } else {
                    who.clone()
                }
            }
            Some(groups::Call::close(group_id, ..)) => {
                if let Some(group) = groups::Module::<T>::groups(group_id) {
                    group.anonymous_account
                } else {
                    who.clone()
                }
            }
            Some(groups::Call::veto(group_id, ..)) => {
                if let Some(group) = groups::Module::<T>::groups(group_id) {
                    group.anonymous_account
                } else {
                    who.clone()
                }
            }
            _ => who.clone(),
        };

        let (fee, _) = self.withdraw_fee(&who, call, info, len)?;
        Ok(ValidTransaction {
            priority: Self::get_priority(len, info, fee),
            ..Default::default()
        })
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        let who = match call.is_sub_type() {
            Some(groups::Call::propose(group_id, ..)) => {
                if let Some(group) = groups::Module::<T>::groups(group_id) {
                    group.anonymous_account
                } else {
                    who.clone()
                }
            }
            Some(groups::Call::execute(group_id, ..)) => {
                if let Some(group) = groups::Module::<T>::groups(group_id) {
                    group.anonymous_account
                } else {
                    who.clone()
                }
            }
            Some(groups::Call::close(group_id, ..)) => {
                if let Some(group) = groups::Module::<T>::groups(group_id) {
                    group.anonymous_account
                } else {
                    who.clone()
                }
            }
            Some(groups::Call::veto(group_id, ..)) => {
                if let Some(group) = groups::Module::<T>::groups(group_id) {
                    group.anonymous_account
                } else {
                    who.clone()
                }
            }
            _ => who.clone(),
        };

        let (_fee, imbalance) = self.withdraw_fee(&who, call, info, len)?;
        Ok((self.0, who, imbalance))
    }

    fn post_dispatch(
        pre: Self::Pre,
        info: &DispatchInfoOf<Self::Call>,
        post_info: &PostDispatchInfoOf<Self::Call>,
        len: usize,
        _result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        let (tip, who, imbalance) = pre;
        let actual_fee = Pallet::<T>::compute_actual_fee(len as u32, info, post_info, tip);
        T::OnChargeTransaction::correct_and_deposit_fee(
            &who, info, post_info, actual_fee, tip, imbalance,
        )?;
        Ok(())
    }
}
