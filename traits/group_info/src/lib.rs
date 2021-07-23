#![cfg_attr(not(feature = "std"), no_std)]

/// Types that implement the GroupInfo trait are able to check if an account is a member of a group
pub trait GroupInfo {
    type AccountId;
    type GroupId;

    fn is_member(groupd_id: Self::GroupId, account_id: &Self::AccountId) -> bool;
    fn is_group_account(groupd_id: Self::GroupId, account_id: &Self::AccountId) -> bool;
}
