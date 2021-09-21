//! Tests for the module.

use crate::{mock::*, GroupMembers};
use core::convert::TryInto;
use frame_support::{assert_ok, codec::Encode};
use primitives::*;

#[test]
fn creating_new_group_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        let caller = 1u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            1_000_000_000u128
        ));

        let group_id = 1u32;

        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));
        let group = super::Groups::<Test>::get(group_id).unwrap();

        assert_eq!(
            Group {
                name: b"Test".to_vec().try_into().unwrap(),
                total_vote_weight: 1u32,
                threshold: 1u32,
                anonymous_account: group.anonymous_account.clone(),
                parent: None,
            },
            group
        );

        assert!(GroupMembers::<Test>::contains_key(group_id, 1));
    });
}

#[test]
fn update_group_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        let caller = 1u64;
        let member_2 = 2u64;
        let member_3 = 3u64;

        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1), (member_2, 1)],
            1u32,
            1_000_000_000u128
        ));

        let group_id = 1u32;

        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(group_id), true);

        assert_ok!(Groups::propose(
            Origin::signed(caller),
            1,
            Box::new(crate::mock::Call::Groups(super::Call::update_group(
                group_id,
                Some("Test_2".to_string().into()),
                Some(vec![(member_3, 1)]),
                Some(vec![member_2]),
                Some(2)
            ))),
            1,
            100
        ));

        assert_eq!(super::Groups::<Test>::contains_key(group_id), true);
        let group = super::Groups::<Test>::get(group_id).unwrap();

        assert_eq!(
            Group {
                name: b"Test_2".to_vec().try_into().unwrap(),
                total_vote_weight: 2u32,
                threshold: 2u32,
                anonymous_account: group.anonymous_account.clone(),
                parent: None,
            },
            group
        );

        assert!(GroupMembers::<Test>::contains_key(group_id, caller));
        assert!(!GroupMembers::<Test>::contains_key(group_id, member_2));
        assert!(GroupMembers::<Test>::contains_key(group_id, member_3));
    });
}

#[test]
fn creating_new_sub_group_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        let caller = 1u64;

        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            1_000_000_000u128
        ));

        let group_id = 1u32;

        // verify group was created
        assert_eq!(super::Groups::<Test>::contains_key(group_id), true);

        let member_2 = 2u64;
        let member_3 = 3u64;

        assert_ok!(Groups::propose(
            Origin::signed(caller),
            1,
            Box::new(crate::mock::Call::Groups(super::Call::create_sub_group(
                "Test".to_string().into(),
                vec![(member_2, 1), (member_3, 1)],
                2,
                1_000_000_000u128
            ))),
            1,
            100
        ));

        let sub_group_id = 2u32;

        assert_eq!(
            super::GroupChildren::<Test>::contains_key(group_id, sub_group_id),
            true
        );

        assert_eq!(super::Groups::<Test>::contains_key(sub_group_id), true);
        let sub_group = super::Groups::<Test>::get(sub_group_id).unwrap();

        assert_eq!(
            Group {
                name: b"Test".to_vec().try_into().unwrap(),
                total_vote_weight: 2u32,
                threshold: 2u32,
                anonymous_account: sub_group.anonymous_account,
                parent: Some(group_id),
            },
            sub_group
        );

        assert_eq!(
            crate::mock::Balances::free_balance(&sub_group.anonymous_account),
            1_000_000_000u128 + 1u128 //minimum_balance(),
        );

        assert!(GroupMembers::<Test>::contains_key(sub_group_id, member_2));
        assert!(GroupMembers::<Test>::contains_key(sub_group_id, member_3));
        assert_eq!(GroupMembers::<Test>::iter_prefix(sub_group_id).count(), 2);
    });
}

#[test]
fn remove_group_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        let caller = 1u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            1_000_000_000u128
        ));
        let group_id = 1u32;
        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));
        let group = super::Groups::<Test>::get(group_id).unwrap();

        assert_ok!(Groups::propose(
            Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::remove_group(
                group_id, caller
            ))),
            1,
            100
        ));

        // verify group was removed
        assert!(!super::Groups::<Test>::contains_key(1u32));

        // verify funds were returned
        assert_eq!(
            crate::mock::Balances::free_balance(&group.anonymous_account),
            0u32.into()
        );
        assert_eq!(
            crate::mock::Balances::free_balance(&caller),
            2_000_000_000u128.into()
        );
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
            vec![(1, 1), (2, 1), (3, 1)],
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
