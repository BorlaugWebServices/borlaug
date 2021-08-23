use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Audit<AccountId, ProposalId> {
    ///if the audit was created by a group, then this is the proposal that created it.
    pub proposal_id: Option<ProposalId>,
    ///the status of the audit.
    pub status: AuditStatus,
    ///the audit creator can create/delete and audit. They assign and auditing_org.
    pub audit_creator: AccountId,
    ///the auditing org group is responsible for accepting/rejecting/completing an audit and they assign the auditors.
    pub auditing_org: AccountId,
    ///the auditors sub_group is responsible for creating observations/evidence and linking observations to evidence.
    pub auditors: Option<AccountId>,
}

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum AuditStatus {
    Requested,
    Accepted,
    Rejected,
    InProgress,
    Completed,
}
