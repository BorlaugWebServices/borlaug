//! Tests for the module.

use super::*;
use crate::{mock::*, GroupMembers};
use core::convert::TryInto;
use frame_support::{assert_ok, dispatch::Weight};
use primitives::*;

const MINIMUM_BALANCE: u128 = 1;

#[test]
fn creating_new_group_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
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
        let caller = 1u64;
        let member_2 = 2u64;
        let member_3 = 3u64;

        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1), (member_2, 1)],
            1u32,
            1_000_000_000u128
        ));

        let group_id = 1u32;

        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::update_group(
                Some(b"Test_2".to_vec().try_into().unwrap()),
                Some(vec![(member_3, 1)]),
                Some(vec![member_2]),
                Some(2)
            ))),
            1,
            100
        ));

        assert!(super::Groups::<Test>::contains_key(group_id));
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
        let caller = 1u64;

        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            1_000_000_000u128
        ));

        let group_id = 1u32;

        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));

        let member_2 = 2u64;
        let member_3 = 3u64;

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
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

        assert!(super::Groups::<Test>::contains_key(sub_group_id));
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
            1_000_000_000u128 + MINIMUM_BALANCE,
        );

        assert!(GroupMembers::<Test>::contains_key(sub_group_id, member_2));
        assert!(GroupMembers::<Test>::contains_key(sub_group_id, member_3));
        assert_eq!(GroupMembers::<Test>::iter_prefix(sub_group_id).count(), 2);
    });
}
#[test]
fn update_sub_group_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;

        // 1 creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            1_000_000_000u128
        ));

        let group_id = 1u32;

        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));

        let member_2 = 2u64;
        let member_3 = 3u64;

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::create_sub_group(
                b"Test".to_vec().try_into().unwrap(),
                vec![(member_2, 1), (member_3, 1)],
                2,
                1_000_000_000u128
            ))),
            1,
            100
        ));

        let sub_group_id = 2u32;

        assert!(super::Groups::<Test>::contains_key(sub_group_id));
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

        let member_4 = 4u64;
        let member_5 = 5u64;

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::update_sub_group(
                sub_group_id,
                Some(b"New Test".to_vec().try_into().unwrap()),
                Some(vec![(member_3, 2), (member_4, 1), (member_5, 1)]),
                Some(vec![member_2]),
                Some(3),
            ))),
            1,
            100
        ));

        assert!(super::Groups::<Test>::contains_key(sub_group_id));
        let sub_group = super::Groups::<Test>::get(sub_group_id).unwrap();

        assert_eq!(
            Group {
                name: b"New Test".to_vec().try_into().unwrap(),
                total_vote_weight: 4u32,
                threshold: 3u32,
                anonymous_account: sub_group.anonymous_account,
                parent: Some(group_id),
            },
            sub_group
        );

        assert!(!GroupMembers::<Test>::contains_key(sub_group_id, member_2));
        assert!(GroupMembers::<Test>::contains_key(sub_group_id, member_3));
        assert_eq!(
            GroupMembers::<Test>::get(sub_group_id, member_3).unwrap(),
            2
        );
        assert!(GroupMembers::<Test>::contains_key(sub_group_id, member_4));
        assert!(GroupMembers::<Test>::contains_key(sub_group_id, member_5));
        assert_eq!(GroupMembers::<Test>::iter_prefix(sub_group_id).count(), 3);
    });
}

#[test]
fn remove_group_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;
        let member_2 = 2u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1), (member_2, 1)],
            1,
            1_000_000u128
        ));
        let group_id = 1u32;
        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));
        let group = super::Groups::<Test>::get(group_id).unwrap();

        // verify group got funds
        assert_eq!(
            crate::mock::Balances::free_balance(&group.anonymous_account),
            1_000_000u128 + MINIMUM_BALANCE
        );
        assert_eq!(
            crate::mock::Balances::free_balance(&caller),
            2_000_000_000u128 - 1_000_000u128 - MINIMUM_BALANCE
        );
        assert!(super::GroupMembers::<Test>::contains_key(group_id, caller));

        // Create a proposal
        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::update_group(
                None, None, None, None,
            ))),
            2,
            100
        ));

        let proposal_id = 1u32;

        assert!(super::Proposals::<Test>::contains_key(
            &group_id,
            &proposal_id
        ));
        assert!(super::ProposalHashes::<Test>::iter_prefix(&group_id)
            .next()
            .is_some());

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::remove_group(
                group_id, caller
            ))),
            1,
            100
        ));

        // verify storage cleaned up
        assert!(!super::Groups::<Test>::contains_key(group_id));
        assert!(!super::GroupMembers::<Test>::contains_key(group_id, caller));
        assert!(!super::GroupMembers::<Test>::contains_key(
            group_id, member_2
        ));
        assert!(super::Proposals::<Test>::iter_prefix(&group_id)
            .next()
            .is_none());
        assert!(super::ProposalHashes::<Test>::iter_prefix(&group_id)
            .next()
            .is_none());

        // verify funds were returned
        assert_eq!(
            crate::mock::Balances::free_balance(&group.anonymous_account),
            0u128
        );
        assert_eq!(
            crate::mock::Balances::free_balance(&caller),
            2_000_000_000u128
        );
    });
}

