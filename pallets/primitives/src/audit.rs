use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Audit<AuditId> {
    pub audit_id: AuditId,
    pub status: AuditStatus,
    pub auditor: Vec<u8>,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum AuditStatus {
    Requested,
    Accepted,
    Rejected,
    InProgress,
    Completed,
}

impl Default for AuditStatus {
    fn default() -> Self {
        AuditStatus::Requested
    }
}
