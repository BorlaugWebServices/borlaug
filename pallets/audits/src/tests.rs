//! Tests for the module.

use super::*;
use crate::mock::*;
use core::convert::TryInto;
use frame_support::assert_ok;
use primitives::*;
use sp_core::blake2_256;

#[test]
fn create_audit_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator = 1;
        let auditor = 2;
        assert_ok!(AuditsModule::create_audit(
            Origin::signed(audit_creator),
            auditor
        ));
        let audit_id = 1u32;
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.audit_creator, audit_creator);
        assert_eq!(audit.auditor, auditor);
        assert_eq!(audit.status, AuditStatus::Requested);
        assert!(<AuditsByCreator<Test>>::contains_key(
            audit.audit_creator,
            audit_id
        ));
        assert!(<AuditsByAuditor<Test>>::contains_key(
            audit.auditor,
            audit_id
        ));
    });
}
#[test]
fn delete_audit_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator = 1;
        let auditor = 2;
        assert_ok!(AuditsModule::create_audit(
            Origin::signed(audit_creator),
            auditor
        ));
        let audit_id = 1u32;
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.audit_creator, audit_creator);
        assert_eq!(audit.auditor, auditor);
        assert!(<AuditsByCreator<Test>>::contains_key(
            audit.audit_creator,
            audit_id
        ));
        assert!(<AuditsByAuditor<Test>>::contains_key(
            audit.auditor,
            audit_id
        ));
        assert_ok!(AuditsModule::delete_audit(
            Origin::signed(audit_creator),
            audit_id
        ));
        assert!(!<Audits<Test>>::contains_key(audit_id));
        assert!(!<AuditsByCreator<Test>>::contains_key(
            audit_creator,
            audit_id
        ));
        assert!(!<AuditsByAuditor<Test>>::contains_key(auditor, audit_id));
    });
}

#[test]
fn accept_audit_should_work() {
    new_test_ext().execute_with(|| {
        //create audit
        let audit_creator = 1;
        let auditor = 2;
        assert_ok!(AuditsModule::create_audit(
            Origin::signed(audit_creator),
            auditor
        ));
        let audit_id = 1u32;
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.audit_creator, audit_creator);
        assert_eq!(audit.auditor, auditor);
        assert_eq!(audit.status, AuditStatus::Requested);
        //accept audit
        assert_ok!(AuditsModule::accept_audit(
            Origin::signed(auditor),
            audit_id
        ));
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::Accepted);
    });
}
#[test]
fn reject_audit_should_work() {
    new_test_ext().execute_with(|| {
        //create audit
        let audit_creator = 1;
        let auditor = 2;
        assert_ok!(AuditsModule::create_audit(
            Origin::signed(audit_creator),
            auditor
        ));
        let audit_id = 1u32;
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.audit_creator, audit_creator);
        assert_eq!(audit.auditor, auditor);
        assert_eq!(audit.status, AuditStatus::Requested);
        //reject audit
        assert_ok!(AuditsModule::reject_audit(
            Origin::signed(auditor),
            audit_id
        ));
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::Rejected);
    });
}
#[test]
fn complete_audit_should_work() {
    new_test_ext().execute_with(|| {
        //create audit
        let audit_creator = 1;
        let auditor = 2;
        assert_ok!(AuditsModule::create_audit(
            Origin::signed(audit_creator),
            auditor
        ));
        let audit_id = 1u32;
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.audit_creator, audit_creator);
        assert_eq!(audit.auditor, auditor);
        assert_eq!(audit.status, AuditStatus::Requested);
        //accept audit
        assert_ok!(AuditsModule::accept_audit(
            Origin::signed(auditor),
            audit_id
        ));
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::Accepted);
        //create observation
        let observation = Observation {
            compliance: Some(Compliance::Compliant),
            procedural_note: Some(blake2_256(b"test note")),
        };
        let control_point_id = 1;
        assert_ok!(AuditsModule::create_observation(
            Origin::signed(auditor),
            audit_id,
            control_point_id,
            observation,
        ));
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::InProgress);
        //complete audit
        assert_ok!(AuditsModule::complete_audit(
            Origin::signed(auditor),
            audit_id,
        ));
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::Completed);
    });
}

