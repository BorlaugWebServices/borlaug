use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub struct Audit<AccountId, ProposalId> {
    /// The proposal that created the audit.
    pub proposal_id: ProposalId,
    /// the status of the audit.
    pub status: AuditStatus,
    /// the audit creator can create/delete and audit. They assign and auditing_org.
    pub audit_creator: AccountId,
    /// the auditing org group is responsible for accepting/rejecting/completing an audit and they assign the auditors.
    pub auditing_org: AccountId,
    /// the auditors sub_group is responsible for creating observations/evidence and linking observations to evidence.
    pub auditors: Option<AccountId>,
}

#[derive(
    Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub enum AuditStatus {
    Requested,
    Accepted,
    Rejected,
    InProgress,
    Completed,
}
