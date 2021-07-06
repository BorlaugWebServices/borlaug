//! Tests for the module.
use super::*;
use crate::mock::*;
use frame_support::assert_ok;
use primitives::{attribute::Attribute, definition_step::DefinitionStep, fact::Fact, DefinitionStatus};

#[test]
fn creating_registry_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);
    });
}

#[test]
fn updating_registry_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        let registry_1 = Provenance::get_registry(1, 1).unwrap();
        assert_eq!(b"John Doe".to_vec(), registry_1.name);

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::update_registry(
                1,
                b"John Snow".to_vec()
            ))),
            1
        ));

        let registry_2 = Provenance::get_registry(1, 1).unwrap();
        assert_eq!(b"John Snow".to_vec(), registry_2.name);

        // Check names are different
        assert_ne!(registry_1.name, registry_2.name);
    });
}

#[test]
fn remove_registry_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // verify registry was created
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), true);

        // remove the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::remove_registry(
                1u32
            ))),
            1
        ));

        // verify registry was removed
        assert_eq!(Registries::<Test>::contains_key(&1, 1u32), false);
    });
}

#[test]
fn create_definition_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_definition(
                1u32,
                b"Test".to_vec(),
                vec![(
                    DefinitionStep {
                        name: b"Test".to_vec(),
                        group_id: 0,
                        threshold: 1

                    }
                )]
            ))),
            1
        ));

        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);
    });
}

#[test]
fn remove_definition_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_definition(
                1u32,
                b"Test".to_vec(),
                vec![(
                    DefinitionStep {
                        name: b"Test".to_vec(),
                        group_id: 0,
                        threshold: 1

                    }
                )]
            ))),
            1
        ));

        // remove definition
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::remove_definition(
                1u32,
                1u32
            ))),
            1
        ));

        // verify definition was removed
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), false);
    });
}

#[test]
fn update_definition_step_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_definition(
                1u32,
                b"Test".to_vec(),
                vec![(
                    DefinitionStep {
                        name: b"Test".to_vec(),
                        group_id: 0,
                        threshold: 1

                    }
                )]
            ))),
            1
        ));

        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // update definition step
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::update_definition_step(
                1u32,
                1u32,
                0,
                b"John Doe".to_vec(),
            ))),
            1
        ));

        let definition_steps = Provenance::get_definition_steps(1, 1);

        // Check whether name updated or not
        assert_eq!(b"John Doe".to_vec(), definition_steps[0].1.name);
    });
}

#[test]
fn set_definition_active_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_definition(
                1u32,
                b"Test".to_vec(),
                vec![(
                    DefinitionStep {
                        name: b"Test".to_vec(),
                        group_id: 0,
                        threshold: 1

                    }
                )]
            ))),
            1
        ));

        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // set definition active
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::set_definition_active(
                1u32,
                1u32,
            ))),
            1
        ));

        let definition = Provenance::get_definition(1, 1).unwrap();

        // Verify definition is active
        assert_eq!(definition.status, DefinitionStatus::Active);
    });
}

#[test]
fn set_definition_inactive_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_definition(
                1u32,
                b"Test".to_vec(),
                vec![(
                    DefinitionStep {
                        name: b"Test".to_vec(),
                        group_id: 0,
                        threshold: 1

                    }
                )]
            ))),
            1
        ));

        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        // set definition inactive
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::set_definition_inactive(
                1u32,
                1u32,
            ))),
            1
        ));

        let definition = Provenance::get_definition(1, 1).unwrap();

        // Verify definition is active
        assert_eq!(definition.status, DefinitionStatus::Inactive);
    });
}

#[test]
fn create_process_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_definition(
                1u32,
                b"Test".to_vec(),
                vec![(
                    DefinitionStep {
                        name: b"Test".to_vec(),
                        group_id: 1,
                        threshold: 1

                    }
                )]
            ))),
            1
        ));

        // set definition active
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::set_definition_active(
                1u32,
                1u32,
            ))),
            1
        ));

        let definition = Provenance::get_definition(1, 1).unwrap();
        // Verify definition is active
        assert_eq!(definition.status, DefinitionStatus::Active);

        // Create process as a group
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_process(
                1u32,
                1u32,
                b"Test".to_vec(),
            ))),
            1
        ));

        // verify process was created
        assert_eq!(Processes::<Test>::contains_key((1u32, 1u32), 1), true);
    });
}

