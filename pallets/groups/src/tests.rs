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
            1,
            1_000_000_000u128
        ));

        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), true);
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
            1,
            1_000_000_000u128
        ));

        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), true);

        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Groups(super::Call::create_sub_group(
                "Test".to_string().into(),
                vec![2, 3],
                2,
                1_000_000_000u128
            ))),
            1,
            100
        ));

        // verify sub group was created
        assert_eq!(super::GroupChildren::<Test>::contains_key(1u32), true);
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
            1,
            1_000_000_000u128
        ));

        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), true);

        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Groups(super::Call::update_group(
                1u32,
                Some("Test_2".to_string().into()),
                Some(vec![3, 2]),
                Some(2)
            ))),
            1,
            100
        ));

        let group = Groups::get_group(1).unwrap();

        // Verify name updated
        // assert_eq!(b"Test_2".to_vec().try_into().unwrap(), group.name);
        // Verify members updated
        assert_eq!(vec![2, 3], group.members);
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
            1,
            1_000_000_000u128
        ));
        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), true);

        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(crate::mock::Call::Groups(super::Call::remove_group(1, 1))),
            1,
            100
        ));

        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), false);
    });
}

fn make_proposal(value: u64) -> Call {
    Call::System(frame_system::Call::remark(value.encode()))
}

#[test]
fn vote_in_a_group_should_work() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1, 2, 3],
            3,
            1_000_000_000u128
        ));
        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(1u32), true);

        let proposal = make_proposal(42);
        // Create Propose
        assert_ok!(Groups::propose(
            Origin::signed(1),
            1,
            Box::new(proposal.clone()),
            3,
            100
        ));

        // Making vote by 2nd member
        assert_ok!(Groups::vote(Origin::signed(2), 1, 1, true));

        // Making vote by 3rd member
        assert_ok!(Groups::vote(Origin::signed(3), 1, 1, true));
    });
}
