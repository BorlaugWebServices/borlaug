//! Tests for the module.
use super::*;
use crate::mock::*;
use frame_support::{assert_ok, dispatch::Weight};
use primitives::*;
use std::convert::TryInto;

static DEFINITION_OWNER: u64 = 1;
static ATTESTOR: u64 = 2;

#[test]
fn create_registry_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(DEFINITION_OWNER),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        let registry = Registries::<Test>::get(DEFINITION_OWNER, registry_id).unwrap();
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
            Origin::signed(DEFINITION_OWNER),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        let registry = Registries::<Test>::get(DEFINITION_OWNER, registry_id).unwrap();
        assert_eq!(
            Registry {
                name: b"John Doe".to_vec().try_into().unwrap()
            },
            registry
        );
        assert_ok!(Provenance::update_registry(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            b"John Snow".to_vec()
        ));
        let registry = Registries::<Test>::get(DEFINITION_OWNER, registry_id).unwrap();
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
            Origin::signed(DEFINITION_OWNER),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        assert!(Registries::<Test>::contains_key(
            DEFINITION_OWNER,
            registry_id
        ));
        assert_ok!(Provenance::remove_registry(
            Origin::signed(DEFINITION_OWNER),
            registry_id
        ));
        assert!(!Registries::<Test>::contains_key(
            DEFINITION_OWNER,
            registry_id
        ));
    });
}

#[test]
fn create_definition_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(DEFINITION_OWNER),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        let threshold = 1u32;
        assert_ok!(Provenance::create_definition(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            b"TestDefinition".to_vec(),
            vec![(b"TestStep".to_vec(), ATTESTOR, threshold)]
        ));
        let definition_id = 1u32;
        assert!(Definitions::<Test>::contains_key(
            registry_id,
            definition_id
        ));
        let definition = Definitions::<Test>::get(registry_id, definition_id).unwrap();
        assert_eq!(
            Definition {
                name: b"TestDefinition".to_vec().try_into().unwrap(),
                status: DefinitionStatus::Active
            },
            definition
        );
        let definition_step_index = 0u32;
        assert!(DefinitionSteps::<Test>::contains_key(
            (registry_id, definition_id),
            definition_step_index
        ));
        let definition_step =
            DefinitionSteps::<Test>::get((registry_id, definition_id), definition_step_index)
                .unwrap();
        assert_eq!(
            DefinitionStep {
                name: b"TestStep".to_vec().try_into().unwrap(),
                attestor: ATTESTOR,
                threshold
            },
            definition_step
        );
    });
}

#[test]
fn remove_definition_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(DEFINITION_OWNER),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        let threshold = 1u32;
        assert_ok!(Provenance::create_definition(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            b"TestDefinition".to_vec(),
            vec![(b"TestStep".to_vec(), ATTESTOR, threshold)]
        ));
        let definition_id = 1u32;
        assert!(Definitions::<Test>::contains_key(
            registry_id,
            definition_id
        ));

        // remove definition
        assert_ok!(Provenance::remove_definition(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            definition_id
        ));
        // verify definition was removed
        assert!(!Definitions::<Test>::contains_key(
            registry_id,
            definition_id
        ));
    });
}

#[test]
fn set_definition_inactive_and_active_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(DEFINITION_OWNER),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        let threshold = 1u32;
        assert_ok!(Provenance::create_definition(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            b"TestDefinition".to_vec(),
            vec![(b"TestStep".to_vec(), ATTESTOR, threshold)]
        ));
        let definition_id = 1u32;
        assert!(Definitions::<Test>::contains_key(
            registry_id,
            definition_id
        ));
        assert!(Definitions::<Test>::contains_key(
            registry_id,
            definition_id
        ));

        assert_ok!(Provenance::set_definition_inactive(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            definition_id
        ));

        let definition = Provenance::get_definition(registry_id, definition_id).unwrap();

        // Verify definition is active
        assert_eq!(definition.status, DefinitionStatus::Inactive);

        assert_ok!(Provenance::set_definition_active(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            definition_id
        ));

        let definition = Provenance::get_definition(registry_id, definition_id).unwrap();

        // Verify definition is active
        assert_eq!(definition.status, DefinitionStatus::Active);
    });
}

