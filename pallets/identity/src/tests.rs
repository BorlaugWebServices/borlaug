//! Tests for the module.
use super::*;
use crate::mock::*;
use chrono::Utc;
use core::convert::TryInto;
use frame_support::{assert_ok, dispatch::Weight};
use primitives::*;

#[test]
fn register_did_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        let property = DidProperty {
            name: vec![42u8; 5],
            fact: Fact::Text(vec![42u8; 5]),
        };

        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(
            Origin::signed(1),
            Some(vec![property])
        ));

        let mut dids_by_controller = Vec::new();
        DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let mut dids_by_subject = Vec::new();
        DidBySubject::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_subject.push(did);
        });
        assert_eq!(dids_by_subject.len(), 1);

        let did = dids_by_controller[0];
        assert_eq!(did.id.len(), 32);

        let did_document = DidDocuments::<Test>::get(&did);
        assert!(did_document.is_some());

        let mut stored_properties = Vec::new();
        DidDocumentProperties::<Test>::iter_prefix(&did).for_each(|(_, property)| {
            stored_properties.push(property);
        });
        assert_eq!(stored_properties.len(), 1);
    });
}

#[test]
fn register_did_for_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        let property = DidProperty {
            name: vec![42u8; 5],
            fact: Fact::Text(vec![42u8; 5]),
        };

        // 1 creates a DID for 2
        assert_ok!(Identity::register_did_for(
            Origin::signed(1),
            2u64,
            Some(vec![property])
        ));

        let mut dids_by_controller = Vec::new();
        DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let mut dids_by_subject = Vec::new();
        DidBySubject::<Test>::iter_prefix(&2).for_each(|(did, _)| {
            dids_by_subject.push(did);
        });
        assert_eq!(dids_by_subject.len(), 1);

        let did = dids_by_controller[0];
        assert_eq!(did.id.len(), 32);

        assert!(DidDocuments::<Test>::contains_key(&did));

        let mut stored_properties = Vec::new();
        DidDocumentProperties::<Test>::iter_prefix(&did).for_each(|(_, property)| {
            stored_properties.push(property);
        });
        assert_eq!(stored_properties.len(), 1);
    });
}

#[test]
fn add_did_properties_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::register_did(Origin::signed(1), None));

        let mut dids_by_controller = Vec::new();
        DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did = dids_by_controller[0];
        assert!(DidDocuments::<Test>::contains_key(&did));

        //check adding properties
        assert_ok!(Identity::add_did_properties(
            Origin::signed(1),
            did,
            vec![
                DidProperty {
                    name: b"name".to_vec(),
                    fact: Fact::Text(b"John Doe".to_vec())
                },
                DidProperty {
                    name: b"age".to_vec(),
                    fact: Fact::U8(255)
                }
            ]
        ));

        let mut stored_properties = Vec::new();
        DidDocumentProperties::<Test>::iter_prefix(&did).for_each(|(_, property)| {
            stored_properties.push(property);
        });
        assert_eq!(stored_properties.len(), 2);
        assert_eq!(
            stored_properties,
            vec![
                DidProperty {
                    name: b"name".to_vec().try_into().unwrap(),
                    fact: Fact::Text(b"John Doe".to_vec().try_into().unwrap())
                },
                DidProperty {
                    name: b"age".to_vec().try_into().unwrap(),
                    fact: Fact::U8(255)
                }
            ]
        );
    });
}

#[test]
fn remove_did_properties_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::register_did(Origin::signed(1), None));

        let mut dids_by_controller = Vec::new();
        DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did = dids_by_controller[0];
        assert!(DidDocuments::<Test>::contains_key(&did));

        //add properties
        assert_ok!(Identity::add_did_properties(
            Origin::signed(1),
            did,
            vec![
                DidProperty {
                    name: b"name".to_vec(),
                    fact: Fact::Text(b"John Doe".to_vec())
                },
                DidProperty {
                    name: b"age".to_vec(),
                    fact: Fact::U8(255)
                }
            ]
        ));

        let mut stored_properties = Vec::new();
        DidDocumentProperties::<Test>::iter_prefix(&did).for_each(|(_, property)| {
            stored_properties.push(property);
        });
        assert_eq!(stored_properties.len(), 2);
        assert_eq!(
            stored_properties,
            vec![
                DidProperty {
                    name: b"name".to_vec().try_into().unwrap(),
                    fact: Fact::Text(b"John Doe".to_vec().try_into().unwrap())
                },
                DidProperty {
                    name: b"age".to_vec().try_into().unwrap(),
                    fact: Fact::U8(255)
                }
            ]
        );

        //check removing properties

        assert_ok!(Identity::remove_did_properties(
            Origin::signed(1),
            did,
            vec![b"name".to_vec()],
        ));

        let mut stored_properties = Vec::new();
        DidDocumentProperties::<Test>::iter_prefix(&did).for_each(|(_, property)| {
            stored_properties.push(property);
        });
        assert_eq!(stored_properties.len(), 1);
        assert_eq!(
            stored_properties,
            vec![DidProperty {
                name: b"age".to_vec().try_into().unwrap(),
                fact: Fact::U8(255)
            }]
        );
    });
}