#[test]
fn remove_sub_group_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            2_000_000u128
        ));
        let group_id = 1u32;
        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));
        let group = super::Groups::<Test>::get(group_id).unwrap();

        let member_2 = 2u64;

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::create_sub_group(
                b"Test".to_vec().try_into().unwrap(),
                vec![(member_2, 1)],
                2,
                1_000_000u128
            ))),
            1,
            100
        ));

        let sub_group_id = 2u32;

        assert!(super::Groups::<Test>::contains_key(sub_group_id));
        let sub_group = super::Groups::<Test>::get(sub_group_id).unwrap();

        // verify subgroup got funds
        assert_eq!(
            crate::mock::Balances::free_balance(&sub_group.anonymous_account),
            1_000_000u128 + MINIMUM_BALANCE
        );
        assert_eq!(
            crate::mock::Balances::free_balance(&group.anonymous_account),
            1_000_000u128
        );
        assert!(super::GroupChildren::<Test>::contains_key(
            group_id,
            sub_group_id
        ));
        assert!(super::GroupMembers::<Test>::contains_key(
            sub_group_id,
            member_2
        ));

        // Create a proposal
        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(member_2),
            sub_group_id,
            Box::new(crate::mock::Call::Groups(super::Call::update_sub_group(
                sub_group_id,
                None,
                None,
                None,
                None,
            ))),
            3,
            100
        ));

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::remove_sub_group(
                sub_group_id
            ))),
            1,
            100
        ));

        // verify storage cleaned up
        assert!(!super::Groups::<Test>::contains_key(sub_group_id));
        assert!(!super::GroupChildren::<Test>::contains_key(
            group_id,
            sub_group_id
        ));
        assert!(!super::GroupMembers::<Test>::contains_key(
            sub_group_id,
            member_2
        ));
        assert!(super::Proposals::<Test>::iter_prefix(&sub_group_id)
            .next()
            .is_none());
        assert!(super::ProposalHashes::<Test>::iter_prefix(&sub_group_id)
            .next()
            .is_none());

        // verify funds were returned
        assert_eq!(
            crate::mock::Balances::free_balance(&sub_group.anonymous_account),
            0u128
        );
        assert_eq!(
            crate::mock::Balances::free_balance(&group.anonymous_account),
            2_000_000u128 + MINIMUM_BALANCE
        );
    });
}

#[test]
fn execute_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            1_000_000_000u128
        ));

        let group_id = 1u32;

        assert_ok!(mock::Groups::execute(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::update_group(
                None, None, None, None,
            ))),
            100
        ));

        //TODO: detect event?
    });
}

