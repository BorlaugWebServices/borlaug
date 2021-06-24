//! Tests for the module.
use super::*;
use crate::mock::*;
use frame_support::assert_ok;
use primitives::{attribute::Attribute, definition_step::DefinitionStep, fact::Fact};

#[test]
fn creating_registry_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1), b"John Doe".to_vec()));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);
    });
}

#[test]
fn remove_registry_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1), b"John Doe".to_vec()));
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
        assert_ok!(Provenance::create_registry(Origin::signed(1), b"John Doe".to_vec()));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                    group_id: 0
                }
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
        assert_ok!(Provenance::create_registry(Origin::signed(1), b"John Doe".to_vec()));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                    group_id: 0
                }
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
        assert_ok!(Provenance::create_registry(Origin::signed(1), b"John Doe".to_vec()));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                    group_id: 0
                }
            )]
        ));
        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // update definition step
        assert_ok!(Provenance::update_definition_step(
            Origin::signed(1),
            1u32,
            1u32,
            0,
            b"John Doe".to_vec(),
        ));
    });
}

#[test]
fn create_process_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1), b"John Doe".to_vec()));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                    group_id: 1
                }
            )]
        ));
        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // set definition step as active
        assert_ok!(Provenance::set_definition_active(
            Origin::signed(1),
            1u32,
            1u32,
        ));

        // Create process as a group
        assert_ok!(Groups::as_group(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_process(
                1u32,
                1u32,
                b"Test".to_vec(),
            ))),
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
        assert_ok!(Provenance::create_registry(Origin::signed(1), b"John Doe".to_vec()));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                    group_id: 1
                }
            )]
        ));
        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // set definition step as active
        assert_ok!(Provenance::set_definition_active(
            Origin::signed(1),
            1u32,
            1u32,
        ));

        // Create process as a group
        assert_ok!(Groups::as_group(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_process(
                1u32,
                1u32,
                b"Test".to_vec(),
            ))),
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
        assert_ok!(Provenance::create_registry(Origin::signed(1), b"John Doe".to_vec()));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                    group_id: 1
                }
            )]
        ));
        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // set definition step as active
        assert_ok!(Provenance::set_definition_active(
            Origin::signed(1),
            1u32,
            1u32,
        ));

        // Create process as a group
        assert_ok!(Groups::as_group(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_process(
                1u32,
                1u32,
                b"Test".to_vec(),
            ))),
        ));

        // verify process was created
        assert_eq!(Processes::<Test>::contains_key((1u32, 1u32), 1u32), true);

        // 1 creates a process step
        assert_ok!(Groups::as_group(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::update_process_step(
                1u32,
                1u32,
                1u32,
                0,
                vec![Attribute {
                    name: b"Test".to_vec(),
                    fact: Fact::Text(b"Test".to_vec())
                }]
            ))),
        ));

        // verify process step was created
        assert_eq!(
            ProcessSteps::<Test>::contains_key((1u32, 1u32, 1u32), 0),
            true
        );
    });
}

#[test]
fn attest_process_step_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Registry for itself
        assert_ok!(Provenance::create_registry(Origin::signed(1), b"John Doe".to_vec()));
        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                DefinitionStep {
                    name: b"Test".to_vec(),
                    group_id: 1
                }
            )]
        ));
        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // set definition step as active
        assert_ok!(Provenance::set_definition_active(
            Origin::signed(1),
            1u32,
            1u32,
        ));

        // Create process as a group
        assert_ok!(Groups::as_group(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_process(
                1u32,
                1u32,
                b"Test".to_vec(),
            ))),
        ));

        // verify process was created
        assert_eq!(Processes::<Test>::contains_key((1u32, 1u32), 1u32), true);

        // 1 creates a process step
        assert_ok!(Groups::as_group(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::attest_process_step(
                1u32,
                1u32,
                1u32,
                0
            ))),
        ));
    });
}
