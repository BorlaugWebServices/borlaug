//! Tests for the module.
use super::*;
use crate::mock::*;
use frame_support::assert_ok;
use primitives::{
    attestor::Attestor, attribute::Attribute, fact::Fact, template_step::TemplateStep,
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
fn create_template_should_work() {
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

        // 1 creates a template in the registry
        assert_ok!(Provenance::create_template(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                TemplateStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify template was created
        assert_eq!(Templates::<Test>::contains_key(1u32, 1u32), true);
    });
}

#[test]
fn remove_template_should_work() {
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

        // 1 creates a template in the registry
        assert_ok!(Provenance::create_template(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                TemplateStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify template was created
        assert_eq!(Templates::<Test>::contains_key(1u32, 1u32), true);
        // remove template
        assert_ok!(Provenance::remove_template(Origin::signed(1), 1u32, 1u32));
        // verify template was removed
        assert_eq!(Templates::<Test>::contains_key(1u32, 1u32), false);
    });
}

#[test]
fn update_template_step_should_work() {
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

        // 1 creates a template in the registry
        assert_ok!(Provenance::create_template(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                TemplateStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify template was created
        assert_eq!(Templates::<Test>::contains_key(1u32, 1u32), true);

        // 1 creates a DID for itself
        assert_ok!(Identity::register_did(Origin::signed(1), None));
        let dids = Identity::dids(&1);
        let attestor_did_2 = dids[1];

        // update template step
        assert_ok!(Provenance::update_template_step(
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
fn create_sequence_should_work() {
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

        // 1 creates a template in the registry
        assert_ok!(Provenance::create_template(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                TemplateStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify template was created
        assert_eq!(Templates::<Test>::contains_key(1u32, 1u32), true);

        // 1 creates a sequence
        assert_ok!(Provenance::create_sequence(
            Origin::signed(1),
            attestor_did_1,
            1u32,
            1u32,
            b"Test".to_vec()
        ));
        // verify sequence was created
        assert_eq!(Sequences::<Test>::contains_key((1u32, 1u32), 1u32), true);
    });
}

#[test]
fn remove_sequence_should_work() {
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

        // 1 creates a template in the registry
        assert_ok!(Provenance::create_template(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                TemplateStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify template was created
        assert_eq!(Templates::<Test>::contains_key(1u32, 1u32), true);

        // 1 creates a sequence
        assert_ok!(Provenance::create_sequence(
            Origin::signed(1),
            attestor_did_1,
            1u32,
            1u32,
            b"Test".to_vec()
        ));
        // verify sequence was created
        assert_eq!(Sequences::<Test>::contains_key((1u32, 1u32), 1u32), true);
        // 1 creates a sequence
        assert_ok!(Provenance::remove_sequence(
            Origin::signed(1),
            1u32,
            1u32,
            1u32
        ));
        // verify sequence was removed
        assert_eq!(Sequences::<Test>::contains_key((1u32, 1u32), 1u32), false);
    });
}

#[test]
fn create_sequence_step_should_work() {
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

        // 1 creates a template in the registry
        assert_ok!(Provenance::create_template(
            Origin::signed(1),
            1u32,
            b"Test".to_vec(),
            vec![(
                TemplateStep {
                    name: b"Test".to_vec(),
                },
                vec![Attestor {
                    did: attestor_did_1,
                    short_name: b"Test".to_vec(),
                }]
            )]
        ));
        // verify template was created
        assert_eq!(Templates::<Test>::contains_key(1u32, 1u32), true);

        // 1 creates a sequence
        assert_ok!(Provenance::create_sequence(
            Origin::signed(1),
            attestor_did_1,
            1u32,
            1u32,
            b"Test".to_vec()
        ));
        // verify sequence was created
        assert_eq!(Sequences::<Test>::contains_key((1u32, 1u32), 1u32), true);

        // 1 creates a sequence step
        assert_ok!(Provenance::create_sequence_step(
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

        // verify sequence step was created
        assert_eq!(
            SequenceSteps::<Test>::contains_key((1u32, 1u32, 1u32), 0),
            true
        );
    });
}
