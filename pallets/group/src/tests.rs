//! Tests for the module.
use crate::mock::*;
use chrono::Utc;
use frame_support::assert_ok;
use primitives::group::Group;

#[test]
fn creating_new_group_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);
    });
}
