#![cfg_attr(not(feature = "std"), no_std)]

/// Types that implement the AccountSet trait are able to supply a set of accounts
/// The trait is generic over the notion of Account used.
pub trait GroupInfo {
    type AccountId;
    type GroupId;

    fn is_member(groupd_id: Self::GroupId, account_id: &Self::AccountId) -> bool;
    fn is_group_account(groupd_id: Self::GroupId, account_id: &Self::AccountId) -> bool;
}