#[test]
fn creating_observation_should_work() {
    new_test_ext().execute_with(|| {
        //create audit
        let audit_creator = 1;
        let auditor = 2;
        assert_ok!(AuditsModule::create_audit(
            Origin::signed(audit_creator),
            auditor
        ));
        let audit_id = 1u32;
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.audit_creator, audit_creator);
        assert_eq!(audit.auditor, auditor);
        assert_eq!(audit.status, AuditStatus::Requested);
        //accept audit
        assert_ok!(AuditsModule::accept_audit(
            Origin::signed(auditor),
            audit_id
        ));
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::Accepted);
        //create observation
        let observation = Observation {
            compliance: Some(Compliance::Compliant),
            procedural_note: Some(blake2_256(b"test note")),
        };
        let control_point_id = 1;
        assert_ok!(AuditsModule::create_observation(
            Origin::signed(auditor),
            audit_id,
            control_point_id,
            observation,
        ));
        //check audit status updated
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::InProgress);
        //check observation exists
        let observation_id = 1;
        let observation =
            Observations::<Test>::get((&audit_id, &control_point_id), &observation_id);
        assert!(observation.is_some());
        let observation = observation.unwrap();
        assert_eq!(
            observation,
            Observation {
                compliance: Some(Compliance::Compliant),
                procedural_note: Some(blake2_256(b"test note")),
            }
        );
    });
}

#[test]
fn creating_evidence_should_work() {
    new_test_ext().execute_with(|| {
        //create audit
        let audit_creator = 1;
        let auditor = 2;
        assert_ok!(AuditsModule::create_audit(
            Origin::signed(audit_creator),
            auditor
        ));
        let audit_id = 1u32;
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.audit_creator, audit_creator);
        assert_eq!(audit.auditor, auditor);
        assert_eq!(audit.status, AuditStatus::Requested);
        //accept audit
        assert_ok!(AuditsModule::accept_audit(
            Origin::signed(auditor),
            audit_id
        ));
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::Accepted);
        //create evidence
        let evidence = Evidence {
            name: b"name".to_vec(),
            content_type: b"image/png".to_vec(),
            url: Some(b"url".to_vec()),
            hash: b"hash".to_vec(),
        };
        assert_ok!(AuditsModule::create_evidence(
            Origin::signed(2),
            audit_id,
            evidence,
        ));
        let evidence_id = 1;
        let evidence = Evidences::<Test>::get(&audit_id, &evidence_id);
        assert!(evidence.is_some());
        let evidence = evidence.unwrap();
        assert_eq!(
            evidence,
            Evidence {
                name: b"name".to_vec().try_into().unwrap(),
                content_type: b"image/png".to_vec().try_into().unwrap(),
                url: Some(b"url".to_vec().try_into().unwrap()),
                hash: b"hash".to_vec().try_into().unwrap(),
            }
        );
    });
}

