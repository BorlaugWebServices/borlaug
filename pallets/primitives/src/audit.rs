use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Audit<AccountId> {
    pub status: AuditStatus,
    pub audit_creator: AccountId,
    pub auditor: AccountId,
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
