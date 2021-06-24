#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;

sp_api::decl_runtime_apis! {
    pub trait SettingsApi<ModuleIndex,ExtrinsicIndex,Balance>
    where
    ModuleIndex: Codec,
    ExtrinsicIndex: Codec,
    Balance: Codec,
     {
        fn get_fee_split_ratio() -> u32;

        fn get_extrinsic_extras() -> Vec<(ModuleIndex,Vec<(ExtrinsicIndex,Balance)>)>;

    }
}