#[test]
fn vote_with_close_and_approve_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;
        let member_2 = 2u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1), (member_2, 2)],
            3u32,
            2_000_000u128
        ));
        let group_id = 1u32;
        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));

        // Create a proposal
        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::update_group(
                Some(b"Updated".to_vec()),
                None,
                None,
                None,
            ))),
            2,
            100
        ));

        let proposal_id = 1u32;

        assert!(super::Proposals::<Test>::contains_key(
            &group_id,
            proposal_id
        ));

        let hash_maybe = super::ProposalHashes::<Test>::iter_prefix(&group_id).next();
        assert!(hash_maybe.is_some());
        let (hash, _) = hash_maybe.unwrap();

        // Making vote by 2nd member
        assert_ok!(mock::Groups::vote(
            mock::Origin::signed(member_2),
            group_id,
            proposal_id,
            true
        ));

        //proposal still exists
        assert!(super::Proposals::<Test>::contains_key(
            &group_id,
            proposal_id
        ));
        assert!(super::ProposalHashes::<Test>::contains_key(
            &group_id, &hash
        ));

        //votes exists

        assert!(super::Voting::<Test>::contains_key(&group_id, &proposal_id));
        let voting = super::Voting::<Test>::get(&group_id, &proposal_id).unwrap();
        assert_eq!(voting.ayes.len(), 2);
        assert_eq!(voting.nays.len(), 0);

        let caller_vote_maybe = voting.ayes.iter().find(|(account, _)| account == &caller);
        assert!(caller_vote_maybe.is_some());
        let caller_vote = caller_vote_maybe.unwrap();
        assert_eq!(*caller_vote, (caller, 1u32));
        let member_2_vote_maybe = voting.ayes.iter().find(|(account, _)| account == &member_2);
        assert!(member_2_vote_maybe.is_some());
        let member_2_vote = member_2_vote_maybe.unwrap();
        assert_eq!(*member_2_vote, (member_2, 2u32));

        //close the proposal
        assert_ok!(mock::Groups::close(
            mock::Origin::signed(caller),
            group_id,
            proposal_id,
            1_000_000_000,
            100
        ));

        //proposal storage cleaned up
        assert!(!super::Proposals::<Test>::contains_key(
            &group_id,
            proposal_id
        ));
        assert!(!super::ProposalHashes::<Test>::contains_key(
            &group_id, &hash
        ));

        assert!(super::Groups::<Test>::contains_key(group_id));
        let group = super::Groups::<Test>::get(group_id).unwrap();
        //Check that proposal was executed
        assert_eq!(group.name, b"Updated".to_vec());
    });
}

//TODO: test more variations

#[test]
fn vote_with_close_and_disapprove_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;
        let member_2 = 2u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1), (member_2, 2)],
            3u32,
            2_000_000u128
        ));
        let group_id = 1u32;
        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));

        // Create a proposal
        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::update_group(
                Some(b"Updated".to_vec()),
                None,
                None,
                None,
            ))),
            2,
            100
        ));

        let proposal_id = 1u32;

        assert!(super::Proposals::<Test>::contains_key(
            &group_id,
            proposal_id
        ));

        let hash_maybe = super::ProposalHashes::<Test>::iter_prefix(&group_id).next();
        assert!(hash_maybe.is_some());
        let (hash, _) = hash_maybe.unwrap();

        // Making vote by 2nd member
        assert_ok!(mock::Groups::vote(
            mock::Origin::signed(member_2),
            group_id,
            proposal_id,
            false
        ));

        //proposal still exists
        assert!(super::Proposals::<Test>::contains_key(
            &group_id,
            proposal_id
        ));
        assert!(super::ProposalHashes::<Test>::contains_key(
            &group_id, &hash
        ));

        //votes exists

        assert!(super::Voting::<Test>::contains_key(&group_id, &proposal_id));
        let voting = super::Voting::<Test>::get(&group_id, &proposal_id).unwrap();
        assert_eq!(voting.ayes.len(), 1);
        assert_eq!(voting.nays.len(), 1);

        let caller_vote_maybe = voting.ayes.iter().find(|(account, _)| account == &caller);
        assert!(caller_vote_maybe.is_some());
        let caller_vote = caller_vote_maybe.unwrap();
        assert_eq!(*caller_vote, (caller, 1u32));
        let member_2_vote_maybe = voting.nays.iter().find(|(account, _)| account == &member_2);
        assert!(member_2_vote_maybe.is_some());
        let member_2_vote = member_2_vote_maybe.unwrap();
        assert_eq!(*member_2_vote, (member_2, 2u32));

        //close the proposal
        assert_ok!(mock::Groups::close(
            mock::Origin::signed(caller),
            group_id,
            proposal_id,
            100_000_000,
            100
        ));

        //proposal storage cleaned up
        assert!(!super::Proposals::<Test>::contains_key(
            &group_id,
            proposal_id
        ));
        assert!(!super::ProposalHashes::<Test>::contains_key(
            &group_id, &hash
        ));

        assert!(super::Groups::<Test>::contains_key(group_id));
        let group = super::Groups::<Test>::get(group_id).unwrap();
        //Check that proposal was NOT executed
        assert_ne!(group.name, b"Updated".to_vec());
    });
}

