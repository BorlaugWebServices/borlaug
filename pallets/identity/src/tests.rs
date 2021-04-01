//! Tests for the module.
use crate::mock::*;
use chrono::Utc;
use frame_support::assert_ok;
use primitives::{
    claim::{ClaimConsumer, ClaimIssuer, Statement},
    did_document::DidDocument,
    did_property::DidProperty,
    fact::Fact,
};

#[test]
fn registering_did_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

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

#[test]
fn managing_controllers_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);
        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let did_1 = dids[0];

        // 1 adds 2 as controller
        assert_ok!(Identity::manage_controllers(
            Origin::signed(1),
            did_1,
            Some(vec![2]),
            None
        ));
        assert_eq!(Identity::controller(&1), vec![did_1]);
        assert_eq!(Identity::controller(&2), vec![did_1]);

        // 1 removes 2 as controller
        assert_ok!(Identity::manage_controllers(
            Origin::signed(1),
            did_1,
            None,
            Some(vec![2])
        ));
        assert_eq!(Identity::controller(&1), vec![did_1]);
        assert_eq!(Identity::controller(&2), vec![]);
    });
}

#[test]
fn update_did_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::register_did(Origin::signed(1), None));

        let dids = Identity::dids(&1);
        let did_1 = dids[0];

        assert_ok!(Identity::update_did(
            Origin::signed(1),
            did_1,
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
            Some(vec![])
        ));

        assert_eq!(
            Identity::did_document(did_1),
            DidDocument {
                properties: vec![
                    DidProperty {
                        name: b"name".to_vec(),
                        fact: Fact::Text(b"John Doe".to_vec())
                    },
                    DidProperty {
                        name: b"age".to_vec(),
                        fact: Fact::U8(255)
                    }
                ]
            }
        );

        assert_ok!(Identity::update_did(
            Origin::signed(1),
            did_1,
            Some(vec![
                DidProperty {
                    name: b"birthday".to_vec(),
                    fact: Fact::Date(1980, 1, 1)
                },
                DidProperty {
                    name: b"citizen".to_vec(),
                    fact: Fact::Bool(true)
                }
            ]),
            Some(vec![b"age".to_vec()])
        ));

        assert_eq!(
            Identity::did_document(did_1),
            DidDocument {
                properties: vec![
                    DidProperty {
                        name: b"name".to_vec(),
                        fact: Fact::Text(b"John Doe".to_vec())
                    },
                    DidProperty {
                        name: b"birthday".to_vec(),
                        fact: Fact::Date(1980, 1, 1)
                    },
                    DidProperty {
                        name: b"citizen".to_vec(),
                        fact: Fact::Bool(true)
                    }
                ]
            }
        );
    });
}

#[test]
fn replacing_properties_to_did_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        assert_ok!(Identity::register_did(Origin::signed(1), None));

        let dids = Identity::dids(&1);
        let did_1 = dids[0];

        assert_ok!(Identity::update_did(
            Origin::signed(1),
            did_1,
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
            Some(vec![])
        ));

        assert_eq!(
            Identity::did_document(did_1),
            DidDocument {
                properties: vec![
                    DidProperty {
                        name: b"name".to_vec(),
                        fact: Fact::Text(b"John Doe".to_vec())
                    },
                    DidProperty {
                        name: b"age".to_vec(),
                        fact: Fact::U8(255)
                    }
                ]
            }
        );

        assert_ok!(Identity::replace_did(
            Origin::signed(1),
            did_1,
            vec![
                DidProperty {
                    name: b"new name".to_vec(),
                    fact: Fact::Text(b"John Doe".to_vec())
                },
                DidProperty {
                    name: b"new age".to_vec(),
                    fact: Fact::U8(255)
                }
            ]
        ));

        assert_eq!(
            Identity::did_document(did_1),
            DidDocument {
                properties: vec![
                    DidProperty {
                        name: b"new name".to_vec(),
                        fact: Fact::Text(b"John Doe".to_vec())
                    },
                    DidProperty {
                        name: b"new age".to_vec(),
                        fact: Fact::U8(255)
                    }
                ]
            }
        );
    });
}

