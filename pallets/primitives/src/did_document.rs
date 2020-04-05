use crate::DidProperty;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, core::fmt::Debug)]
pub struct DidDocument {
    pub properties: Vec<DidProperty>,
}
