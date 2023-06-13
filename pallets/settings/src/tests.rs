//! Tests for the module.
use super::*;
use crate::mock::*;
use frame_support::assert_ok;
use sp_runtime::Perbill;

#[test]
fn set_weight_to_fee_coefficients_should_work() {
    new_test_ext().execute_with(|| {
        let mut new_coefficents = vec![];
        for i in 0..5u8 {
            new_coefficents.push((10u32.into(), Perbill::from_percent(50), false, i));
        }

        assert_ok!(Settings::set_weight_to_fee_coefficients(
            RuntimeOrigin::root(),
            new_coefficents.clone()
        ));

        let stored_coefficents = WeightToFeePolinomialCoefficients::<Test>::get();
        assert_eq!(stored_coefficents.len(), new_coefficents.len());
    });
}

#[test]
fn set_transaction_byte_fee_should_work() {
    new_test_ext().execute_with(|| {
        let new_fee = 10;

        assert_ok!(Settings::set_transaction_byte_fee(
            RuntimeOrigin::root(),
            new_fee
        ));

        let stored_fee = TransactionByteFee::<Test>::get();
        assert_eq!(stored_fee, new_fee);
    });
}

#[test]
fn set_fee_split_ratio_should_work() {
    new_test_ext().execute_with(|| {
        let new_ratio = 10u32;

        assert_ok!(Settings::set_fee_split_ratio(
            RuntimeOrigin::root(),
            new_ratio
        ));

        let stored_ratio = FeeSplitRatio::<Test>::get();
        assert_eq!(stored_ratio, new_ratio);
    });
}

#[test]
fn set_extrinsic_extra_should_work() {
    new_test_ext().execute_with(|| {
        let new_extra = 10;
        let module_index = 1u8;
        let extrinsic_index = 1u8;

        assert_ok!(Settings::set_extrinsic_extra(
            RuntimeOrigin::root(),
            module_index,
            extrinsic_index,
            new_extra
        ));

        let stored_extra = ExtrinsicExtra::<Test>::get(module_index, extrinsic_index);
        assert!(stored_extra.is_some());
        let stored_extra = stored_extra.unwrap();
        assert_eq!(stored_extra, new_extra);
    });
}
#[test]
fn remove_extrinsic_extra_should_work() {
    new_test_ext().execute_with(|| {
        let module_index = 1u8;
        let extrinsic_index = 1u8;

        assert_ok!(Settings::remove_extrinsic_extra(
            RuntimeOrigin::root(),
            module_index,
            extrinsic_index,
        ));

        assert!(!ExtrinsicExtra::<Test>::contains_key(
            module_index,
            extrinsic_index
        ));
    });
}
