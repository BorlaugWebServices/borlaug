//! Low-level types used throughout the code.

use frame_support::parameter_types;
use frame_support::BoundedVec;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiSignature, OpaqueExtrinsic,
};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Timestamp = u64;
//groups
pub type GroupId = u32;
pub type ProposalId = u32;
pub type MemberCount = u32;
//identity
pub type CatalogId = u32;
pub type ClaimId = u32;
//provenance
pub type RegistryId = u32;
pub type DefinitionId = u32;
pub type DefinitionStepIndex = u32;
pub type ProcessId = u32;
//audits
pub type AuditId = u32;
pub type ControlPointId = u32;
pub type EvidenceId = u32;
pub type ObservationId = u32;
//asset_registry
pub type AssetId = u32;
pub type LeaseId = u32;

parameter_types! {
    pub const NameLimit: u32 = 100;
    pub const FactStringLimit: u32 = 100;

}

pub type BoundedStringName = BoundedVec<u8, NameLimit>;
pub type BoundedStringFact = BoundedVec<u8, FactStringLimit>;
// #[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
// pub enum Module {
//     Identity,
//     Provenance,
//     Groups,
//     AssetRegistry,
//     Audits,
// }

/// Groups=1
/// Identity=2
/// Provenance=3
/// AssetRegistry=4
/// Audits=5
pub type ModuleIndex = u8;
pub type ExtrinsicIndex = u8;

/// Digest item type.
pub type DigestItem = generic::DigestItem;
/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;
