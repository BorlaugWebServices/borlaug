#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::*;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait AuditsApi<AccountId,AuditId,ControlPointId,EvidenceId,ObservationId,BoundedStringName>
    where
    AccountId: Codec,
    AuditId: Codec,
    ControlPointId: Codec,
    EvidenceId: Codec,
    ObservationId: Codec,
    BoundedStringName: Codec + Into<Vec<u8>>,
     {
        fn get_audits_by_creator(account: AccountId) -> Vec<(AuditId,Audit<AccountId>)>;

        fn get_audits_by_auditor(account: AccountId) -> Vec<(AuditId,Audit<AccountId>)>;

        fn get_observation(audit_id:AuditId,control_point_id:ControlPointId,observation_id:ObservationId)->Option<Observation>;

        fn get_observation_by_control_point(audit_id:AuditId,control_point_id:ControlPointId)->Vec<(ObservationId,Observation)>;

        fn get_evidence(audit_id:AuditId,evidence_id:EvidenceId)->Option<Evidence<BoundedStringName>>;

        fn get_evidence_by_audit(audit_id:AuditId)->Vec<(EvidenceId,Evidence<BoundedStringName>)>;

        fn get_evidence_links_by_evidence(evidence_id:EvidenceId)->Vec<ObservationId>;

        fn get_evidence_links_by_observation(observation_id:ObservationId)->Vec<EvidenceId>;
    }
}
