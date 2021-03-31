//! Tests for the module.

use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn creating_registry_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1)));
    });
}
