// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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

benchmarks! {
    create_audit {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let auditor:T::AccountId=account("account", 1, 1);

    }: _(SystemOrigin::Signed(caller.clone()), auditor.clone())

    verify {
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.audit_creator,caller);
        assert_eq!(audit.auditor,auditor);
        assert!(<AuditsByCreator<T>>::contains_key(audit.audit_creator,audit_id));
        assert!(<AuditsByAuditor<T>>::contains_key(audit.auditor,audit_id));
    }

    delete_audit {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let auditor:T::AccountId=account("account", 1, 1);
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        AuditsPallet::<T>::create_audit(origin,auditor.clone())?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        assert!(<Audits<T>>::contains_key(audit_id));

    }: _(SystemOrigin::Signed(caller.clone()), audit_id)

    verify {
        assert!(!<Audits<T>>::contains_key(audit_id));
        assert!(!<AuditsByCreator<T>>::contains_key(caller,audit_id));
        assert!(!<AuditsByAuditor<T>>::contains_key(auditor,audit_id));
    }

    accept_audit {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let auditor:T::AccountId=account("account", 1, 1);
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        AuditsPallet::<T>::create_audit(origin,auditor.clone())?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        assert!(<Audits<T>>::contains_key(audit_id));

    }: _(SystemOrigin::Signed(auditor.clone()), audit_id)

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.status,AuditStatus::Accepted);
    }

    reject_audit {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let auditor:T::AccountId=account("account", 1, 1);
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        AuditsPallet::<T>::create_audit(origin,auditor.clone())?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        assert!(<Audits<T>>::contains_key(audit_id));

    }: _(SystemOrigin::Signed(auditor.clone()), audit_id)

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.status,AuditStatus::Rejected);
    }


    complete_audit {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let auditor:T::AccountId=account("account", 1, 1);
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        AuditsPallet::<T>::create_audit(origin,auditor.clone())?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        assert!(<Audits<T>>::contains_key(audit_id));
        let auditor_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditor.clone()).into();
        AuditsPallet::<T>::accept_audit(auditor_origin.clone(),audit_id)?;

        let observation = Observation{
            compliance:Some(Compliance::Compliant),
            procedural_note:Some([42u8;32])
        };
        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);
        AuditsPallet::<T>::create_observation(auditor_origin,audit_id,control_point_id,observation.clone())?;

    }: _(SystemOrigin::Signed(auditor.clone()), audit_id)

    verify {
        let audit=<Audits<T>>::get(audit_id);
        assert!(audit.is_some());
        let audit=audit.unwrap();
        assert_eq!(audit.status,AuditStatus::Completed);
    }

    create_observation {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let auditor:T::AccountId=account("account", 1, 1);
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        AuditsPallet::<T>::create_audit(origin,auditor.clone())?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        assert!(<Audits<T>>::contains_key(audit_id));
        let auditor_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditor.clone()).into();
        AuditsPallet::<T>::accept_audit(auditor_origin.clone(),audit_id)?;

        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);

        let observation = Observation{
            compliance:Some(Compliance::Compliant),
            procedural_note:Some([42u8;32])
        };

    }: _(SystemOrigin::Signed(auditor.clone()), audit_id,control_point_id,observation.clone())

    verify {
        let observation_id=T::ObservationId::unique_saturated_from(1u32);

        let stored_observation=<Observations<T>>::get((audit_id,control_point_id),observation_id);
        assert!(stored_observation.is_some());
        let stored_observation=stored_observation.unwrap();
        assert_eq!(stored_observation,observation);
    }

    create_evidence {
        let a in 1 .. (<T as Config>::NameLimit::get() -1); //name
        let b in 1 .. (<T as Config>::NameLimit::get()-1); //content_type
        let c in 1 .. (<T as Config>::NameLimit::get()-1); //url
        let d in 1 .. (<T as Config>::NameLimit::get()-1); //hash

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let auditor:T::AccountId=account("account", 1, 1);
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        AuditsPallet::<T>::create_audit(origin,auditor.clone())?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        assert!(<Audits<T>>::contains_key(audit_id));
        let auditor_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditor.clone()).into();
        AuditsPallet::<T>::accept_audit(auditor_origin.clone(),audit_id)?;

        let evidence = Evidence{
            name:vec![42u8;a as usize],
            content_type:vec![42u8;b as usize],
            url:Some(vec![42u8;c as usize]),
            hash:vec![42u8;d as usize]
        };

    }: _(SystemOrigin::Signed(auditor.clone()), audit_id,evidence.clone())

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

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let auditor:T::AccountId=account("account", 1, 1);
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        AuditsPallet::<T>::create_audit(origin,auditor.clone())?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        assert!(<Audits<T>>::contains_key(audit_id));
        let auditor_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditor.clone()).into();
        AuditsPallet::<T>::accept_audit(auditor_origin.clone(),audit_id)?;

        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);

        let evidence = Evidence{
            name:vec![42u8;<T as Config>::NameLimit::get() as usize],
            content_type:vec![42u8;<T as Config>::NameLimit::get() as usize],
            url:Some(vec![42u8;<T as Config>::NameLimit::get() as usize]),
            hash:vec![42u8;<T as Config>::NameLimit::get() as usize]
        };

        AuditsPallet::<T>::create_evidence(auditor_origin.clone(),audit_id,evidence.clone())?;

        let evidence_id=T::EvidenceId::unique_saturated_from(1u32);

        let observation = Observation{
            compliance:Some(Compliance::Compliant),
            procedural_note:Some([42u8;32])
        };

        AuditsPallet::<T>::create_observation(auditor_origin,audit_id,control_point_id,observation.clone())?;
        let observation_id=T::ObservationId::unique_saturated_from(1u32);

    }: _(SystemOrigin::Signed(auditor.clone()), audit_id,control_point_id,observation_id,evidence_id)

    verify {
        assert!(<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
        assert!(<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));
    }

    unlink_evidence {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let auditor:T::AccountId=account("account", 1, 1);
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        AuditsPallet::<T>::create_audit(origin,auditor.clone())?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        assert!(<Audits<T>>::contains_key(audit_id));
        let auditor_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditor.clone()).into();
        AuditsPallet::<T>::accept_audit(auditor_origin.clone(),audit_id)?;

        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);

        let evidence = Evidence{
            name:vec![42u8;<T as Config>::NameLimit::get() as usize],
            content_type:vec![42u8;<T as Config>::NameLimit::get() as usize],
            url:Some(vec![42u8;<T as Config>::NameLimit::get() as usize]),
            hash:vec![42u8;<T as Config>::NameLimit::get() as usize]
        };

        AuditsPallet::<T>::create_evidence(auditor_origin.clone(),audit_id,evidence.clone())?;
        let evidence_id=T::EvidenceId::unique_saturated_from(1u32);

        let observation = Observation{
            compliance:Some(Compliance::Compliant),
            procedural_note:Some([42u8;32])
        };

        AuditsPallet::<T>::create_observation(auditor_origin.clone(),audit_id,control_point_id,observation.clone())?;
        let observation_id=T::ObservationId::unique_saturated_from(1u32);


        AuditsPallet::<T>::link_evidence(auditor_origin,audit_id,control_point_id,observation_id,evidence_id)?;

        assert!(<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
        assert!(<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));

    }: _(SystemOrigin::Signed(auditor.clone()), audit_id,control_point_id,observation_id,evidence_id)

    verify {
        assert!(!<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
        assert!(!<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));
    }

    delete_evidence {

        let a in 1 .. (<T as Config>::MaxLinkRemove::get() -1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let auditor:T::AccountId=account("account", 1, 1);
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        AuditsPallet::<T>::create_audit(origin,auditor.clone())?;
        let audit_id=T::AuditId::unique_saturated_from(1u32);
        assert!(<Audits<T>>::contains_key(audit_id));
        let auditor_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(auditor.clone()).into();
        AuditsPallet::<T>::accept_audit(auditor_origin.clone(),audit_id)?;

        let control_point_id=T::ControlPointId::unique_saturated_from(1u32);

        let evidence = Evidence{
            name:vec![42u8;<T as Config>::NameLimit::get() as usize],
            content_type:vec![42u8;<T as Config>::NameLimit::get() as usize],
            url:Some(vec![42u8;<T as Config>::NameLimit::get() as usize]),
            hash:vec![42u8;<T as Config>::NameLimit::get() as usize]
        };
        let evidence_id=T::EvidenceId::unique_saturated_from(1u32);

        for i in 1..a+1 {
            AuditsPallet::<T>::create_evidence(auditor_origin.clone(),audit_id,evidence.clone())?;
            let observation = Observation{
                compliance:Some(Compliance::Compliant),
                procedural_note:Some([42u8;32])
            };
            AuditsPallet::<T>::create_observation(auditor_origin.clone(),audit_id,control_point_id,observation.clone())?;
            let observation_id=T::ObservationId::unique_saturated_from(i);
            AuditsPallet::<T>::link_evidence(auditor_origin.clone(),audit_id,control_point_id,observation_id,evidence_id)?;
            assert!(<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
            assert!(<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));
        }

    }: _(SystemOrigin::Signed(auditor.clone()), audit_id,evidence_id,a)

    verify {
        for i in 1..a+1 {
            let observation_id=T::ObservationId::unique_saturated_from(i);
            assert!(!<EvidenceLinksByEvidence<T>>::contains_key(evidence_id,observation_id));
            assert!(!<EvidenceLinksByObservation<T>>::contains_key(observation_id,evidence_id));
        }
        assert!(!<Evidences<T>>::contains_key(audit_id,evidence_id));
    }
}

impl_benchmark_test_suite!(AuditsPallet, crate::mock::new_test_ext(), crate::mock::Test,);