#[test]
fn link_evidence_should_work() {
    new_test_ext().execute_with(|| {
        //create audit
        let audit_creator = 1;
        let auditor = 2;
        assert_ok!(AuditsModule::create_audit(
            Origin::signed(audit_creator),
            auditor
        ));
        let audit_id = 1u32;
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.audit_creator, audit_creator);
        assert_eq!(audit.auditor, auditor);
        assert_eq!(audit.status, AuditStatus::Requested);
        //accept audit
        assert_ok!(AuditsModule::accept_audit(
            Origin::signed(auditor),
            audit_id
        ));
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::Accepted);
        //create observation
        let observation = Observation {
            compliance: Some(Compliance::Compliant),
            procedural_note: Some(blake2_256(b"test note")),
        };
        let control_point_id = 1;
        assert_ok!(AuditsModule::create_observation(
            Origin::signed(auditor),
            audit_id,
            control_point_id,
            observation,
        ));
        //check audit status updated
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::InProgress);
        //check observation exists
        let observation_id = 1;
        let observation =
            Observations::<Test>::get((&audit_id, &control_point_id), &observation_id);
        assert!(observation.is_some());
        let observation = observation.unwrap();
        assert_eq!(
            observation,
            Observation {
                compliance: Some(Compliance::Compliant),
                procedural_note: Some(blake2_256(b"test note")),
            }
        );
        //create evidence
        let evidence = Evidence {
            name: b"name".to_vec(),
            content_type: b"image/png".to_vec(),
            url: Some(b"url".to_vec()),
            hash: b"hash".to_vec(),
        };
        assert_ok!(AuditsModule::create_evidence(
            Origin::signed(2),
            audit_id,
            evidence,
        ));
        let evidence_id = 1;

        let evidence = Evidences::<Test>::get(&audit_id, &evidence_id);
        assert!(evidence.is_some());
        let evidence = evidence.unwrap();
        assert_eq!(
            evidence,
            Evidence {
                name: b"name".to_vec().try_into().unwrap(),
                content_type: b"image/png".to_vec().try_into().unwrap(),
                url: Some(b"url".to_vec().try_into().unwrap()),
                hash: b"hash".to_vec().try_into().unwrap(),
            }
        );
        //link evidence
        assert_ok!(AuditsModule::link_evidence(
            Origin::signed(2),
            audit_id,
            control_point_id,
            observation_id,
            evidence_id
        ));
        assert!(EvidenceLinks::<Test>::contains_key(
            evidence_id,
            observation_id
        ));
    });
}

#[test]
fn unlink_evidence_should_work() {
    new_test_ext().execute_with(|| {
        //create audit
        let audit_creator = 1;
        let auditor = 2;
        assert_ok!(AuditsModule::create_audit(
            Origin::signed(audit_creator),
            auditor
        ));
        let audit_id = 1u32;
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.audit_creator, audit_creator);
        assert_eq!(audit.auditor, auditor);
        assert_eq!(audit.status, AuditStatus::Requested);
        //accept audit
        assert_ok!(AuditsModule::accept_audit(
            Origin::signed(auditor),
            audit_id
        ));
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::Accepted);
        //create observation
        let observation = Observation {
            compliance: Some(Compliance::Compliant),
            procedural_note: Some(blake2_256(b"test note")),
        };
        let control_point_id = 1;
        assert_ok!(AuditsModule::create_observation(
            Origin::signed(auditor),
            audit_id,
            control_point_id,
            observation,
        ));
        //check audit status updated
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::InProgress);
        //check observation exists
        let observation_id = 1;
        let observation =
            Observations::<Test>::get((&audit_id, &control_point_id), &observation_id);
        assert!(observation.is_some());
        let observation = observation.unwrap();
        assert_eq!(
            observation,
            Observation {
                compliance: Some(Compliance::Compliant),
                procedural_note: Some(blake2_256(b"test note")),
            }
        );
        //create evidence
        let evidence = Evidence {
            name: b"name".to_vec(),
            content_type: b"image/png".to_vec(),
            url: Some(b"url".to_vec()),
            hash: b"hash".to_vec(),
        };
        assert_ok!(AuditsModule::create_evidence(
            Origin::signed(2),
            audit_id,
            evidence,
        ));
        let evidence_id = 1;

        let evidence = Evidences::<Test>::get(&audit_id, &evidence_id);
        assert!(evidence.is_some());
        let evidence = evidence.unwrap();
        assert_eq!(
            evidence,
            Evidence {
                name: b"name".to_vec().try_into().unwrap(),
                content_type: b"image/png".to_vec().try_into().unwrap(),
                url: Some(b"url".to_vec().try_into().unwrap()),
                hash: b"hash".to_vec().try_into().unwrap(),
            }
        );
        //link evidence
        assert_ok!(AuditsModule::link_evidence(
            Origin::signed(2),
            audit_id,
            control_point_id,
            observation_id,
            evidence_id
        ));
        assert!(EvidenceLinks::<Test>::contains_key(
            evidence_id,
            observation_id
        ));
        //unlink evidence
        assert_ok!(AuditsModule::unlink_evidence(
            Origin::signed(2),
            audit_id,
            control_point_id,
            observation_id,
            evidence_id
        ));
        assert!(!EvidenceLinks::<Test>::contains_key(
            evidence_id,
            observation_id
        ));
    });
}

