#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use sp_runtime::Perbill;

sp_api::decl_runtime_apis! {
    pub trait SettingsApi<ModuleIndex,ExtrinsicIndex,Balance>
    where
    ModuleIndex: Codec,
    ExtrinsicIndex: Codec,
    Balance: Codec,
     {

        fn get_weight_to_fee_coefficients() -> Vec<(u64, Perbill, bool, u8)> ;

        fn get_transaction_byte_fee() -> Balance;

        fn get_fee_split_ratio() -> u32;

        fn get_extrinsic_extra(module_index:ModuleIndex,extrinsic_index:ExtrinsicIndex) -> Option<Balance>;

        fn get_extrinsic_extras() -> Vec<(ModuleIndex,Vec<(ExtrinsicIndex,Balance)>)>;

    }
}
