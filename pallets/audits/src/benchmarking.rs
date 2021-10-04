//! Audits pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    dispatch::Vec,
    traits::{Currency, Get},
};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::*;
use sp_runtime::traits::{Bounded, UniqueSaturatedFrom};
use sp_std::{prelude::*, vec};

#[allow(unused)]
use crate::Pallet as AuditsPallet;

type BalanceOf<T> =
    <<T as groups::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

fn audit_create<T: Config>(auditing_org: T::AccountId) -> Result<T::AuditId, &'static str> {
    let audit_creator = whitelisted_caller();
    T::Currency::make_free_balance_be(&audit_creator, BalanceOf::<T>::max_value());
    let origin: <T as frame_system::Config>::Origin = SystemOrigin::Signed(audit_creator).into();
    AuditsPallet::<T>::create_audit(origin, auditing_org)?;
    let audit_id = T::AuditId::unique_saturated_from(1u32);
    assert!(<Audits<T>>::contains_key(audit_id));
    Ok(audit_id)
}

fn audit_create_and_assign<T: Config>(auditors: T::AccountId) -> Result<T::AuditId, &'static str> {
    let auditing_org: T::AccountId = whitelisted_caller();
    let audit_id = audit_create::<T>(auditing_org.clone())?;
    let auditing_org_origin: <T as frame_system::Config>::Origin =
        SystemOrigin::Signed(auditing_org).into();
    AuditsPallet::<T>::accept_audit(auditing_org_origin.clone(), audit_id)?;
    AuditsPallet::<T>::assign_auditors(auditing_org_origin, audit_id, auditors.clone())?;
    Ok(audit_id)
}

