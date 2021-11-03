use crate::Did;
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_core::H256;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum Fact<BoundedString> {
    /// true or false
    Bool(bool),
    /// char collection
    Text(BoundedString),
    /// A file attachment. File itself is not stored on chain. (hash, filename)
    Attachment(H256, BoundedString),
    /// A lat/lng pair each multiplied by 1_000_000
    Location(u32, u32),
    /// A Did
    Did(Did),
    /// A float stored as le bytes
    Float([u8; 8]),
    /// 0 - 255
    U8(u8),
    /// 0 - 65535
    U16(u16),
    /// 0 - 4294967295
    U32(u32),
    /// 0 - 340282366920938463463374607431768211455
    U128(u128),
    /// (Year, Month, Day)
    Date(u16, u8, u8),
    /// (Year, Month, Day, Hour, Minute, Second, Time Zone Offset)
    Iso8601(u16, u8, u8, u8, u8, u8, Vec<u8>),
}
