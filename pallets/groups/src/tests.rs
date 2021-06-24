//! Tests for the module.
use crate::mock::*;
use frame_support::{assert_ok, codec::Encode};

#[test]
fn creating_new_group_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), true);
    });
}

#[test]
fn update_group_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), true);

        // 1 creates a Group
        assert_ok!(Groups::as_group(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Groups(super::Call::update_group(
                1u32,
                Some("Test_2".to_string().into()),
                Some(vec![3, 2]),
                Some(2)
            ))),
        ));
    });
}

#[test]
fn remove_group_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));
        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), true);

        // Remove 1 Group
        assert_ok!(Groups::as_group(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Groups(super::Call::remove_group(
                1
            ))),
        ));
        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), false);
    });
}

#[test]
fn creating_new_sub_group_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));

        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), true);

        // 1 creates a Group
        assert_ok!(Groups::as_group(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Groups(super::Call::create_sub_group(
                1u32,
                "Test".to_string().into(),
                vec![2, 3],
                2
            ))),
        ));
    });
}

fn make_proposal(value: u64) -> Call {
    Call::System(frame_system::Call::remark(value.encode()))
}

#[test]
fn make_propose_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1, 2, 3],
            3
        ));
        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), true);

        let proposal = make_proposal(42);
        // Create Propose
        assert_ok!(Groups::propose(
            Origin::signed(3),
            1,
            Box::new(proposal.clone()),
        ));
        // verify proposal was created
        assert_eq!(super::Proposals::<Test>::contains_key(1u32, 1u32), true);
    });
}

#[test]
fn vote_in_a_group_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1, 2, 3],
            3
        ));
        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), true);

        let proposal = make_proposal(42);
        // Create Propose
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(proposal.clone()),
        ));
        // verify proposal was created
        assert_eq!(super::Proposals::<Test>::contains_key(1u32, 1u32), true);

        // Making vote by 2nd member
        assert_ok!(Groups::vote(
            Origin::signed(2),
            1,
            1,
            true
        ));

        // Making vote by 3rd member
        assert_ok!(Groups::vote(
            Origin::signed(3),
            1,
            1,
            true
        ));
    });
}
