#![cfg_attr(not(feature = "std"), no_std)]

/// Types that implement the AccountSet trait are able to supply a set of accounts
/// The trait is generic over the notion of Account used.
pub trait GetExtrinsicExtra {
    type ModuleIndex;
    type ExtrinsicIndex;
    type Balance;

    fn get_extrinsic_extra(
        module_index: &Self::ModuleIndex,
        extrinsic_index: &Self::ExtrinsicIndex,
    ) -> Self::Balance;
}
