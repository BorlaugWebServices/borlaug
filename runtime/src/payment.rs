///! custom implementation for paying transaction fees.
use codec::FullCodec;
use frame_support::{
    debug,
    traits::{Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced, WithdrawReasons},
    unsigned::TransactionValidityError,
};
use pallet_transaction_payment::{Config, OnChargeTransaction};
use sp_runtime::print;
use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned, DispatchInfoOf, MaybeSerializeDeserialize, PostDispatchInfoOf,
        Saturating, UniqueSaturatedFrom, Zero,
    },
    transaction_validity::InvalidTransaction,
};
use sp_std::{fmt::Debug, marker::PhantomData};

type NegativeImbalanceOf<C, T> =
    <C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

/// Implements the transaction payment for a module implementing the `Currency`
/// trait (eg. the pallet_balances) using an unbalance handler (implementing
/// `OnUnbalanced`).
pub struct GroupsCurrencyAdapter<C, OU>(PhantomData<(C, OU)>);

/// Default implementation for a Currency and an OnUnbalanced handler.
impl<T, C, OU> OnChargeTransaction<T> for GroupsCurrencyAdapter<C, OU>
where
    T: Config,
    T::TransactionByteFee: Get<<C as Currency<<T as frame_system::Config>::AccountId>>::Balance>,
    C: Currency<<T as frame_system::Config>::AccountId>,
    C::PositiveImbalance: Imbalance<
        <C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
        Opposite = C::NegativeImbalance,
    >,
    C::NegativeImbalance: Imbalance<
        <C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
        Opposite = C::PositiveImbalance,
    >,
    OU: OnUnbalanced<NegativeImbalanceOf<C, T>>,
{
    type LiquidityInfo = Option<NegativeImbalanceOf<C, T>>;
    type Balance = <C as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Withdraw the predicted fee from the transaction origin.
    ///
    /// Note: The `fee` already includes the `tip`.
    fn withdraw_fee(
        who: &T::AccountId,
        _call: &T::Call,
        _info: &DispatchInfoOf<T::Call>,
        fee: Self::Balance,
        tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        if fee.is_zero() {
            return Ok(None);
        }

        // print("Here");
        // let fee = fee + Self::Balance::unique_saturated_from(2_000u32);

        let withdraw_reason = if tip.is_zero() {
            WithdrawReasons::TRANSACTION_PAYMENT
        } else {
            WithdrawReasons::TRANSACTION_PAYMENT | WithdrawReasons::TIP
        };

        match C::withdraw(who, fee, withdraw_reason, ExistenceRequirement::KeepAlive) {
            Ok(imbalance) => Ok(Some(imbalance)),
            Err(_) => Err(InvalidTransaction::Payment.into()),
        }
    }

    /// Hand the fee and the tip over to the `[OnUnbalanced]` implementation.
    /// Since the predicted fee might have been too high, parts of the fee may
    /// be refunded.
    ///
    /// Note: The `fee` already includes the `tip`.
    fn correct_and_deposit_fee(
        who: &T::AccountId,
        _dispatch_info: &DispatchInfoOf<T::Call>,
        _post_info: &PostDispatchInfoOf<T::Call>,
        corrected_fee: Self::Balance,
        tip: Self::Balance,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        if let Some(paid) = already_withdrawn {
            // Calculate how much refund we should return
            let refund_amount = paid.peek().saturating_sub(corrected_fee);

            // print("In correct_and_deposit_fee fn");

            debug::info!(
                "{:?} {:?} {:?} {:?}",
                corrected_fee,
                tip,
                paid.peek(),
                refund_amount
            );

            let refund_amount = refund_amount + Self::Balance::unique_saturated_from(2_000u128);
            // refund to the the account that paid the fees. If this fails, the
            // account might have dropped below the existential balance. In
            // that case we don't refund anything.
            let refund_imbalance = C::deposit_into_existing(&who, refund_amount)
                .unwrap_or_else(|_| C::PositiveImbalance::zero());
            // merge the imbalance caused by paying the fees and refunding parts of it again.
            let adjusted_paid = paid
                .offset(refund_imbalance)
                .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;
            // Call someone else to handle the imbalance (fee and tip separately)
            let imbalances = adjusted_paid.split(tip);
            OU::on_unbalanceds(Some(imbalances.0).into_iter().chain(Some(imbalances.1)));
        }
        Ok(())
    }
}