#[test]
fn create_process_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(DEFINITION_OWNER),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        let threshold = 1u32;
        assert_ok!(Provenance::create_definition(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            b"TestDefinition".to_vec(),
            vec![(b"TestStep".to_vec(), ATTESTOR, threshold)]
        ));
        let definition_id = 1u32;
        assert!(Definitions::<Test>::contains_key(
            registry_id,
            definition_id
        ));

        assert_ok!(Provenance::create_process(
            Origin::signed(ATTESTOR),
            registry_id,
            definition_id,
            b"TestProcess".to_vec(),
        ));
        let process_id = 1u32;
        assert!(Processes::<Test>::contains_key(
            (registry_id, definition_id),
            process_id
        ));
        let process = Processes::<Test>::get((registry_id, definition_id), process_id).unwrap();

        assert_eq!(
            process,
            Process {
                name: b"TestProcess".to_vec().try_into().unwrap(),
                status: ProcessStatus::InProgress
            }
        );
    });
}

#[test]
fn update_process_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(DEFINITION_OWNER),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        let threshold = 1u32;
        assert_ok!(Provenance::create_definition(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            b"TestDefinition".to_vec(),
            vec![(b"TestStep".to_vec(), ATTESTOR, threshold)]
        ));
        let definition_id = 1u32;
        assert!(Definitions::<Test>::contains_key(
            registry_id,
            definition_id
        ));

        assert_ok!(Provenance::create_process(
            Origin::signed(ATTESTOR),
            registry_id,
            definition_id,
            b"TestProcess".to_vec(),
        ));
        let process_id = 1u32;
        assert!(Processes::<Test>::contains_key(
            (registry_id, definition_id),
            process_id
        ));
        let process = Processes::<Test>::get((registry_id, definition_id), process_id).unwrap();

        assert_eq!(
            process,
            Process {
                name: b"TestProcess".to_vec().try_into().unwrap(),
                status: ProcessStatus::InProgress
            }
        );

        assert_ok!(Provenance::update_process(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            definition_id,
            process_id,
            b"RenamedProcess".to_vec(),
        ));
        let process = Processes::<Test>::get((registry_id, definition_id), process_id).unwrap();

        assert_eq!(
            process,
            Process {
                name: b"RenamedProcess".to_vec().try_into().unwrap(),
                status: ProcessStatus::InProgress
            }
        );
    });
}

#[test]
fn remove_process_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(DEFINITION_OWNER),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        let threshold = 1u32;
        assert_ok!(Provenance::create_definition(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            b"TestDefinition".to_vec(),
            vec![(b"TestStep".to_vec(), ATTESTOR, threshold)]
        ));
        let definition_id = 1u32;
        assert!(Definitions::<Test>::contains_key(
            registry_id,
            definition_id
        ));

        assert_ok!(Provenance::create_process(
            Origin::signed(ATTESTOR),
            registry_id,
            definition_id,
            b"TestProcess".to_vec(),
        ));
        let process_id = 1u32;
        assert!(Processes::<Test>::contains_key(
            (registry_id, definition_id),
            process_id
        ));
        let process = Processes::<Test>::get((registry_id, definition_id), process_id).unwrap();

        assert_eq!(
            process,
            Process {
                name: b"TestProcess".to_vec().try_into().unwrap(),
                status: ProcessStatus::InProgress
            }
        );
        assert_ok!(Provenance::remove_process(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            definition_id,
            process_id
        ));
        assert!(!Processes::<Test>::contains_key(
            (registry_id, definition_id),
            process_id
        ));
    });
}

