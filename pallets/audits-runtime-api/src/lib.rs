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




    }
}
