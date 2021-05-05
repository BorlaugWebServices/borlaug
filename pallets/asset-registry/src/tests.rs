//! Tests for the module.

use super::*;
use crate::mock::*;
use chrono::Utc;
use frame_support::{assert_err, assert_err_ignore_postinfo, assert_ok};
use primitives::{
    asset::{Asset, AssetStatus},
    did::Did,
    lease_agreement::{AssetAllocation, LeaseAgreement},
};

fn create_did() -> Did {
    let _ = Identity::register_did(Origin::signed(1), None);
    let dids = Identity::dids(&1);
    *dids.last().unwrap()
}

fn create_registry(did: Did) -> u32 {
    let _ = AssetRegistry::create_registry(Origin::signed(1), did);
    *AssetRegistry::registries(&did).last().unwrap()
}

//warning this function doesn't work if called multiple times.
fn create_asset(did: Did, registry_id: u32) {
    let now = Utc::now().timestamp() as u64;
    let asset = Asset {
        properties: None,
        name: Some(b"Cat".to_vec()),
        asset_number: Some(b"CAR_001".to_vec()),
        status: Some(AssetStatus::Active),
        serial_number: Some(b"1234567890".to_vec()),
        total_shares: Some(100),
        residual_value: Some(1_000_000),
        purchase_value: Some(1_000_000),
        acquired_date: Some(now),
    };
    let _ = AssetRegistry::create_asset(Origin::signed(1), did, registry_id, asset);
}

fn create_lease(did_lessor: Did, did_lessee: Did) {
    let registry_id = create_registry(did_lessor);
    create_asset(did_lessor, registry_id);
    let asset_id = 0u32;
    let now = Utc::now().timestamp() as u64;
    let next_week = (Utc::now().timestamp() + 60 * 60 * 24 * 7) as u64;
    let lease = LeaseAgreement {
        contract_number: b"001".to_vec(),
        lessor: did_lessor,
        lessee: did_lessee,
        allocations: vec![AssetAllocation {
            registry_id,
            asset_id,
            allocated_shares: 50,
        }],
        effective_ts: now,
        expiry_ts: next_week,
    };
    let _ = AssetRegistry::new_lease(Origin::signed(1), lease);
}

#[test]
fn creating_registry_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));

        let dids = Identity::dids(&1);
        let did_1 = dids[0];

        assert_ok!(AssetRegistry::create_registry(Origin::signed(1), did_1));
        assert_eq!(AssetRegistry::registries(&did_1), vec![1u32]);
    });
}

#[test]
fn deleting_registry_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));

        let dids = Identity::dids(&1);
        let did_1 = dids[0];

        assert_ok!(AssetRegistry::create_registry(Origin::signed(1), did_1));
        assert_eq!(AssetRegistry::registries(&did_1), vec![1u32]);

        assert_ok!(AssetRegistry::delete_registry(
            Origin::signed(1),
            did_1,
            1u32
        ));
    });
}

//TODO: add asset properties
#[test]
fn creating_assets_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        let did_1 = create_did();

        let registry_id = create_registry(did_1);

        let now = Utc::now().timestamp() as u64;
        let asset = Asset {
            properties: None,
            name: Some(b"Cat".to_vec()),
            asset_number: Some(b"CAR_001".to_vec()),
            status: Some(AssetStatus::Active),
            serial_number: Some(b"1234567890".to_vec()),
            total_shares: Some(100),
            residual_value: Some(1_000_000),
            purchase_value: Some(1_000_000),
            acquired_date: Some(now),
        };
        assert_ok!(AssetRegistry::create_asset(
            Origin::signed(1),
            did_1,
            registry_id,
            asset.clone()
        ));

        let created_asset = AssetRegistry::assets(registry_id, 1u32);

        assert_eq!(created_asset, asset);
    });
}

#[test]
fn updating_asset_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        let did_1 = create_did();

        let registry_id = create_registry(did_1);

        create_asset(did_1, registry_id);

        let asset_id = 0u32;

        let now = Utc::now().timestamp() as u64;

        let new_asset = Asset {
            properties: None,
            name: Some(b"Dog".to_vec()),
            asset_number: Some(b"CAR_002".to_vec()),
            status: Some(AssetStatus::Active),
            serial_number: Some(b"1234567890".to_vec()),
            total_shares: Some(200),
            residual_value: Some(1_000_000),
            purchase_value: Some(1_000_000),
            acquired_date: Some(now),
        };

        assert_ok!(AssetRegistry::update_asset(
            Origin::signed(1),
            did_1,
            registry_id,
            asset_id,
            new_asset.clone()
        ));

        assert_eq!(AssetRegistry::assets(registry_id, 0u32), new_asset);
    });
}

#[test]
fn deleting_asset_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        let did_1 = create_did();

        let registry_id = create_registry(did_1);

        create_asset(did_1, registry_id);

        let asset_id = 1u32;

        assert_ok!(AssetRegistry::delete_asset(
            Origin::signed(1),
            did_1,
            registry_id,
            asset_id
        ));

        assert_eq!(Assets::<Test>::contains_key(registry_id, 1u32), false);
    });
}

#[test]
fn creating_lease_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a DID for itself (lessor)
        let did_lessor = create_did();

        //2 create DID for lessee
        let did_lessee = create_did();

        let registry_id = create_registry(did_lessor);

        //Create an asset

        create_asset(did_lessor, registry_id);

        let asset_id = 1u32;

        let now = Utc::now().timestamp() as u64;
        let next_week = (Utc::now().timestamp() + 60 * 60 * 24 * 7) as u64;

        let lease = LeaseAgreement {
            contract_number: b"001".to_vec(),
            lessor: did_lessor,
            lessee: did_lessee,
            allocations: vec![AssetAllocation {
                registry_id,
                asset_id,
                allocated_shares: 50,
            }],
            effective_ts: now,
            expiry_ts: next_week,
        };

        assert_ok!(AssetRegistry::new_lease(Origin::signed(1), lease));
    });
}

#[test]
fn lease_asset_over_allocation() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a DID for itself (lessor)
        let did_lessor = create_did();

        //2 create DID for lessee
        let did_lessee = create_did();

        let registry_id = create_registry(did_lessor);

        //Create an asset

        create_asset(did_lessor, registry_id);

        let asset_id = 1u32;

        let now = Utc::now().timestamp() as u64;
        let next_week = (Utc::now().timestamp() + 60 * 60 * 24 * 7) as u64;

        let lease1 = LeaseAgreement {
            contract_number: b"001".to_vec(),
            lessor: did_lessor,
            lessee: did_lessee,
            allocations: vec![AssetAllocation {
                registry_id,
                asset_id,
                allocated_shares: 50,
            }],
            effective_ts: now,
            expiry_ts: next_week,
        };

        assert_ok!(AssetRegistry::new_lease(Origin::signed(1), lease1));

        let lease2 = LeaseAgreement {
            contract_number: b"002".to_vec(),
            lessor: did_lessor,
            lessee: did_lessee,
            allocations: vec![AssetAllocation {
                registry_id,
                asset_id,
                allocated_shares: 51,
            }],
            effective_ts: now,
            expiry_ts: next_week,
        };

        assert_ok!(AssetRegistry::new_lease(Origin::signed(1), lease2));
    });
}

#[test]
fn voiding_lease_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        let did_lessor = create_did();
        let did_lessee = create_did();
        create_lease(did_lessor, did_lessee);
        let lease_id = 0u32;
        assert_ok!(AssetRegistry::void_lease(
            Origin::signed(1),
            did_lessor,
            lease_id
        ));
    });
}
