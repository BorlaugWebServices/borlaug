use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum Fact {
    /// true or false
    Bool(bool),
    /// char collection
    Text(Vec<u8>),
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
    Iso8601(u8, u8, u8, u8, u8, u8, Vec<u8>),
}

impl Default for Fact {
    fn default() -> Self {
        Fact::Text("".as_bytes().to_vec())
    }
}
