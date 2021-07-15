use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Definition<BoundedString> {
    pub name: BoundedString,
    pub status: DefinitionStatus,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum DefinitionStatus {
    Creating,
    Active,
    Inactive,
}
