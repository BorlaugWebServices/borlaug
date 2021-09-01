//! Tests for the module.
use super::*;
use crate::mock::*;
use frame_support::assert_ok;
use primitives::*;
use std::convert::TryInto;

#[test]
fn create_registry_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(1),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        assert!(Registries::<Test>::contains_key(1, registry_id));
        let registry = Registries::<Test>::get(1, registry_id).unwrap();
        assert_eq!(
            Registry {
                name: b"John Doe".to_vec().try_into().unwrap()
            },
            registry
        );
    });
}

#[test]
fn updating_registry_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(1),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        assert!(Registries::<Test>::contains_key(1, registry_id));
        let registry = Registries::<Test>::get(1, registry_id).unwrap();
        assert_eq!(
            Registry {
                name: b"John Doe".to_vec().try_into().unwrap()
            },
            registry
        );
        assert_ok!(Provenance::update_registry(
            Origin::signed(1),
            registry_id,
            b"John Snow".to_vec()
        ));
        let registry = Registries::<Test>::get(1, registry_id).unwrap();
        assert_eq!(
            Registry {
                name: b"John Snow".to_vec().try_into().unwrap()
            },
            registry
        );
    });
}

#[test]
fn remove_registry_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(1),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        assert!(Registries::<Test>::contains_key(1, registry_id));
        assert_ok!(Provenance::remove_registry(Origin::signed(1), registry_id));
        assert!(!Registries::<Test>::contains_key(1, registry_id));
    });
}

#[test]
fn create_definition_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(1),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        assert_ok!(Provenance::create_definition(
            Origin::signed(1),
            registry_id,
            b"Test".to_vec()
        ));
        let definition_id = 1u32;
        assert!(Definitions::<Test>::contains_key(
            registry_id,
            definition_id
        ));
        let definition = Definitions::<Test>::get(registry_id, definition_id).unwrap();
        assert_eq!(
            Definition {
                name: b"Test".to_vec().try_into().unwrap(),
                status: DefinitionStatus::Creating
            },
            definition
        );
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
            1,
            10_000
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1,
            100
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(
                super::Call::create_definition(1u32, b"Test".to_vec(),)
            )),
            1,
            100
        ));

        // remove definition
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(
                super::Call::remove_definition(1u32, 1u32)
            )),
            1,
            100
        ));

        // verify definition was removed
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), false);
    });
}

#[test]
fn create_definition_step_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1,
            10_000
        ));
        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1,
            100
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(
                super::Call::create_definition(1u32, b"Test".to_vec(),)
            )),
            1,
            100
        ));

        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        let group_maybe = Groups::get_group(1);
        assert!(group_maybe.is_some());
        let group = group_maybe.unwrap();
        let group_account = group.anonymous_account;

        // 1 adds step to definition
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(
                super::Call::create_definition_step(
                    1,
                    1,
                    0,
                    b"Test".to_vec(),
                    Some(group_account),
                    1
                )
            )),
            1,
            100
        ));

        assert!(DefinitionSteps::<Test>::contains_key((1, 1), 0));
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
            1,
            10_000
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1,
            100
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(
                super::Call::create_definition(1u32, b"Test".to_vec(),)
            )),
            1,
            100
        ));

        // verify definition was created
        assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

        let group_maybe = Groups::get_group(1);
        assert!(group_maybe.is_some());
        let group = group_maybe.unwrap();
        let group_account = group.anonymous_account;

        // 1 adds step to definition
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(
                super::Call::create_definition_step(
                    1,
                    1,
                    0,
                    b"Test".to_vec(),
                    Some(group_account),
                    1
                )
            )),
            1,
            100
        ));

        let definition_steps = Provenance::get_definition_steps(1, 1);

        assert_eq!(
            DefinitionStep {
                name: b"Test".to_vec().try_into().unwrap(),
                attestor: Some(group_account),
                threshold: 1
            },
            definition_steps[0].1
        );

        // update definition step
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(
                super::Call::update_definition_step(
                    1u32,
                    1u32,
                    0,
                    Some(b"John Doe".to_vec()),
                    None,
                    None
                )
            )),
            1,
            100
        ));

        let definition_steps = Provenance::get_definition_steps(1, 1);

        assert_eq!(
            DefinitionStep {
                name: b"John Doe".to_vec().try_into().unwrap(),
                attestor: Some(group_account),
                threshold: 1
            },
            definition_steps[0].1
        );
    });
}

// #[test]
// fn set_definition_active_should_work() {
//     new_test_ext().execute_with(|| {
//         // 1 creates a Group
//         assert_ok!(Groups::create_group(
//             Origin::signed(1),
//             "Test".to_string().into(),
//             vec![1],
//             1,
//             10_000
//         ));

