#![cfg_attr(not(feature = "std"), no_std)]

/// Types that implement the AccountSet trait are able to supply a set of accounts
/// The trait is generic over the notion of Account used.
pub trait Membership {
    type AccountId;
    type GroupId;

    fn is_member(groupd_id: Self::GroupId, account_id: Self::AccountId) -> bool;
}
