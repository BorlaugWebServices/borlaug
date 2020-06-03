use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Audit<AuditId, AuditCreatorId, AuditorId> {
    pub audit_id: AuditId,
    pub status: AuditStatus,
    pub audit_creator: AuditCreatorId,
    pub auditor: AuditorId,
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