//         // 1 creates a Registry for itself
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
//                 b"John Doe".to_vec()
//             ))),
//             1
//         ));

//         // 1 creates a definition in the registry
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::create_definition(
//                     1u32,
//                     b"Test".to_vec(),
//                     vec![
//                         (DefinitionStep {
//                             name: b"Test".to_vec(),
//                             group_id: None,
//                             threshold: 1
//                         })
//                     ]
//                 )
//             )),
//             1
//         ));

//         // verify definition was created
//         assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

//         // set definition active
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::set_definition_active(1u32, 1u32,)
//             )),
//             1
//         ));

//         let definition = Provenance::get_definition(1, 1).unwrap();

//         // Verify definition is active
//         assert_eq!(definition.status, DefinitionStatus::Active);
//     });
// }

// #[test]
// fn set_definition_inactive_should_work() {
//     new_test_ext().execute_with(|| {
//         // 1 creates a Group
//         assert_ok!(Groups::create_group(
//             Origin::signed(1),
//             "Test".to_string().into(),
//             vec![1],
//             1,
//             10_000
//         ));

//         // 1 creates a Registry for itself
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
//                 b"John Doe".to_vec()
//             ))),
//             1
//         ));

//         // 1 creates a definition in the registry
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::create_definition(
//                     1u32,
//                     b"Test".to_vec(),
//                     vec![
//                         (DefinitionStep {
//                             name: b"Test".to_vec(),
//                             group_id: None,
//                             threshold: 1
//                         })
//                     ]
//                 )
//             )),
//             1
//         ));

//         // verify definition was created
//         assert_eq!(Definitions::<Test>::contains_key(1u32, 1u32), true);

//         // set definition inactive
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::set_definition_inactive(1u32, 1u32,)
//             )),
//             1
//         ));

//         let definition = Provenance::get_definition(1, 1).unwrap();

//         // Verify definition is active
//         assert_eq!(definition.status, DefinitionStatus::Inactive);
//     });
// }

#[test]
fn create_process_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1,
            10_000
        ));

        // 1 creates a Registry for itself
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
                b"John Doe".to_vec()
            ))),
            1,
            100
        ));

        // 1 creates a definition in the registry
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(
                super::Call::create_definition(1u32, b"Test".to_vec(),)
            )),
            1,
            100
        ));

        let group_maybe = Groups::get_group(1);
        assert!(group_maybe.is_some());
        let group = group_maybe.unwrap();
        let group_account = group.anonymous_account;

        // 1 adds step to definition
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(
                super::Call::create_definition_step(
                    1,
                    1,
                    0,
                    b"Test".to_vec(),
                    Some(group_account),
                    1
                )
            )),
            1,
            100
        ));

        assert!(DefinitionSteps::<Test>::contains_key((1, 1), 0));

        // set definition active
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Provenance(
                super::Call::set_definition_active(1u32, 1u32,)
            )),
            1,
            100
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
            1,
            100
        ));

        // verify process was created
        assert!(Processes::<Test>::contains_key((1u32, 1u32), 1));
    });
}

// #[test]
// fn update_process_should_work() {
//     new_test_ext().execute_with(|| {
//         // 1 creates a Group
//         assert_ok!(Groups::create_group(
//             Origin::signed(1),
//             "Test".to_string().into(),
//             vec![1],
//             1,
//             10_000
//         ));

//         // 1 creates a Registry for itself
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
//                 b"John Doe".to_vec()
//             ))),
//             1
//         ));

