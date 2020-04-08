//! Tests for the module.

#![cfg(test)]

extern crate chrono;

use super::*;
use crate::mock::{new_test_ext, Audits, ExtBuilder, Origin, System, Test};
use chrono::Utc;
use frame_support::{assert_noop, assert_ok};

#[test]
fn creating_audit_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Audits::create_audit(Origin::signed(1)));
    });
}