#[test]
fn catalogs_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // Target
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let did_1 = dids[0];

        // Claim consumer
        assert_ok!(Identity::register_did(Origin::signed(1000), None));
        let dids = Identity::dids(&1000);
        let did_1000 = dids[0];

        // Claim issuer
        assert_ok!(Identity::register_did(Origin::signed(2000), None));
        let dids = Identity::dids(&2000);
        let did_2000 = dids[0];

        // Catalog for consumers
        assert_ok!(Identity::create_catalog(Origin::signed(1), did_1));
        // Catalog for issuers
        assert_ok!(Identity::create_catalog(Origin::signed(1), did_1));

        let catalogs = Identity::catalog_ownership(&did_1);

        // Add DIDs from catalog
        assert_ok!(Identity::add_dids_to_catalog(
            Origin::signed(1),
            did_1,
            catalogs[0],
            vec![(did_1000, b"Consulate".to_vec())]
        ));
        assert_ok!(Identity::add_dids_to_catalog(
            Origin::signed(1),
            did_1,
            catalogs[1],
            vec![(did_2000, b"Employer".to_vec())]
        ));

        assert_eq!(
            Identity::catalogs(catalogs[0], did_1000),
            Some(b"Consulate".to_vec())
        );
        assert_eq!(
            Identity::catalogs(catalogs[1], did_2000),
            Some(b"Employer".to_vec())
        );

        // Remove DIDs from catalog
        assert_ok!(Identity::remove_dids_from_catalog(
            Origin::signed(1),
            did_1,
            catalogs[1],
            vec![did_2000]
        ));
        assert_eq!(Identity::catalogs(catalogs[1], did_2000), None);

        // Remove catalog
        assert_ok!(Identity::remove_catalog(
            Origin::signed(1),
            did_1,
            catalogs[0]
        ));
        assert_eq!(Identity::catalog_ownership(did_1), vec![catalogs[1]]);
    });
}

#[test]
fn consumer_authorizations_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // Target
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let did_1 = dids[0];

        // Claim consumer
        assert_ok!(Identity::register_did(Origin::signed(1000), None));
        let dids = Identity::dids(&1000);
        let did_1000 = dids[0];

        assert_ok!(Identity::register_did(Origin::signed(2000), None));
        let dids = Identity::dids(&2000);
        let did_2000 = dids[0];

        // Grant claim consuming permission to 1000, 2000
        let now = Utc::now().timestamp() as u64;
        assert_ok!(Identity::authorize_claim_consumers(
            Origin::signed(1),
            did_1,
            vec![
                ClaimConsumer {
                    consumer: did_1000,
                    expiration: now + 8640000
                },
                ClaimConsumer {
                    consumer: did_2000,
                    expiration: now + 8640000
                },
            ]
        ));

        // Revoke claim consuming permission from 2000
        assert_ok!(Identity::revoke_claim_consumers(
            Origin::signed(1),
            did_1,
            vec![did_2000]
        ));

        assert_eq!(
            Identity::claim_comsumers(did_1),
            vec![ClaimConsumer {
                consumer: did_1000,
                expiration: now + 8640000
            }]
        );
    });
}

#[test]
fn issuer_authorizations_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // Target
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let did_1 = dids[0];

        // Claim consumer
        assert_ok!(Identity::register_did(Origin::signed(1000), None));
        let dids = Identity::dids(&1000);
        let did_1000 = dids[0];

        assert_ok!(Identity::register_did(Origin::signed(2000), None));
        let dids = Identity::dids(&2000);
        let did_2000 = dids[0];

        // Grant claim attesting permission to 1000, 2000
        let now = Utc::now().timestamp() as u64;
        assert_ok!(Identity::authorize_claim_issuers(
            Origin::signed(1),
            did_1,
            vec![
                ClaimIssuer {
                    issuer: did_1000,
                    expiration: now + 8640000
                },
                ClaimIssuer {
                    issuer: did_2000,
                    expiration: now + 8640000
                },
            ]
        ));

        // Revoke claim attesting permission from 2000
        assert_ok!(Identity::revoke_claim_issuers(
            Origin::signed(1),
            did_1,
            vec![did_2000]
        ));

        assert_eq!(
            Identity::claim_issuers(did_1),
            vec![ClaimIssuer {
                issuer: did_1000,
                expiration: now + 8640000
            }]
        );
    });
}