#[test]
fn managing_controllers_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);
        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));

        let mut dids_by_controller = Vec::new();
        DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);

        let did = dids_by_controller[0];

        // 1 adds 2 as controller
        assert_ok!(Identity::manage_controllers(
            Origin::signed(1),
            did,
            Some(vec![2]),
            None
        ));
        assert!(DidByController::<Test>::get(&2, &did).is_some());

        // 1 removes 2 as controller
        assert_ok!(Identity::manage_controllers(
            Origin::signed(1),
            did,
            None,
            Some(vec![2])
        ));
        assert!(DidByController::<Test>::get(&2, &did).is_none());
    });
}

#[test]
fn create_catalog_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::create_catalog(Origin::signed(1),));

        let mut catalogs = Vec::new();
        Catalogs::<Test>::iter_prefix(&1).for_each(|(catalog_id, _)| {
            catalogs.push(catalog_id);
        });
        assert_eq!(catalogs.len(), 1);

        let catalog_id = catalogs[0];

        assert!(Catalogs::<Test>::contains_key(1, catalog_id));
    });
}

#[test]
fn add_dids_to_catalog_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::create_catalog(Origin::signed(1),));

        let mut catalogs = Vec::new();
        Catalogs::<Test>::iter_prefix(&1).for_each(|(catalog_id, _)| {
            catalogs.push(catalog_id);
        });
        assert_eq!(catalogs.len(), 1);

        let catalog_id = catalogs[0];

        assert_ok!(Identity::register_did(
            Origin::signed(1),
            Some(vec![DidProperty {
                name: b"name".to_vec(),
                fact: Fact::Text(b"John Doe".to_vec())
            }])
        ));

        let mut dids_by_controller = Vec::new();
        DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did = dids_by_controller[0];

        assert_ok!(Identity::add_dids_to_catalog(
            Origin::signed(1),
            catalog_id,
            vec![did]
        ));

        assert!(DidsByCatalog::<Test>::contains_key(&catalog_id, &did));
    })
}

#[test]
fn authorize_claim_consumer_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::register_did(
            Origin::signed(1),
            Some(vec![DidProperty {
                name: b"name".to_vec(),
                fact: Fact::Text(b"John Doe".to_vec())
            }])
        ));

        let mut dids_by_controller = Vec::new();
        DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did = dids_by_controller[0];

        let now = Utc::now().timestamp() as u64;
        assert_ok!(Identity::authorize_claim_consumers(
            Origin::signed(1),
            did,
            vec![ClaimConsumer {
                consumer: 2u64,
                expiration: now + 8640000
            }]
        ));

        assert!(ClaimConsumers::<Test>::get(&did, &2).is_some());

        let expiry = ClaimConsumers::<Test>::get(&did, &2).unwrap();
        assert_eq!(expiry, now + 8640000);
    })
}

#[test]
fn authorize_claim_issuer_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::register_did(
            Origin::signed(1),
            Some(vec![DidProperty {
                name: b"name".to_vec(),
                fact: Fact::Text(b"John Doe".to_vec())
            }])
        ));

        let mut dids_by_controller = Vec::new();
        DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did = dids_by_controller[0];

        let now = Utc::now().timestamp() as u64;
        assert_ok!(Identity::authorize_claim_issuers(
            Origin::signed(1),
            did,
            vec![ClaimIssuer {
                issuer: 2u64,
                expiration: now + 8640000
            }]
        ));

        assert!(ClaimIssuers::<Test>::get(&did, &2).is_some());

        let expiry = ClaimIssuers::<Test>::get(&did, &2).unwrap();
        assert_eq!(expiry, now + 8640000);
    })
}

#[test]
fn make_claim_without_proposal_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::register_did(
            Origin::signed(1),
            Some(vec![DidProperty {
                name: b"name".to_vec(),
                fact: Fact::Text(b"John Doe".to_vec())
            }])
        ));

        let mut dids_by_controller = Vec::new();
        DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did = dids_by_controller[0];

        let now = Utc::now().timestamp() as u64;
        assert_ok!(Identity::authorize_claim_consumers(
            Origin::signed(1),
            did,
            vec![ClaimConsumer {
                consumer: 2u64,
                expiration: now + 8640000
            }]
        ));

        let mut claim_consumers = Vec::new();
        ClaimConsumers::<Test>::iter_prefix(&did).for_each(|(claim_consumer, _)| {
            claim_consumers.push(claim_consumer);
        });
        assert_eq!(claim_consumers.len(), 1);

        assert!(ClaimConsumers::<Test>::get(&did, &2).is_some());

        assert_ok!(Identity::make_claim(
            Origin::signed(2),
            did,
            b"some desc".to_vec(),
            vec![Statement {
                name: b"name".to_vec(),
                fact: Fact::Text(b"John Doe".to_vec()),
                for_issuer: false
            }],
            1u32
        ));

        assert!(Claims::<Test>::get(&did, &1).is_some());

        let claim = Claims::<Test>::get(&did, &1).unwrap();

        assert_eq!(claim.description, b"some desc".to_vec());
        assert_eq!(
            claim.statements,
            vec![Statement {
                name: b"name".to_vec().try_into().unwrap(),
                fact: Fact::Text(b"John Doe".to_vec().try_into().unwrap()),
                for_issuer: false
            }]
        );
        assert_eq!(claim.threshold, 1u32);
    });
}

