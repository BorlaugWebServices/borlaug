//! Tests for the module.
use super::*;
use crate::mock::*;
use frame_support::assert_ok;
use primitives::{
    attestor::Attestor, attribute::Attribute, definition_step::DefinitionStep, fact::Fact,
};

#[test]
fn creating_registry_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1)));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);
    });
}

#[test]
fn remove_registry_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1)));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);
        // remove the registry
        assert_ok!(Provenance::remove_registry(Origin::signed(1), 1u32));
        // verify registry was removed
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), false);
    });
}

#[test]
fn create_definition_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1)));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let attestor_did_1 = dids[0];

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);
    });
}

#[test]
fn remove_definition_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1)));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let attestor_did_1 = dids[0];

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);
        // remove definition
        assert_ok!(Provenance::remove_definition(Origin::signed(1), 1u32, 1u32));
        // verify definition was removed
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), false);
    });
}

#[test]
fn update_definition_step_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1)));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let attestor_did_1 = dids[0];

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let attestor_did_2 = dids[1];

        // update definition step
        assert_ok!(Provenance::update_definition_step(
            Origin::signed(1),
            1u32,
            1u32,
            0,
            Some(vec![Attestor {
                did: attestor_did_2,
                short_name: b"Test".to_vec(),
            }]),
            Some(vec![Attestor {
                did: attestor_did_1,
                short_name: b"Test".to_vec(),
            }]),
        ));
        // verify attestor removed
        assert_eq!(
            Attestors::<Test>::contains_key((1u32, 1u32, 0), attestor_did_1),
            false
        );
        // verify attestor added
        assert_eq!(
            Attestors::<Test>::contains_key((1u32, 1u32, 0), attestor_did_2),
            true
        );
    });
}

#[test]
fn create_process_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1)));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let attestor_did_1 = dids[0];

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // 1 creates a process
        assert_ok!(Provenance::create_process(
            Origin::signed(1),
            attestor_did_1,
            1u32,
            1u32,
            b"Test".to_vec()
        ));
        // verify process was created
        assert_eq!(Processes::<Test>::contains_key((1u32, 1u32), 1u32), true);
    });
}

#[test]
fn remove_process_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1)));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let attestor_did_1 = dids[0];

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // 1 creates a process
        assert_ok!(Provenance::create_process(
            Origin::signed(1),
            attestor_did_1,
            1u32,
            1u32,
            b"Test".to_vec()
        ));
        // verify process was created
        assert_eq!(Processes::<Test>::contains_key((1u32, 1u32), 1u32), true);
        // 1 creates a process
        assert_ok!(Provenance::remove_process(
            Origin::signed(1),
            1u32,
            1u32,
            1u32
        ));
        // verify process was removed
        assert_eq!(Processes::<Test>::contains_key((1u32, 1u32), 1u32), false);
    });
}

#[test]
fn create_process_step_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1)));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let attestor_did_1 = dids[0];

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // 1 creates a process
        assert_ok!(Provenance::create_process(
            Origin::signed(1),
            attestor_did_1,
            1u32,
            1u32,
            b"Test".to_vec()
        ));
        // verify process was created
        assert_eq!(Processes::<Test>::contains_key((1u32, 1u32), 1u32), true);

        // 1 creates a process step
        assert_ok!(Provenance::create_process_step(
            Origin::signed(1),
            attestor_did_1,
            1u32,
            1u32,
            0,
            1u32,
            vec![Attribute {
                name: b"Test".to_vec(),
                fact: Fact::Text(b"Test".to_vec())
            }]
        ));

        // verify process step was created
        assert_eq!(
            ProcessSteps::<Test>::contains_key((1u32, 1u32, 1u32), 0),
            true
        );
    });
}
