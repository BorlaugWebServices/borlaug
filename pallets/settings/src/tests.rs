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
    new_test_ext().execute_with(|| {});
}