#[test]
fn update_process_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_definition(
                1u32,
                b"Test".to_vec(),
                vec![(
                    DefinitionStep {
                        name: b"Test".to_vec(),
                        group_id: 1,
                        threshold: 1

                    }
                )]
            ))),
            1
        ));

        // set definition active
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::set_definition_active(
                1u32,
                1u32,
            ))),
            1
        ));

        let definition = Provenance::get_definition(1, 1).unwrap();
        // Verify definition is active
        assert_eq!(definition.status, DefinitionStatus::Active);

        // Create process as a group
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_process(
                1u32,
                1u32,
                b"Test".to_vec(),
            ))),
            1
        ));

        // Update process definition
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::update_process(
                1u32,
                1u32,
                1u32,
                b"John Doe".to_vec(),
            ))),
            1
        ));

        let process = Provenance::get_process(1, 1, 1).unwrap();

        // verify Updating of process name
        assert_eq!(process.name, b"John Doe".to_vec());
    });
}

#[test]
fn remove_process_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_definition(
                1u32,
                b"Test".to_vec(),
                vec![(
                    DefinitionStep {
                        name: b"Test".to_vec(),
                        group_id: 1,
                        threshold: 1

                    }
                )]
            ))),
            1
        ));

        // set definition active
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::set_definition_active(
                1u32,
                1u32,
            ))),
            1
        ));

        let definition = Provenance::get_definition(1, 1).unwrap();
        // Verify definition is active
        assert_eq!(definition.status, DefinitionStatus::Active);

        // Create process as a group
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_process(
                1u32,
                1u32,
                b"Test".to_vec(),
            ))),
            1
        ));

        // 1 creates a process
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::remove_process(
                1u32,
                1u32,
                1u32,
            ))),
            1
        ));

        // verify process was removed
        assert_eq!(Processes::<Test>::contains_key((1u32, 1u32), 1u32), false);
    });
}

#[test]
fn update_process_step_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_definition(
                1u32,
                b"Test".to_vec(),
                vec![(
                    DefinitionStep {
                        name: b"Test".to_vec(),
                        group_id: 1,
                        threshold: 1

                    }
                )]
            ))),
            1
        ));

        // set definition active
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::set_definition_active(
                1u32,
                1u32,
            ))),
            1
        ));

        let definition = Provenance::get_definition(1, 1).unwrap();
        // Verify definition is active
        assert_eq!(definition.status, DefinitionStatus::Active);

        // Create process as a group
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_process(
                1u32,
                1u32,
                b"Test".to_vec(),
            ))),
            1
        ));

        // 1 creates a process step
        assert_ok!(Groups::propose(
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
            1
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
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_definition(
                1u32,
                b"Test".to_vec(),
                vec![(
                    DefinitionStep {
                        name: b"Test".to_vec(),
                        group_id: 1,
                        threshold: 1

                    }
                )]
            ))),
            1
        ));

        // set definition active
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::set_definition_active(
                1u32,
                1u32,
            ))),
            1
        ));

        let definition = Provenance::get_definition(1, 1).unwrap();
        // Verify definition is active
        assert_eq!(definition.status, DefinitionStatus::Active);

        // Create process as a group
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_process(
                1u32,
                1u32,
                b"Test".to_vec(),
            ))),
            1
        ));

        // 1 creates a process step
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::attest_process_step(
                1u32,
                1u32,
                1u32,
                0
            ))),
            1
        ));

        let process_step = Provenance::get_process_step(1u32, 1u32, 1u32, 0).unwrap();

        // verify process step attested
        assert_eq!(process_step.attested, true);
    });
}
