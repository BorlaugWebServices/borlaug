#![cfg_attr(not(feature = "std"), no_std)]

/// This allows extrinsics to charge a special fee, customizably by the council
pub trait GetExtrinsicExtra {
    type ModuleIndex;
    type ExtrinsicIndex;
    type AccountId;

    fn charge_extrinsic_extra(
        module_index: &Self::ModuleIndex,
        extrinsic_index: &Self::ExtrinsicIndex,
        account: &Self::AccountId,
    );
}