benchmarks! {
    create_audit {
        let audit_creator = whitelisted_caller();
        T::Currency::make_free_balance_be(&audit_creator, BalanceOf::<T>::max_value());
        let auditing_org:T::AccountId=account("auditing_org", 1, 1);
    }: _(SystemOrigin::Signed(audit_creator.clone()), auditing_org.clone())

    verify {
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.audit_creator,audit_creator);
        assert_eq!(audit.auditing_org,auditing_org);
        assert!(<AuditsByCreator<T>>::contains_key(audit.audit_creator,audit_id));
        assert!(<AuditsByAuditingOrg<T>>::contains_key(audit.auditing_org,audit_id));
    }

    delete_audit {
        let audit_creator = whitelisted_caller();
        T::Currency::make_free_balance_be(&audit_creator, BalanceOf::<T>::max_value());
        let auditing_org:T::AccountId=account("account", 1, 1);
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(audit_creator.clone()).into();
        AuditsPallet::<T>::create_audit(origin,auditing_org.clone())?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        assert!(<Audits<T>>::contains_key(audit_id));

    }: _(SystemOrigin::Signed(audit_creator.clone()), audit_id)

    verify {
        assert!(!<Audits<T>>::contains_key(audit_id));
        assert!(!<AuditsByCreator<T>>::contains_key(audit_creator,audit_id));
        assert!(!<AuditsByAuditingOrg<T>>::contains_key(auditing_org,audit_id));
    }

    accept_audit {
        let auditing_org:T::AccountId=whitelisted_caller();
        let audit_id=audit_create::<T>(auditing_org.clone())?;
    }: _(SystemOrigin::Signed(auditing_org.clone()), audit_id)

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.status,AuditStatus::Accepted);
    }

    assign_auditors_initial_assign {
        let auditing_org:T::AccountId=whitelisted_caller();
        let audit_id=audit_create::<T>(auditing_org.clone())?;
        let origin_auditing_org:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditing_org.clone()).into();
        AuditsPallet::<T>::accept_audit(origin_auditing_org,audit_id)?;
        let auditors:T::AccountId=account("auditors", 1, 1);
    }: assign_auditors(SystemOrigin::Signed(auditing_org.clone()), audit_id, auditors.clone())

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.auditors,Some(auditors.clone()));
        assert!(<AuditsByAuditors<T>>::contains_key(auditors,audit_id));
    }

    assign_auditors_replace {
        let auditing_org:T::AccountId=whitelisted_caller();
        let audit_id=audit_create::<T>(auditing_org.clone())?;
        let origin_auditing_org:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditing_org.clone()).into();
        AuditsPallet::<T>::accept_audit(origin_auditing_org.clone(),audit_id)?;
        let auditors:T::AccountId=account("auditors", 1, 1);
        AuditsPallet::<T>::assign_auditors(origin_auditing_org,audit_id,auditors.clone())?;
        let new_auditors:T::AccountId=account("new_auditors", 1, 1);
    }: assign_auditors(SystemOrigin::Signed(auditing_org.clone()), audit_id, new_auditors.clone())

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.auditors,Some(new_auditors.clone()));
        assert!(!<AuditsByAuditors<T>>::contains_key(auditors,audit_id));
        assert!(<AuditsByAuditors<T>>::contains_key(new_auditors,audit_id));
    }

    reject_audit {
        let auditing_org:T::AccountId=whitelisted_caller();
        let audit_id=audit_create::<T>(auditing_org.clone())?;
    }: _(SystemOrigin::Signed(auditing_org.clone()), audit_id)

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.status,AuditStatus::Rejected);
    }

    complete_audit {
        let auditing_org:T::AccountId=whitelisted_caller();
        let audit_id=audit_create::<T>(auditing_org.clone())?;
        let auditing_org_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditing_org.clone()).into();
        AuditsPallet::<T>::accept_audit(auditing_org_origin.clone(),audit_id)?;
        let auditors:T::AccountId=whitelisted_caller();
        AuditsPallet::<T>::assign_auditors(auditing_org_origin,audit_id,auditors.clone())?;
        let auditors_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditors).into();
        let observation = Observation{
            compliance:Some(Compliance::Compliant),
            procedural_note:Some([42u8;32])
        };
        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);
        AuditsPallet::<T>::create_observation(auditors_origin,audit_id,control_point_id,observation.clone())?;
    }: _(SystemOrigin::Signed(auditing_org.clone()), audit_id)

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.status,AuditStatus::Completed);
    }

    create_observation {
        let auditors:T::AccountId=whitelisted_caller();
        let audit_id=audit_create_and_assign::<T>(auditors.clone())?;

        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);
        let observation = Observation{
            compliance:Some(Compliance::Compliant),
            procedural_note:Some([42u8;32])
        };
    }: _(SystemOrigin::Signed(auditors), audit_id,control_point_id,observation.clone())

    verify {
        let observation_id=T::ObservationId::unique_saturated_from(1u32);
        let stored_observation=<Observations<T>>::get((audit_id,control_point_id),observation_id);
        assert!(stored_observation.is_some());
        let stored_observation=stored_observation.unwrap();
        assert_eq!(stored_observation,observation);
    }

    create_evidence {
        let a in 1 .. <T as Config>::NameLimit::get(); //name
        let b in 1 .. <T as Config>::NameLimit::get(); //content_type
        let c in 1 .. <T as Config>::NameLimit::get(); //url
        let d in 1 .. <T as Config>::NameLimit::get(); //hash

        let auditors:T::AccountId=whitelisted_caller();
        let audit_id=audit_create_and_assign::<T>(auditors.clone())?;


        let  name=vec![42u8;a as usize];
        let  content_type=vec![42u8;b as usize];
        let  url=Some(vec![42u8;c as usize]);
        let   hash=vec![42u8;d as usize];


    }: _(SystemOrigin::Signed(auditors), audit_id, name ,content_type,url,hash)

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
        let auditors:T::AccountId=whitelisted_caller();
        let audit_id=audit_create_and_assign::<T>(auditors.clone())?;
        let auditors_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditors.clone()).into();

        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);
        let  name=vec![42u8;<T as Config>::NameLimit::get() as usize];
            let  content_type=vec![42u8;<T as Config>::NameLimit::get() as usize];
            let  url=Some(vec![42u8;<T as Config>::NameLimit::get() as usize]);
            let   hash=vec![42u8;<T as Config>::NameLimit::get() as usize];

        AuditsPallet::<T>::create_evidence(auditors_origin.clone(),audit_id,name ,content_type,url,hash)?;
        let evidence_id=T::EvidenceId::unique_saturated_from(1u32);
        let observation = Observation{
            compliance:Some(Compliance::Compliant),
            procedural_note:Some([42u8;32])
        };

        AuditsPallet::<T>::create_observation(auditors_origin,audit_id,control_point_id,observation.clone())?;
        let observation_id=T::ObservationId::unique_saturated_from(1u32);

    }: _(SystemOrigin::Signed(auditors), audit_id,control_point_id,observation_id,evidence_id)

    verify {
        assert!(<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
        assert!(<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));
    }

    unlink_evidence {

        let auditors:T::AccountId=whitelisted_caller();
        let audit_id=audit_create_and_assign::<T>(auditors.clone())?;
        let auditors_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditors.clone()).into();

        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);




            let  name=vec![42u8;<T as Config>::NameLimit::get() as usize];
            let  content_type=vec![42u8;<T as Config>::NameLimit::get() as usize];
            let  url=Some(vec![42u8;<T as Config>::NameLimit::get() as usize]);
            let   hash=vec![42u8;<T as Config>::NameLimit::get() as usize];


        AuditsPallet::<T>::create_evidence(auditors_origin.clone(),audit_id,name ,content_type,url,hash)?;
        let evidence_id=T::EvidenceId::unique_saturated_from(1u32);

        let observation = Observation{
            compliance:Some(Compliance::Compliant),
            procedural_note:Some([42u8;32])
        };

        AuditsPallet::<T>::create_observation(auditors_origin.clone(),audit_id,control_point_id,observation.clone())?;
        let observation_id=T::ObservationId::unique_saturated_from(1u32);


        AuditsPallet::<T>::link_evidence(auditors_origin,audit_id,control_point_id,observation_id,evidence_id)?;

        assert!(<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
        assert!(<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));

    }: _(SystemOrigin::Signed(auditors), audit_id,control_point_id,observation_id,evidence_id)

    verify {
        assert!(!<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
        assert!(!<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));
    }

    delete_evidence {

        let a in 1 .. <T as Config>::MaxLinkRemove::get();

        let auditors:T::AccountId=whitelisted_caller();
        let audit_id=audit_create_and_assign::<T>(auditors.clone())?;
        let auditors_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditors.clone()).into();

        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);

        let  name=vec![42u8;<T as Config>::NameLimit::get() as usize];
        let  content_type=vec![42u8;<T as Config>::NameLimit::get() as usize];
        let  url=Some(vec![42u8;<T as Config>::NameLimit::get() as usize]);
        let   hash=vec![42u8;<T as Config>::NameLimit::get() as usize];
        AuditsPallet::<T>::create_evidence(auditors_origin.clone(),audit_id,name ,content_type,url,hash)?;
        let evidence_id=T::EvidenceId::unique_saturated_from(1u32);



        for i in 0..a {

            let observation = Observation{
                compliance:Some(Compliance::Compliant),
                procedural_note:Some([42u8;32])
            };
            AuditsPallet::<T>::create_observation(auditors_origin.clone(),audit_id,control_point_id,observation.clone())?;
            let observation_id=T::ObservationId::unique_saturated_from(i+1);
            AuditsPallet::<T>::link_evidence(auditors_origin.clone(),audit_id,control_point_id,observation_id,evidence_id)?;
            assert!(<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
            assert!(<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));
        }

    }: _(SystemOrigin::Signed(auditors), audit_id,evidence_id,a)

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
