//! Tests for the module.

use super::*;
use crate::mock::*;
use chrono::Utc;
use core::convert::TryInto;
use frame_support::{assert_err, assert_ok, dispatch::Weight};
use primitives::{bounded_vec::BoundedVec, *};

fn create_did() -> Did {
    let _ = Identity::register_did(Origin::signed(1), None);
    let mut dids_by_controller = Vec::new();
    identity::DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
        dids_by_controller.push(did);
    });
    dids_by_controller[0]
}

fn create_registry(did: Did) -> u32 {
    let name = b"name".to_vec();
    assert_ok!(AssetRegistry::create_registry(Origin::signed(1), did, name));
    1u32 //registry_id
}

fn create_asset(did: Did, registry_id: u32) {
    let now = Utc::now().timestamp() as u64;
    let asset = Asset {
        properties: vec![],
        name: b"Cat".to_vec(),
        asset_number: Some(b"CAR_001".to_vec()),
        status: AssetStatus::Active,
        serial_number: Some(b"1234567890".to_vec()),
        total_shares: 100,
        residual_value: Some(1_000_000),
        purchase_value: Some(1_000_000),
        acquired_date: Some(now),
    };
    assert_ok!(AssetRegistry::create_asset(
        Origin::signed(1),
        did,
        registry_id,
        asset
    ));
}

fn create_lease(did_lessor: Did, did_lessee: Did) {
    let registry_id = create_registry(did_lessor);
    create_asset(did_lessor, registry_id);
    let asset_id = 1u32;
    let now = Utc::now().timestamp() as u64;
    let next_week = (Utc::now().timestamp() + 60 * 60 * 24 * 7) as u64;
    let lease = LeaseAgreement {
        proposal_id: None,
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
}

#[test]
fn creating_registry_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a DID for itself
        let did_1 = create_did();

        let name = b"name".to_vec();

        assert_ok!(AssetRegistry::create_registry(
            Origin::signed(1),
            did_1,
            name
        ));
        let registry_id = 1u32;
        let limited_name: BoundedVec<u8, <Test as Config>::NameLimit> =
            b"name".to_vec().try_into().unwrap();
        assert!(Registries::<Test>::contains_key(&did_1, registry_id));
        let registry = Registries::<Test>::get(&did_1, registry_id).unwrap();
        assert_eq!(registry, Registry { name: limited_name });
    });
}

#[test]
fn updating_registry_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a DID for itself
        let did_1 = create_did();

        let name = b"name".to_vec();
        assert_ok!(AssetRegistry::create_registry(
            Origin::signed(1),
            did_1,
            name
        ));
        let registry_id = 1u32;

        let new_name = b"new name".to_vec();
        assert_ok!(AssetRegistry::update_registry(
            Origin::signed(1),
            did_1,
            registry_id,
            new_name
        ));

        let limited_name: BoundedVec<u8, <Test as Config>::NameLimit> =
            b"new name".to_vec().try_into().unwrap();
        assert!(Registries::<Test>::contains_key(&did_1, registry_id));
        let registry = Registries::<Test>::get(&did_1, registry_id).unwrap();
        assert_eq!(registry, Registry { name: limited_name });
    });
}

#[test]
fn deleting_registry_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a DID for itself
        let did_1 = create_did();

        let name = b"name".to_vec();
        assert_ok!(AssetRegistry::create_registry(
            Origin::signed(1),
            did_1,
            name
        ));
        let registry_id = 1u32;

        assert_ok!(AssetRegistry::delete_registry(
            Origin::signed(1),
            did_1,
            registry_id,
        ));

        assert!(!Registries::<Test>::contains_key(&did_1, registry_id));
    });
}

#[test]
fn creating_asset_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        let did_1 = create_did();

        let registry_id = create_registry(did_1);

        let now = Utc::now().timestamp() as u64;

        let properties = vec![AssetProperty {
            name: b"property name".to_vec(),
            fact: Fact::Text(b"fact text".to_vec()),
        }];

        let asset = Asset {
            properties,
            name: b"Cat".to_vec(),
            asset_number: Some(b"CAR_001".to_vec()),
            status: AssetStatus::Active,
            serial_number: Some(b"1234567890".to_vec()),
            total_shares: 100,
            residual_value: Some(1_000_000),
            purchase_value: Some(1_000_000),
            acquired_date: Some(now),
        };
        assert_ok!(AssetRegistry::create_asset(
            Origin::signed(1),
            did_1,
            registry_id,
            asset
        ));

        let asset_id = 1u32;

        assert!(Assets::<Test>::contains_key(registry_id, asset_id));
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
            properties: vec![],
            name: b"Dog".to_vec(),
            asset_number: Some(b"CAR_002".to_vec()),
            status: AssetStatus::Active,
            serial_number: Some(b"1234567890".to_vec()),
            total_shares: 200,
            residual_value: Some(1_000_000),
            purchase_value: Some(1_000_000),
            acquired_date: Some(now),
        };

        assert_ok!(AssetRegistry::update_asset(
            Origin::signed(1),
            did_1,
            registry_id,
            asset_id,
            new_asset
        ));

        assert!(Assets::<Test>::contains_key(registry_id, asset_id));
        let stored_asset = Assets::<Test>::get(registry_id, asset_id).unwrap();
        assert_eq!(stored_asset.total_shares, 200);
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

        assert!(!Assets::<Test>::contains_key(registry_id, 1u32));
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
            proposal_id: None,
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

        let lease_id = 1u32;

        assert!(LeaseAgreements::<Test>::contains_key(
            &did_lessor,
            &lease_id
        ));
    });
}

#[test]
fn lease_asset_over_allocation_should_fail() {
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
            proposal_id: None,
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

        let lease_id = 1u32;

        assert!(LeaseAgreements::<Test>::contains_key(
            &did_lessor,
            &lease_id
        ));

        let lease2 = LeaseAgreement {
            proposal_id: None,
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

        assert_err!(
            AssetRegistry::new_lease(Origin::signed(1), lease2),
            Error::<Test>::AssetAllocationFailed
        );

        assert!(!LeaseAgreements::<Test>::contains_key(&did_lessor, 2u32));
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
        let lease_id = 1u32;
        assert_ok!(AssetRegistry::void_lease(
            Origin::signed(1),
            did_lessor,
            lease_id
        ));
        assert!(!LeaseAgreements::<Test>::contains_key(
            &did_lessor,
            lease_id
        ));
    });
}

//Make sure weights cannot exceed 10% of total allowance for block.

#[test]
fn weights_should_not_be_excessive() {
    new_test_ext().execute_with(|| {
        const MAXIMUM_ALLOWED_WEIGHT: Weight = 130_000_000_000;

        let weight =
            <Test as Config>::WeightInfo::create_registry(<Test as Config>::NameLimit::get());
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight =
            <Test as Config>::WeightInfo::update_registry(<Test as Config>::NameLimit::get());
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::delete_registry();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::create_asset(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::FactStringLimit::get(),
            <Test as Config>::AssetPropertyLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::update_asset(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::FactStringLimit::get(),
            <Test as Config>::AssetPropertyLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::delete_asset();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::new_lease(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::LeaseAssetLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight =
            <Test as Config>::WeightInfo::void_lease(<Test as Config>::LeaseAssetLimit::get());
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
    });
}
