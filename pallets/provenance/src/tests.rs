//! Tests for the module.

#![cfg(test)]

extern crate chrono;

use super::*;
use crate::mock::{new_test_ext, ExtBuilder, Identity, Origin, System, Test};
use chrono::Utc;
use frame_support::{assert_noop, assert_ok};
use primitives::fact::Fact;

#[test]
fn registering_did_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));

        let dids = Identity::dids(&1);
        let did_1 = dids[0];

        assert_eq!(did_1.id.len(), 32);

        // 1 creates a DID for 2
        assert_ok!(Identity::register_did_for(Origin::signed(1), 2u64, None));

        let dids = Identity::dids(&2);
        let did_2 = dids[0];

        // 1 is the controller of both DIDs
        assert_eq!(Identity::controller(&1), vec![did_1, did_2]);
    });
}
