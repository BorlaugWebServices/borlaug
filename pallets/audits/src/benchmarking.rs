//! Audits pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, EnsureOrigin, Get, UnfilteredDispatchable};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::*;
use sp_runtime::traits::{Bounded, UniqueSaturatedFrom};
use sp_std::{prelude::*, vec};

use crate::Call;
#[allow(unused)]
use crate::Pallet as AuditsPallet;

type BalanceOf<T> =
    <<T as groups::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
//we only use one group for all parties due to a limitation of `successful_origin()`
fn create_group<T: Config>() -> Result<T::AccountId, &'static str> {
    let account_id: T::AccountId = whitelisted_caller();
    T::Currency::make_free_balance_be(&account_id, BalanceOf::<T>::max_value());
    let origin: <T as frame_system::Config>::RuntimeOrigin =
        SystemOrigin::Signed(account_id.clone()).into();
    groups::Pallet::<T>::create_group(
        origin,
        vec![42u8; 2 as usize],
        vec![(account_id.clone(), 1u32.into())],
        1u32.into(),
        1_000_000_000u32.into(),
    )?;
    let group_id = T::GroupId::unique_saturated_from(1u32);
    let group_maybe = groups::Pallet::<T>::groups(group_id);
    assert!(group_maybe.is_some());
    let group = group_maybe.unwrap();
    let group_account = group.anonymous_account;
    Ok(group_account)
}

fn audit_create<T: Config>() -> Result<T::AccountId, &'static str> {
    let group_account = create_group::<T>()?;
    let origin = T::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
    let call = Call::<T>::create_audit {
        auditing_org: group_account.clone(),
        unique_ref: 1u32,
    };
    call.dispatch_bypass_filter(origin)?;
    let audit_id = T::AuditId::unique_saturated_from(1u32);
    assert!(<Audits<T>>::contains_key(audit_id));
    Ok(group_account)
}

fn audit_create_and_assign<T: Config>() -> Result<
    (
        T::AccountId,
        T::AuditId,
        <T as frame_system::Config>::RuntimeOrigin,
    ),
    &'static str,
> {
    let group_account = audit_create::<T>()?;
    let origin = T::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
    let audit_id = T::AuditId::unique_saturated_from(1u32);
    let call = Call::<T>::accept_audit { audit_id };
    call.dispatch_bypass_filter(origin.clone())?;
    let call = Call::<T>::assign_auditors {
        audit_id,
        auditors: group_account.clone(),
    };
    call.dispatch_bypass_filter(origin.clone())?;
    Ok((group_account, audit_id, origin))
}

