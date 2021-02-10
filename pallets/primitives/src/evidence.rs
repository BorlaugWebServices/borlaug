use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use frame_support::dispatch::Vec;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Evidence {   
    pub name: Vec<u8>,
    pub content_type: Vec<u8>,
    pub url: Option<Vec<u8>>,
    pub hash: Vec<u8>
}
