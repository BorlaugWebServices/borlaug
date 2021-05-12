//! Tests for the module.
use crate::mock::*;
use frame_support::{
    assert_ok,
    codec::{Decode, Encode},
};
use frame_system::{self as system, EventRecord, Phase};
use hex_literal::hex;
use primitives::group::Group;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

// pub type MemberCount = u32;
// /// Origin for the collective module.
// #[derive(PartialEq, Eq, Clone, Encode, Decode)]
// pub enum RawOrigin<AccountId> {
//     /// It has been condoned by a given number of members of the collective from a given total.
//     Members(MemberCount, MemberCount),
//     /// It has been condoned by a single member of the collective.
//     Member(AccountId),
// }

// /// Origin for the collective module.
// pub type Origin<T> = RawOrigin<<T as frame_system::Config>::AccountId>;

#[test]
fn creating_new_group_should_work() {
    new_test_ext().execute_with(|| {
        //required for randomness_collective_flip module
        System::set_block_number(1);

        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));
    });
}

fn make_proposal(value: u64) -> Call {
    Call::System(frame_system::Call::remark(value.encode()))
}

#[test]
fn propose_works() {
    new_test_ext().execute_with(|| {
        // 1 creates a Group
        assert_ok!(Groups::create_group(
            Origin::signed(1),
            "Test".to_string().into(),
            vec![1],
            1
        ));
        let group_id = 1;
        let proposal = make_proposal(42);
        // let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
        // let hash = proposal.blake2_256().into();
        // let end = 4;
        assert_ok!(Groups::propose(
            Origin::signed(1),
            group_id,
            Box::new(proposal.clone())
        ));
        // assert_eq!(Groups::proposals(), vec![hash]);

        // assert_eq!(
        //     System::events(),
        //     vec![EventRecord {
        //         phase: Phase::Initialization,
        //         event: Event::groups(Event::Proposed(
        //             1,
        //             0,
        //             hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"].into(),
        //             3,
        //         )),
        //         topics: vec![],
        //     }]
        // );
    });
}