#[test]
fn delete_evidence() {
    new_test_ext().execute_with(|| {
        //create audit
        let audit_creator = 1;
        let auditor = 2;
        assert_ok!(AuditsModule::create_audit(
            Origin::signed(audit_creator),
            auditor
        ));
        let audit_id = 1u32;
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.audit_creator, audit_creator);
        assert_eq!(audit.auditor, auditor);
        assert_eq!(audit.status, AuditStatus::Requested);
        //accept audit
        assert_ok!(AuditsModule::accept_audit(
            Origin::signed(auditor),
            audit_id
        ));
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::Accepted);
        //create observation
        let observation = Observation {
            compliance: Some(Compliance::Compliant),
            procedural_note: Some(blake2_256(b"test note")),
        };
        let control_point_id = 1;
        assert_ok!(AuditsModule::create_observation(
            Origin::signed(auditor),
            audit_id,
            control_point_id,
            observation,
        ));
        //check audit status updated
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::InProgress);
        //check observation exists
        let observation_id = 1;
        let observation =
            Observations::<Test>::get((&audit_id, &control_point_id), &observation_id);
        assert!(observation.is_some());
        let observation = observation.unwrap();
        assert_eq!(
            observation,
            Observation {
                compliance: Some(Compliance::Compliant),
                procedural_note: Some(blake2_256(b"test note")),
            }
        );
        //create evidence
        let evidence = Evidence {
            name: b"name".to_vec(),
            content_type: b"image/png".to_vec(),
            url: Some(b"url".to_vec()),
            hash: b"hash".to_vec(),
        };
        assert_ok!(AuditsModule::create_evidence(
            Origin::signed(2),
            audit_id,
            evidence,
        ));
        let evidence_id = 1;

        let evidence = Evidences::<Test>::get(&audit_id, &evidence_id);
        assert!(evidence.is_some());
        let evidence = evidence.unwrap();
        assert_eq!(
            evidence,
            Evidence {
                name: b"name".to_vec().try_into().unwrap(),
                content_type: b"image/png".to_vec().try_into().unwrap(),
                url: Some(b"url".to_vec().try_into().unwrap()),
                hash: b"hash".to_vec().try_into().unwrap(),
            }
        );
        //link evidence
        assert_ok!(AuditsModule::link_evidence(
            Origin::signed(2),
            audit_id,
            control_point_id,
            observation_id,
            evidence_id
        ));
        assert!(EvidenceLinks::<Test>::contains_key(
            evidence_id,
            observation_id
        ));
        //unlink evidence
        assert_ok!(AuditsModule::unlink_evidence(
            Origin::signed(2),
            audit_id,
            control_point_id,
            observation_id,
            evidence_id
        ));
        assert!(!EvidenceLinks::<Test>::contains_key(
            evidence_id,
            observation_id
        ));
        //delete evidence
        assert_ok!(AuditsModule::delete_evidence(
            Origin::signed(2),
            audit_id,
            evidence_id,
        ));
        assert!(!Evidences::<Test>::contains_key(audit_id, evidence_id,));
        assert!(!EvidenceLinks::<Test>::contains_key(
            evidence_id,
            observation_id
        ));
    });
}
