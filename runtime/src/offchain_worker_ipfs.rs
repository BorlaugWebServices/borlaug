// For better debugging (printout) support
use frame_support::{debug, decl_event, decl_module, decl_storage, dispatch::DispatchResult};
use sp_runtime::transaction_validity::{
    InvalidTransaction, TransactionLongevity, TransactionValidity, ValidTransaction,
};
use sp_std::prelude::*;
use system::ensure_signed;
use system::offchain;

// The key type ID can be any 4-character string
pub const KEY_TYPE: sp_core::crypto::KeyTypeId = sp_core::crypto::KeyTypeId(*b"abcd");

pub mod crypto {
    pub use super::KEY_TYPE;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    app_crypto!(sr25519, KEY_TYPE);
}
pub trait Trait: timestamp::Trait + system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Call: From<Call<Self>>;

    type SubmitSignedTransaction: offchain::SubmitSignedTransaction<Self, <Self as Trait>::Call>;
    type SubmitUnsignedTransaction: offchain::SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;
}

decl_event!(
    pub enum Event<T>
    where
    <T as system::Trait>::AccountId,

    {
        OffChainCallMade(AccountId),
    }
);

decl_storage! {
trait Store for Module<T: Trait> as OffchainPallet {
    /// Incrementing nonce
    pub Nonce get(fn nonce) build(|_| 1u64): u64;
  }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    fn deposit_event() = default;

    // fn offchain_worker(block: T::BlockNumber) {
    //   debug::info!("Hello World.");
    // }

    pub fn onchain_callback(origin, _block: T::BlockNumber, input: Vec<u8>) -> DispatchResult {
      let who = ensure_signed(origin)?;
      debug::info!("{:?}", core::str::from_utf8(&input).unwrap());
      Ok(())
    }

    fn offchain_worker(block: T::BlockNumber) {
      use system::offchain::SubmitSignedTransaction;
      // Here we specify the function to be called back on-chain in next block import.
      let call = Call::onchain_callback(block, b"hello world!".to_vec());
      T::SubmitSignedTransaction::submit_signed(call);
    }
  }

}