#[test]
fn claims_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // Target
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let did_1 = dids[0];

        // Claim consumer
        assert_ok!(Identity::register_did(Origin::signed(1000), None));
        let dids = Identity::dids(&1000);
        let did_1000 = dids[0];

        assert_ok!(Identity::register_did(Origin::signed(2000), None));
        let dids = Identity::dids(&2000);
        let did_2000 = dids[0];

        let now = Utc::now().timestamp() as u64;

        // Grant claim consuming permission to 1000
        assert_ok!(Identity::authorize_claim_consumers(
            Origin::signed(1),
            did_1,
            vec![ClaimConsumer {
                consumer: did_1000,
                expiration: now + 8640000
            }]
        ));

        // Grant claim attesting permission to 2000
        assert_ok!(Identity::authorize_claim_issuers(
            Origin::signed(1),
            did_1,
            vec![ClaimIssuer {
                issuer: did_2000,
                expiration: now + 8640000
            }]
        ));

        // 1000 creates some statements for a `No Objection Letter` claim
        assert_ok!(Identity::make_claim(
            Origin::signed(1000),
            did_1000,
            did_1,
            b"No objection Letter".to_vec(),
            vec![
                Statement {
                    name: b"Purpose".to_vec(),
                    fact: Fact::Text(b"Shengen Visa".to_vec()),
                    for_issuer: false
                },
                Statement {
                    name: b"Salary".to_vec(),
                    fact: Fact::U32(70_000),
                    for_issuer: false
                },
                Statement {
                    name: b"TB Immunization Status".to_vec(),
                    fact: Fact::Bool(false),
                    for_issuer: true
                },
                Statement {
                    name: b"OK To Fly".to_vec(),
                    fact: Fact::Bool(false),
                    for_issuer: true
                }
            ]
        ));

        let claim_indexes = Identity::claims_of(&did_1);
        let claim_index = claim_indexes[0];
        assert_eq!(claim_index, 0);

        let claim = Identity::claims(&did_1, claim_index);
        assert_eq!(claim.description, b"No objection Letter".to_vec());
        assert_eq!(
            claim.statements,
            vec![
                Statement {
                    name: b"Purpose".to_vec(),
                    fact: Fact::Text(b"Shengen Visa".to_vec()),
                    for_issuer: false
                },
                Statement {
                    name: b"Salary".to_vec(),
                    fact: Fact::U32(70_000),
                    for_issuer: false
                },
                Statement {
                    name: b"TB Immunization Status".to_vec(),
                    fact: Fact::Bool(false),
                    for_issuer: true
                },
                Statement {
                    name: b"OK To Fly".to_vec(),
                    fact: Fact::Bool(false),
                    for_issuer: true
                }
            ]
        );

        //2000 attests claim index `0` and adds 2 statements for a `No Objection Letter` claim
        let now = Utc::now().timestamp() as u64;
        let tomorrow = now + 8640000;
        assert_ok!(Identity::attest_claim(
            Origin::signed(2000),
            did_2000,
            did_1,
            claim_index,
            vec![
                Statement {
                    name: b"TB Immunization Status".to_vec(),
                    fact: Fact::Bool(true),
                    for_issuer: true
                },
                Statement {
                    name: b"OK To Fly".to_vec(),
                    fact: Fact::Bool(true),
                    for_issuer: true
                }
            ],
            tomorrow
        ));

        // Verify attestation was successful
        let claim = Identity::claims(&did_1, claim_index);
        assert_eq!(
            claim.statements,
            vec![
                Statement {
                    name: b"Purpose".to_vec(),
                    fact: Fact::Text(b"Shengen Visa".to_vec()),
                    for_issuer: false
                },
                Statement {
                    name: b"Salary".to_vec(),
                    fact: Fact::U32(70_000),
                    for_issuer: false
                },
                Statement {
                    name: b"TB Immunization Status".to_vec(),
                    fact: Fact::Bool(true),
                    for_issuer: true
                },
                Statement {
                    name: b"OK To Fly".to_vec(),
                    fact: Fact::Bool(true),
                    for_issuer: true
                }
            ]
        );

        assert_ok!(Identity::revoke_attestation(
            Origin::signed(2000),
            did_2000,
            did_1,
            claim_index
        ));

        let claim = Identity::claims(&did_1, claim_index);
        assert_eq!(claim.attestation, None);
    });
}