#[test]
fn veto_yes_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            3_000_000u128
        ));
        let group_id = 1u32;
        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));

        let member_2 = 2u64;
        let member_3 = 3u64;

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::create_sub_group(
                "Test".to_string().into(),
                vec![(member_2, 1), (member_3, 1)],
                2,
                2_000_000u128
            ))),
            1,
            100
        ));
        let sub_group_id = 2u32;

        //subgroup proposes creating another subgroup
        let member_4 = 2u64;
        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(member_2),
            sub_group_id,
            Box::new(crate::mock::Call::Groups(super::Call::create_sub_group(
                "SubSubGroup".to_string().into(),
                vec![(member_4, 1)],
                1,
                1_000_000u128
            ))),
            2,
            100
        ));

        let sub_sub_group_id = 3u32;

        let proposal_id = 2u32;

        assert!(super::Proposals::<Test>::contains_key(
            &sub_group_id,
            proposal_id
        ));

        let hash_maybe = super::ProposalHashes::<Test>::iter_prefix(&sub_group_id).next();
        assert!(hash_maybe.is_some());
        let (hash, _) = hash_maybe.unwrap();

        // Caller vetos proposal in the affirmative
        assert_ok!(mock::Groups::veto(
            mock::Origin::signed(caller),
            sub_group_id,
            proposal_id,
            true,
            10_000_000_000,
            100
        ));

        //proposal storage cleaned up
        assert!(!super::Proposals::<Test>::contains_key(
            &group_id,
            proposal_id
        ));
        assert!(!super::ProposalHashes::<Test>::contains_key(
            &group_id, &hash
        ));
        //subsubgroup was created
        assert!(super::Groups::<Test>::contains_key(sub_sub_group_id));
        let sub_sub_group = super::Groups::<Test>::get(group_id).unwrap();
        assert_ne!(sub_sub_group.name, b"SubSubGroup".to_vec());
    });
}

#[test]
fn veto_no_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            3_000_000u128
        ));
        let group_id = 1u32;
        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));

        let member_2 = 2u64;
        let member_3 = 3u64;

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::create_sub_group(
                "Test".to_string().into(),
                vec![(member_2, 1), (member_3, 1)],
                2,
                2_000_000u128
            ))),
            1,
            100
        ));
        let sub_group_id = 2u32;

        //subgroup proposes creating another subgroup
        let member_4 = 2u64;
        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(member_2),
            sub_group_id,
            Box::new(crate::mock::Call::Groups(super::Call::create_sub_group(
                "SubSubGroup".to_string().into(),
                vec![(member_4, 1)],
                1,
                1_000_000u128
            ))),
            2,
            100
        ));

        let sub_sub_group_id = 3u32;

        let proposal_id = 2u32;

        assert!(super::Proposals::<Test>::contains_key(
            &sub_group_id,
            proposal_id
        ));

        let hash_maybe = super::ProposalHashes::<Test>::iter_prefix(&sub_group_id).next();
        assert!(hash_maybe.is_some());
        let (hash, _) = hash_maybe.unwrap();

        // Caller vetos proposal in the affirmative
        assert_ok!(mock::Groups::veto(
            mock::Origin::signed(caller),
            sub_group_id,
            proposal_id,
            false,
            100_000_000,
            100
        ));

        //proposal storage cleaned up
        assert!(!super::Proposals::<Test>::contains_key(
            &group_id,
            proposal_id
        ));
        assert!(!super::ProposalHashes::<Test>::contains_key(
            &group_id, &hash
        ));
        //subsubgroup was NOT created
        assert!(!super::Groups::<Test>::contains_key(sub_sub_group_id));
    });
}

#[test]
fn withdraw_funds_group_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            2_000_000u128
        ));
        let group_id = 1u32;
        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(
                super::Call::withdraw_funds_group(caller, 1_000_000u128)
            )),
            1,
            100
        ));

        let group = super::Groups::<Test>::get(group_id).unwrap();

        // verify funds were returned
        assert_eq!(
            crate::mock::Balances::free_balance(&group.anonymous_account),
            1_000_000u128 + MINIMUM_BALANCE
        );
        assert_eq!(
            crate::mock::Balances::free_balance(&caller),
            2_000_000_000u128 - 1_000_000u128 - MINIMUM_BALANCE
        );
    });
}