benchmarks! {
    create_audit {
        let audit_creator= create_group::<T>()?;
        let auditing_org:T::AccountId=account("auditing_org", 1, 1);
        let origin=T::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::create_audit{
            auditing_org: auditing_org.clone(),
            unique_ref: 1u32
        };

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        let group_id=T::GroupId::unique_saturated_from(1u32);
        let group=groups::Pallet::<T>::groups(group_id).unwrap();
        assert_eq!(audit.audit_creator,group.anonymous_account);
        assert_eq!(audit.auditing_org,auditing_org);
        assert!(<AuditsByCreator<T>>::contains_key(audit.audit_creator,audit_id));
        assert!(<AuditsByAuditingOrg<T>>::contains_key(audit.auditing_org,audit_id));
    }

    delete_audit {
        let _ = audit_create::<T>()?;

        let audit_id = T::AuditId::unique_saturated_from(1u32);
        assert!(<Audits<T>>::contains_key(audit_id));
        let audit=<Audits<T>>::get(audit_id).unwrap();

        let origin=T::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::delete_audit{ audit_id};

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(!<Audits<T>>::contains_key(audit_id));
        assert!(!<AuditsByCreator<T>>::contains_key(audit.audit_creator,audit_id));
        assert!(!<AuditsByAuditingOrg<T>>::contains_key(audit.auditing_org,audit_id));
    }


    link_audit {
        let (auditors,parent_audit_id,origin) = audit_create_and_assign::<T>()?;
        let call = Call::<T>::create_audit{auditing_org:auditors.clone(),unique_ref:1u32};
        call.dispatch_bypass_filter(origin.clone())?;
        let child_audit_id = T::AuditId::unique_saturated_from(2u32);
        assert!(<Audits<T>>::contains_key(child_audit_id));
        let call = Call::<T>::accept_audit{audit_id:child_audit_id};
        call.dispatch_bypass_filter(origin.clone())?;
        let call = Call::<T>::assign_auditors{audit_id:child_audit_id, auditors:auditors.clone()};
        call.dispatch_bypass_filter(origin.clone())?;
        let call = Call::<T>::link_audit{parent_audit_id,child_audit_id};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(<LinkedAudits<T>>::contains_key(parent_audit_id, child_audit_id));
    }

    unlink_audit {
        let (auditors,parent_audit_id,origin) = audit_create_and_assign::<T>()?;
        let call = Call::<T>::create_audit{auditing_org:auditors.clone(),unique_ref:1u32};
        call.dispatch_bypass_filter(origin.clone())?;
        let child_audit_id = T::AuditId::unique_saturated_from(2u32);
        assert!(<Audits<T>>::contains_key(child_audit_id));
        let call = Call::<T>::accept_audit{audit_id:child_audit_id};
        call.dispatch_bypass_filter(origin.clone())?;
        let call = Call::<T>::assign_auditors{audit_id:child_audit_id, auditors:auditors.clone()};
        call.dispatch_bypass_filter(origin.clone())?;
        let call = Call::<T>::link_audit{parent_audit_id,child_audit_id};
        call.dispatch_bypass_filter(origin.clone())?;
        assert!(<LinkedAudits<T>>::contains_key(parent_audit_id, child_audit_id));
        let call = Call::<T>::unlink_audit{parent_audit_id,child_audit_id};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(!<LinkedAudits<T>>::contains_key(parent_audit_id, child_audit_id));

    }


    accept_audit {
        let _ = audit_create::<T>()?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        let origin=T::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::accept_audit{audit_id};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.status,AuditStatus::Accepted);
    }

    assign_auditors_initial_assign {
        let _ = audit_create::<T>()?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        let origin=T::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::accept_audit{audit_id};
        call.dispatch_bypass_filter(origin.clone())?;
        let auditors:T::AccountId=account("auditors", 1, 1);
        let call = Call::<T>::assign_auditors{audit_id,  auditors: auditors.clone()};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.auditors,Some(auditors.clone()));
        assert!(<AuditsByAuditors<T>>::contains_key(auditors,audit_id));
    }

    assign_auditors_replace {
        let (auditors,audit_id,origin) = audit_create_and_assign::<T>()?;
        let new_auditors:T::AccountId=account("new_auditors", 1, 1);
        let call = Call::<T>::assign_auditors{audit_id, auditors:new_auditors.clone()};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.auditors,Some(new_auditors.clone()));
        assert!(!<AuditsByAuditors<T>>::contains_key(auditors,audit_id));
        assert!(<AuditsByAuditors<T>>::contains_key(new_auditors,audit_id));
    }

    reject_audit {
        let _ = audit_create::<T>()?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        let origin=T::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::reject_audit{audit_id};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.status,AuditStatus::Rejected);
    }

    complete_audit {
        let (auditors,audit_id,origin) = audit_create_and_assign::<T>()?;
        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);
        let call = Call::<T>::create_observation{audit_id,control_point_id,compliance: Some(Compliance::Compliant),procedural_note_hash:Some([42u8;32])};
        call.dispatch_bypass_filter(origin.clone())?;
        let call = Call::<T>::complete_audit{audit_id};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.status,AuditStatus::Completed);
    }

    create_observation {
        let (auditors,audit_id,origin) = audit_create_and_assign::<T>()?;
        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);
        let call = Call::<T>::create_observation{audit_id,control_point_id,  compliance:Some(Compliance::Compliant),procedural_note_hash:Some([42u8;32])};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let observation_id=T::ObservationId::unique_saturated_from(1u32);
        let stored_observation=<Observations<T>>::get((audit_id,control_point_id),observation_id);
        assert!(stored_observation.is_some());
        let stored_observation=stored_observation.unwrap();
        let proposal_id=T::ProposalId::unique_saturated_from(1u32);
        assert_eq!(stored_observation,Observation{
            proposal_id,
            compliance:Some(Compliance::Compliant),
            procedural_note_hash:Some([42u8;32])
        });
    }

    create_evidence {
        let a in 1 .. <T as Config>::NameLimit::get(); //name
        let b in 1 .. <T as Config>::NameLimit::get(); //content_type
        let c in 1 .. <T as Config>::UrlLimit::get(); //url
        let d in 1 .. <T as Config>::NameLimit::get(); //hash

        let (auditors,audit_id,origin) = audit_create_and_assign::<T>()?;

        let name=vec![42u8;a as usize];
        let content_type=vec![42u8;b as usize];
        let url=Some(vec![42u8;c as usize]);
        let hash=vec![42u8;d as usize];

        let call = Call::<T>::create_evidence{audit_id, name, content_type, url, hash};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let evidence_id=T::EvidenceId::unique_saturated_from(1u32);
        let stored_evidence=<Evidences<T>>::get(audit_id,evidence_id);
        assert!(stored_evidence.is_some());
        let stored_evidence=stored_evidence.unwrap();
        assert_eq!(stored_evidence.name.len() , a as usize );
        assert_eq!(stored_evidence.content_type.len(), b as usize);
        assert_eq!(stored_evidence.url.unwrap().len(), c as usize);
        assert_eq!(stored_evidence.hash.len(), d as usize);
    }

    link_evidence {
        let (auditors,audit_id,origin) = audit_create_and_assign::<T>()?;

        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);
        let name=vec![42u8;<T as Config>::NameLimit::get() as usize];
        let content_type=vec![42u8;<T as Config>::NameLimit::get() as usize];
        let url=Some(vec![42u8;<T as Config>::NameLimit::get() as usize]);
        let hash=vec![42u8;<T as Config>::NameLimit::get() as usize];
        let call = Call::<T>::create_evidence{audit_id, name ,content_type,url,hash};
        call.dispatch_bypass_filter(origin.clone())? ;
        let evidence_id=T::EvidenceId::unique_saturated_from(1u32);
        let call = Call::<T>::create_observation{audit_id,control_point_id, compliance:Some(Compliance::Compliant), procedural_note_hash: Some([42u8;32])};
        call.dispatch_bypass_filter(origin.clone())?;

        let observation_id=T::ObservationId::unique_saturated_from(1u32);
        let call = Call::<T>::link_evidence{audit_id,control_point_id,observation_id,evidence_id};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
        assert!(<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));
    }

    unlink_evidence {
        let (auditors,audit_id,origin) = audit_create_and_assign::<T>()?;

        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);
        let name=vec![42u8;<T as Config>::NameLimit::get() as usize];
        let content_type=vec![42u8;<T as Config>::NameLimit::get() as usize];
        let url=Some(vec![42u8;<T as Config>::NameLimit::get() as usize]);
        let hash=vec![42u8;<T as Config>::NameLimit::get() as usize];
        let call = Call::<T>::create_evidence{audit_id, name ,content_type,url,hash};
        call.dispatch_bypass_filter(origin.clone())?;
        let evidence_id=T::EvidenceId::unique_saturated_from(1u32);
        let call = Call::<T>::create_observation{audit_id,control_point_id,compliance:Some(Compliance::Compliant), procedural_note_hash:Some([42u8;32])};
        call.dispatch_bypass_filter(origin.clone())?;
        let observation_id=T::ObservationId::unique_saturated_from(1u32);
        let call = Call::<T>::link_evidence{audit_id,control_point_id,observation_id,evidence_id};
        call.dispatch_bypass_filter(origin.clone())?;

        assert!(<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
        assert!(<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));

        let call = Call::<T>::unlink_evidence{audit_id,control_point_id,observation_id,evidence_id};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(!<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
        assert!(!<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));
    }

    delete_evidence {
        let a in 1 .. <T as Config>::MaxLinkRemove::get();
        let (auditors,audit_id,origin) = audit_create_and_assign::<T>()?;

        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);
        let name=vec![42u8;<T as Config>::NameLimit::get() as usize];
        let content_type=vec![42u8;<T as Config>::NameLimit::get() as usize];
        let url=Some(vec![42u8;<T as Config>::NameLimit::get() as usize]);
        let hash=vec![42u8;<T as Config>::NameLimit::get() as usize];
        let call = Call::<T>::create_evidence{audit_id, name ,content_type,url,hash};
        call.dispatch_bypass_filter(origin.clone())?;
        let evidence_id=T::EvidenceId::unique_saturated_from(1u32);

        for i in 0..a {
            let call = Call::<T>::create_observation{audit_id,control_point_id, compliance:Some(Compliance::Compliant), procedural_note_hash:Some([42u8;32])};
            call.dispatch_bypass_filter(origin.clone())?;
            let observation_id=T::ObservationId::unique_saturated_from(i+1);
            let call = Call::<T>::link_evidence{audit_id,control_point_id,observation_id,evidence_id};
            call.dispatch_bypass_filter(origin.clone())?;
            assert!(<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
            assert!(<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));
        }
        let call = Call::<T>::delete_evidence{audit_id,evidence_id,link_count: a};
    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        for i in 0..a {
            let observation_id=T::ObservationId::unique_saturated_from(i+1);
            assert!(!<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
            assert!(!<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));
        }
        assert!(!<Evidences<T>>::contains_key(audit_id,evidence_id));
    }
}

impl_benchmark_test_suite!(AuditsPallet, crate::mock::new_test_ext(), crate::mock::Test,);
