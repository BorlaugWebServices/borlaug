//! Tests for the module.

#![cfg(test)]

extern crate chrono;

use super::*;
use crate::mock::{new_test_ext, ExtBuilder, Origin, Provenance, System, Test};
use chrono::Utc;
use frame_support::{assert_noop, assert_ok};

#[test]
fn creating_registry_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1)));
    });
}