//         // 1 creates a definition in the registry
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::create_definition(
//                     1u32,
//                     b"Test".to_vec(),
//                     vec![
//                         (DefinitionStep {
//                             name: b"Test".to_vec(),
//                             group_id: Some(1),
//                             threshold: 1
//                         })
//                     ]
//                 )
//             )),
//             1
//         ));

//         // set definition active
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::set_definition_active(1u32, 1u32,)
//             )),
//             1
//         ));

//         let definition = Provenance::get_definition(1, 1).unwrap();
//         // Verify definition is active
//         assert_eq!(definition.status, DefinitionStatus::Active);

//         // Create process as a group
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::create_process(
//                 1u32,
//                 1u32,
//                 b"Test".to_vec(),
//             ))),
//             1
//         ));

//         // Update process definition
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::update_process(
//                 1u32,
//                 1u32,
//                 1u32,
//                 b"John Doe".to_vec(),
//             ))),
//             1
//         ));

//         let process = Provenance::get_process(1, 1, 1).unwrap();

//         // verify Updating of process name
//         assert_eq!(process.name, b"John Doe".to_vec());
//     });
// }

// #[test]
// fn remove_process_should_work() {
//     new_test_ext().execute_with(|| {
//         // 1 creates a Group
//         assert_ok!(Groups::create_group(
//             Origin::signed(1),
//             "Test".to_string().into(),
//             vec![1],
//             1,
//             10_000
//         ));

//         // 1 creates a Registry for itself
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
//                 b"John Doe".to_vec()
//             ))),
//             1
//         ));

//         // 1 creates a definition in the registry
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::create_definition(
//                     1u32,
//                     b"Test".to_vec(),
//                     vec![
//                         (DefinitionStep {
//                             name: b"Test".to_vec(),
//                             group_id: Some(1),
//                             threshold: 1
//                         })
//                     ]
//                 )
//             )),
//             1
//         ));

//         // set definition active
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::set_definition_active(1u32, 1u32,)
//             )),
//             1
//         ));

//         let definition = Provenance::get_definition(1, 1).unwrap();
//         // Verify definition is active
//         assert_eq!(definition.status, DefinitionStatus::Active);

//         // Create process as a group
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::create_process(
//                 1u32,
//                 1u32,
//                 b"Test".to_vec(),
//             ))),
//             1
//         ));

//         // 1 creates a process
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::remove_process(
//                 1u32, 1u32, 1u32,
//             ))),
//             1
//         ));

//         // verify process was removed
//         assert_eq!(Processes::<Test>::contains_key((1u32, 1u32), 1u32), false);
//     });
// }

// #[test]
// fn update_process_step_should_work() {
//     new_test_ext().execute_with(|| {
//         // 1 creates a Group
//         assert_ok!(Groups::create_group(
//             Origin::signed(1),
//             "Test".to_string().into(),
//             vec![1],
//             1,
//             10_000
//         ));

//         // 1 creates a Registry for itself
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
//                 b"John Doe".to_vec()
//             ))),
//             1
//         ));

//         // 1 creates a definition in the registry
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::create_definition(
//                     1u32,
//                     b"Test".to_vec(),
//                     vec![
//                         (DefinitionStep {
//                             name: b"Test".to_vec(),
//                             group_id: Some(1),
//                             threshold: 1
//                         })
//                     ]
//                 )
//             )),
//             1
//         ));

//         // set definition active
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::set_definition_active(1u32, 1u32,)
//             )),
//             1
//         ));

//         let definition = Provenance::get_definition(1, 1).unwrap();
//         // Verify definition is active
//         assert_eq!(definition.status, DefinitionStatus::Active);

//         // Create process as a group
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::create_process(
//                 1u32,
//                 1u32,
//                 b"Test".to_vec(),
//             ))),
//             1
//         ));

//         // 1 creates a process step
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::update_process_step(
//                     1u32,
//                     1u32,
//                     1u32,
//                     0,
//                     vec![Attribute {
//                         name: b"Test".to_vec(),
//                         fact: Fact::Text(b"Test".to_vec())
//                     }]
//                 )
//             )),
//             1
//         ));

//         // verify process step was created
//         assert_eq!(
//             ProcessSteps::<Test>::contains_key((1u32, 1u32, 1u32), 0),
//             true
//         );
//     });
// }

// #[test]
// fn attest_process_step_should_work() {
//     new_test_ext().execute_with(|| {
//         // 1 creates a Group
//         assert_ok!(Groups::create_group(
//             Origin::signed(1),
//             "Test".to_string().into(),
//             vec![1],
//             1,
//             10_000
//         ));

//         // 1 creates a Registry for itself
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::create_registry(
//                 b"John Doe".to_vec()
//             ))),
//             1
//         ));

//         // 1 creates a definition in the registry
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::create_definition(
//                     1u32,
//                     b"Test".to_vec(),
//                     vec![
//                         (DefinitionStep {
//                             name: b"Test".to_vec(),
//                             group_id: Some(1),
//                             threshold: 1
//                         })
//                     ]
//                 )
//             )),
//             1
//         ));

//         // set definition active
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::set_definition_active(1u32, 1u32,)
//             )),
//             1
//         ));

//         let definition = Provenance::get_definition(1, 1).unwrap();
//         // Verify definition is active
//         assert_eq!(definition.status, DefinitionStatus::Active);

//         // Create process as a group
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(super::Call::create_process(
//                 1u32,
//                 1u32,
//                 b"Test".to_vec(),
//             ))),
//             1
//         ));

//         // 1 creates a process step
//         assert_ok!(Groups::propose(
//             Origin::signed(1),
//             1,
//             Box::new(crate::mock::Call::Provenance(
//                 super::Call::attest_process_step(1u32, 1u32, 1u32, 0)
//             )),
//             1
//         ));

//         let process_step = Provenance::get_process_step(1u32, 1u32, 1u32, 0).unwrap();

//         // verify process step attested
//         assert_eq!(process_step.attested, true);
//     });
// }