#[test]
fn attest_process_step_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Provenance::create_registry(
            Origin::signed(DEFINITION_OWNER),
            b"John Doe".to_vec()
        ));
        let registry_id = 1u32;
        let threshold = 1u32;
        assert_ok!(Provenance::create_definition(
            Origin::signed(DEFINITION_OWNER),
            registry_id,
            b"TestDefinition".to_vec(),
            vec![
                (b"TestStep_1".to_vec(), ATTESTOR, threshold),
                (b"TestStep_2".to_vec(), ATTESTOR, threshold)
            ]
        ));
        let definition_id = 1u32;
        assert!(Definitions::<Test>::contains_key(
            registry_id,
            definition_id
        ));

        assert_ok!(Provenance::create_process(
            Origin::signed(ATTESTOR),
            registry_id,
            definition_id,
            b"TestProcess".to_vec(),
        ));
        let process_id = 1u32;
        assert!(Processes::<Test>::contains_key(
            (registry_id, definition_id),
            process_id
        ));
        let process = Processes::<Test>::get((registry_id, definition_id), process_id).unwrap();

        assert_eq!(
            process,
            Process {
                name: b"TestProcess".to_vec().try_into().unwrap(),
                status: ProcessStatus::InProgress
            }
        );
        let definition_step_index = 0;
        assert_ok!(Provenance::attest_process_step(
            Origin::signed(ATTESTOR),
            registry_id,
            definition_id,
            process_id,
            definition_step_index,
            vec![Attribute {
                name: b"TestAttribute".to_vec(),
                fact: Fact::Text(b"TestFact".to_vec())
            }]
        ));

        assert!(ProcessSteps::<Test>::contains_key(
            (registry_id, definition_id, process_id),
            definition_step_index
        ));
        let process_step = ProcessSteps::<Test>::get(
            (registry_id, definition_id, process_id),
            definition_step_index,
        )
        .unwrap();
        assert_eq!(
            process_step,
            ProcessStep {
                attributes: vec![Attribute {
                    name: b"TestAttribute".to_vec().try_into().unwrap(),
                    fact: Fact::Text(b"TestFact".to_vec().try_into().unwrap())
                }]
            }
        );
        let process = Processes::<Test>::get((registry_id, definition_id), process_id).unwrap();
        assert_eq!(
            process,
            Process {
                name: b"TestProcess".to_vec().try_into().unwrap(),
                status: ProcessStatus::InProgress
            }
        );

        let definition_step_index = 1;
        assert_ok!(Provenance::attest_process_step(
            Origin::signed(ATTESTOR),
            registry_id,
            definition_id,
            process_id,
            definition_step_index,
            vec![Attribute {
                name: b"TestAttribute_2".to_vec(),
                fact: Fact::Text(b"TestFact_2".to_vec())
            }]
        ));

        assert!(ProcessSteps::<Test>::contains_key(
            (registry_id, definition_id, process_id),
            definition_step_index
        ));
        let process_step = ProcessSteps::<Test>::get(
            (registry_id, definition_id, process_id),
            definition_step_index,
        )
        .unwrap();
        assert_eq!(
            process_step,
            ProcessStep {
                attributes: vec![Attribute {
                    name: b"TestAttribute_2".to_vec().try_into().unwrap(),
                    fact: Fact::Text(b"TestFact_2".to_vec().try_into().unwrap())
                }]
            }
        );
        let process = Processes::<Test>::get((registry_id, definition_id), process_id).unwrap();
        assert_eq!(
            process,
            Process {
                name: b"TestProcess".to_vec().try_into().unwrap(),
                status: ProcessStatus::Completed
            }
        );
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

        let weight = <Test as Config>::WeightInfo::remove_registry();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);

        let weight = <Test as Config>::WeightInfo::create_definition(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::DefinitionStepLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::set_definition_active();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::set_definition_inactive();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::remove_definition(
            <Test as Config>::DefinitionStepLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::update_definition_step();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight =
            <Test as Config>::WeightInfo::create_process(<Test as Config>::NameLimit::get());
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight =
            <Test as Config>::WeightInfo::update_process(<Test as Config>::NameLimit::get());
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::remove_process(
            <Test as Config>::DefinitionStepLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::attest_process_step(
            <Test as Config>::AttributeLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::FactStringLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
    });
}
