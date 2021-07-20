//! Tests for the module.
use super::*;
use crate::mock::*;
use chrono::Utc;
use core::convert::TryInto;
use frame_support::assert_ok;
use primitives::{bounded_vec::BoundedVec, *};

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
            Some(vec![42u8; 5]),
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
        let did_document = did_document.unwrap();
        assert!(did_document.short_name.is_some());
        assert_eq!(did_document.short_name.unwrap().len(), 5);

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
            Some(vec![42u8; 5]),
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

        let did_document = DidDocuments::<Test>::get(&did);
        assert!(did_document.is_some());
        let did_document = did_document.unwrap();
        assert!(did_document.short_name.is_some());
        assert_eq!(did_document.short_name.unwrap().len(), 5);

        let mut stored_properties = Vec::new();
        DidDocumentProperties::<Test>::iter_prefix(&did).for_each(|(_, property)| {
            stored_properties.push(property);
        });
        assert_eq!(stored_properties.len(), 1);
    });
}

#[test]
fn update_did_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::register_did(Origin::signed(1), None, None));

        let mut dids_by_controller = Vec::new();
        DidByController::<Test>::iter_prefix(&1).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did = dids_by_controller[0];

        //check chainging name
        assert_ok!(Identity::update_did(
            Origin::signed(1),
            did,
            Some(b"name".to_vec()),
            None,
            None
        ));

        let did_document = DidDocuments::<Test>::get(&did);
        assert!(did_document.is_some());
        let did_document = did_document.unwrap();
        assert!(did_document.short_name.is_some());
        assert_eq!(
            did_document.short_name,
            Some(b"name".to_vec().try_into().unwrap())
        );

        //check adding properties
        assert_ok!(Identity::update_did(
            Origin::signed(1),
            did,
            None,
            Some(vec![
                DidProperty {
                    name: b"name".to_vec(),
                    fact: Fact::Text(b"John Doe".to_vec())
                },
                DidProperty {
                    name: b"age".to_vec(),
                    fact: Fact::U8(255)
                }
            ]),
            None
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

        assert_ok!(Identity::update_did(
            Origin::signed(1),
            did,
            None,
            None,
            Some(vec![b"name".to_vec()]),
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
fn replace_did_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::register_did(
            Origin::signed(1),
            None,
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
        let mut stored_properties = Vec::new();
        DidDocumentProperties::<Test>::iter_prefix(&did).for_each(|(_, property)| {
            stored_properties.push(property);
        });
        assert_eq!(stored_properties.len(), 1);
        assert_eq!(
            stored_properties,
            vec![DidProperty {
                name: b"name".to_vec().try_into().unwrap(),
                fact: Fact::Text(b"John Doe".to_vec().try_into().unwrap())
            }]
        );

        assert_ok!(Identity::replace_did(
            Origin::signed(1),
            did,
            vec![
                DidProperty {
                    name: b"name".to_vec(),
                    fact: Fact::Text(b"James Doe".to_vec())
                },
                DidProperty {
                    name: b"age".to_vec(),
                    fact: Fact::U8(255)
                }
            ],
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
                    fact: Fact::Text(b"James Doe".to_vec().try_into().unwrap())
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
fn managing_controllers_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);
        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None, None));

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

        assert_ok!(Identity::create_catalog(
            Origin::signed(1),
            b"name".to_vec()
        ));

        let mut catalogs = Vec::new();
        CatalogOwnership::<Test>::iter_prefix(&1).for_each(|(catalog_id, _)| {
            catalogs.push(catalog_id);
        });
        assert_eq!(catalogs.len(), 1);

        let catalog_id = catalogs[0];

        let catalog = CatalogName::<Test>::get(catalog_id);
        assert!(catalog.is_some());
        let name: BoundedVec<u8, <Test as Config>::NameLimit> =
            b"name".to_vec().try_into().unwrap();
        assert_eq!(catalog.unwrap().name, name);
    });
}

#[test]
fn rename_catalog_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::create_catalog(
            Origin::signed(1),
            b"name".to_vec()
        ));

        let mut catalogs = Vec::new();
        CatalogOwnership::<Test>::iter_prefix(&1).for_each(|(catalog_id, _)| {
            catalogs.push(catalog_id);
        });
        assert_eq!(catalogs.len(), 1);

        let catalog_id = catalogs[0];

        let catalog = CatalogName::<Test>::get(catalog_id);
        assert!(catalog.is_some());
        let name: BoundedVec<u8, <Test as Config>::NameLimit> =
            b"name".to_vec().try_into().unwrap();
        assert_eq!(catalog.unwrap().name, name);

        assert_ok!(Identity::rename_catalog(
            Origin::signed(1),
            catalog_id,
            b"updated name".to_vec()
        ));
        let catalog = CatalogName::<Test>::get(catalog_id);
        assert!(catalog.is_some());
        let name: BoundedVec<u8, <Test as Config>::NameLimit> =
            b"updated name".to_vec().try_into().unwrap();
        assert_eq!(catalog.unwrap().name, name);
    });
}
