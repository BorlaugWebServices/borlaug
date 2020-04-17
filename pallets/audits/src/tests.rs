//! Tests for the module.

#![cfg(test)]

extern crate chrono;

use super::*;
use crate::mock::{Audits, ExtBuilder, Origin};
use frame_support::assert_ok;
use primitives::observation::{Observation, Compliance};
use sp_core::blake2_256;


#[test]
fn creating_audit_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Audits::create_audit(Origin::signed(1)));
    });
}

#[test]
fn creating_observation_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Audits::create_audit(Origin::signed(1)));

        let audits = Audits::audits(&1);
        let audit_1 = audits[0];
        let hello = String::from("hello");

        let observation = Observation {
            observation_id: None,
            compliance: Compliance::Compliant,
            procedural_note: blake2_256(&hello.as_bytes()),
        };

        assert_ok!(Audits::create_observation(
            Origin::signed(1),
            audit_1,
            1,
            observation.clone(),
         ));
    });
}