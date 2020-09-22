use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Template {
    pub name: Vec<u8>,
    pub status: TemplateStatus,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum TemplateStatus {
    Creating,
    Active,
    InActive,
}

impl Default for TemplateStatus {
    fn default() -> Self {
        TemplateStatus::Creating
    }
}