#[test]
fn withdraw_funds_sub_group_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            3_000_000u128
        ));
        let group_id = 1u32;
        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));

        let member_2 = 2u64;

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::create_sub_group(
                b"Test".to_vec().try_into().unwrap(),
                vec![(member_2, 1)],
                2,
                2_000_000u128
            ))),
            1,
            100
        ));

        let sub_group_id = 2u32;

        assert!(super::Groups::<Test>::contains_key(sub_group_id));
        let sub_group = super::Groups::<Test>::get(sub_group_id).unwrap();

        let group = super::Groups::<Test>::get(group_id).unwrap();
        // verify subgroup got funds
        assert_eq!(
            crate::mock::Balances::free_balance(&sub_group.anonymous_account),
            2_000_000u128 + MINIMUM_BALANCE
        );
        assert_eq!(
            crate::mock::Balances::free_balance(&group.anonymous_account),
            1_000_000u128
        );

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(
                super::Call::withdraw_funds_sub_group(sub_group_id, 1_000_000u128)
            )),
            1,
            100
        ));

        let group = super::Groups::<Test>::get(group_id).unwrap();

        // verify funds were returned
        assert_eq!(
            crate::mock::Balances::free_balance(&sub_group.anonymous_account),
            1_000_000u128 + MINIMUM_BALANCE
        );
        assert_eq!(
            crate::mock::Balances::free_balance(&group.anonymous_account),
            2_000_000u128
        );
    });
}

#[test]
fn send_funds_to_sub_group_should_work() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;

        // caller creates a Group
        assert_ok!(crate::mock::Groups::create_group(
            mock::Origin::signed(caller),
            b"Test".to_vec(),
            vec![(caller, 1)],
            1u32,
            3_000_000u128
        ));
        let group_id = 1u32;
        // verify group was created
        assert!(super::Groups::<Test>::contains_key(group_id));

        let member_2 = 2u64;

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(super::Call::create_sub_group(
                b"Test".to_vec().try_into().unwrap(),
                vec![(member_2, 1)],
                2,
                1_000_000u128
            ))),
            1,
            100
        ));

        let sub_group_id = 2u32;

        assert!(super::Groups::<Test>::contains_key(sub_group_id));
        let sub_group = super::Groups::<Test>::get(sub_group_id).unwrap();

        let group = super::Groups::<Test>::get(group_id).unwrap();
        // verify subgroup got funds
        assert_eq!(
            crate::mock::Balances::free_balance(&sub_group.anonymous_account),
            1_000_000u128 + MINIMUM_BALANCE
        );
        assert_eq!(
            crate::mock::Balances::free_balance(&group.anonymous_account),
            2_000_000u128
        );

        assert_ok!(mock::Groups::propose(
            mock::Origin::signed(caller),
            group_id,
            Box::new(crate::mock::Call::Groups(
                super::Call::send_funds_to_sub_group(sub_group_id, 1_000_000u128)
            )),
            1,
            100
        ));

        let group = super::Groups::<Test>::get(group_id).unwrap();

        // verify funds were returned
        assert_eq!(
            crate::mock::Balances::free_balance(&sub_group.anonymous_account),
            2_000_000u128 + MINIMUM_BALANCE
        );
        assert_eq!(
            crate::mock::Balances::free_balance(&group.anonymous_account),
            1_000_000u128
        );
    });
}

//Make sure weights cannot exceed 10% of total allowance for block.

#[test]
fn weights_should_not_be_excessive() {
    new_test_ext().execute_with(|| {
        const MAXIMUM_ALLOWED_WEIGHT: Weight = 130_000_000_000;

        let weight = <Test as Config>::WeightInfo::create_group(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::MaxMembers::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::update_group(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::MaxMembers::get(),
            <Test as Config>::MaxMembers::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::create_sub_group(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::MaxMembers::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::update_sub_group(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::MaxMembers::get(),
            <Test as Config>::MaxMembers::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::remove_group(
            <Test as Config>::MaxMembers::get(),
            <Test as Config>::MaxProposals::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::remove_sub_group(
            <Test as Config>::MaxMembers::get(),
            <Test as Config>::MaxProposals::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight =
            <Test as Config>::WeightInfo::execute(<Test as Config>::MaxProposalLength::get());
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::propose_execute(
            <Test as Config>::MaxProposalLength::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::propose_proposed(
            <Test as Config>::MaxProposalLength::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::vote(<Test as Config>::MaxMembers::get());
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight =
            <Test as Config>::WeightInfo::close_disapproved(<Test as Config>::MaxMembers::get());
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::close_approved(
            <Test as Config>::MaxProposalLength::get(),
            <Test as Config>::MaxMembers::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::veto_disapproved();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight =
            <Test as Config>::WeightInfo::veto_approved(<Test as Config>::MaxProposalLength::get());
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::withdraw_funds_group();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::withdraw_funds_sub_group();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::send_funds_to_sub_group();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
    });
}