#[test]
fn attest_claim_without_proposal_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::register_did(
            Origin::signed(1),
            Some(vec![DidProperty {
                name: b"name".to_vec(),
                fact: Fact::Text(b"John Doe".to_vec())
            }])
        ));

        let mut dids_by_controller = Vec::new();
        DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did = dids_by_controller[0];

        let now = Utc::now().timestamp() as u64;
        assert_ok!(Identity::authorize_claim_consumers(
            Origin::signed(1),
            did,
            vec![ClaimConsumer {
                consumer: 2u64,
                expiration: now + 8640000
            }]
        ));

        let mut claim_consumers = Vec::new();
        ClaimConsumers::<Test>::iter_prefix(&did).for_each(|(claim_consumer, _)| {
            claim_consumers.push(claim_consumer);
        });
        assert_eq!(claim_consumers.len(), 1);

        assert!(ClaimConsumers::<Test>::get(&did, &2).is_some());

        assert_ok!(Identity::make_claim(
            Origin::signed(2),
            did,
            b"some desc".to_vec(),
            vec![Statement {
                name: b"name".to_vec(),
                fact: Fact::Text(b"John Doe".to_vec()),
                for_issuer: false
            }],
            1u32
        ));

        assert!(Claims::<Test>::get(&did, &1).is_some());

        let claim = Claims::<Test>::get(&did, &1).unwrap();

        assert_eq!(claim.description, b"some desc".to_vec());
        assert_eq!(
            claim.statements,
            vec![Statement {
                name: b"name".to_vec().try_into().unwrap(),
                fact: Fact::Text(b"John Doe".to_vec().try_into().unwrap()),
                for_issuer: false
            }]
        );
        assert_eq!(claim.threshold, 1u32);
        assert!(claim.attestation.is_none());

        let now = Utc::now().timestamp() as u64;
        assert_ok!(Identity::authorize_claim_issuers(
            Origin::signed(1),
            did,
            vec![ClaimIssuer {
                issuer: 3u64,
                expiration: now + 8640000
            }]
        ));

        assert!(ClaimIssuers::<Test>::get(&did, &3).is_some());

        assert_ok!(Identity::attest_claim(
            Origin::signed(3),
            did,
            1u32,
            vec![Statement {
                name: b"name".to_vec(),
                fact: Fact::Text(b"John Doe".to_vec()),
                for_issuer: true
            }],
            now + 8640000
        ));

        let claim_after_attestation = Claims::<Test>::get(&did, &1).unwrap();

        assert!(claim_after_attestation.attestation.is_some());
    });
}

//Make sure weights cannot exceed 10% of total allowance for block.

#[test]
fn weights_should_not_be_excessive() {
    new_test_ext().execute_with(|| {
        const MAXIMUM_ALLOWED_WEIGHT: Weight = 130_000_000_000;

        let weight = <Test as Config>::WeightInfo::register_did(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::FactStringLimit::get(),
            <Test as Config>::PropertyLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::register_did_for(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::FactStringLimit::get(),
            <Test as Config>::PropertyLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        //register_did_for_bulk
        let weight = <Test as Config>::WeightInfo::register_did_for(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::FactStringLimit::get(),
            <Test as Config>::BulkDidPropertyLimit::get(),
        )
        .saturating_mul(<Test as Config>::BulkDidLimit::get().into());
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::add_did_properties(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::FactStringLimit::get(),
            <Test as Config>::PropertyLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::remove_did_properties(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::PropertyLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::manage_controllers(
            <Test as Config>::ControllerLimit::get(),
            <Test as Config>::ControllerLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::authorize_claim_consumers(
            <Test as Config>::ClaimConsumerLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::revoke_claim_consumers(
            <Test as Config>::ClaimConsumerLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::authorize_claim_issuers(
            <Test as Config>::ClaimIssuerLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::revoke_claim_issuers(
            <Test as Config>::ClaimIssuerLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::make_claim(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::StatementLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::FactStringLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::attest_claim(
            <Test as Config>::StatementLimit::get(),
            <Test as Config>::StatementLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::FactStringLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::revoke_attestation(
            <Test as Config>::StatementLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::FactStringLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::create_catalog();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::remove_catalog();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::add_dids_to_catalog(
            <Test as Config>::CatalogDidLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::remove_dids_from_catalog(
            <Test as Config>::CatalogDidLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
    });
}
