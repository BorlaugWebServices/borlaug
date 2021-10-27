//! Tests for the module.

use super::*;
use crate::mock::*;
use core::convert::TryInto;
use frame_support::{assert_ok, dispatch::Weight};
use primitives::*;
use sp_core::blake2_256;

fn create_group(member: u64, group_id: u32) -> u64 {
    assert_ok!(Groups::create_group(
        Origin::signed(member),
        b"Test".to_vec(),
        vec![(member, 1)],
        1,
        10_000,
    ));
    let group_maybe = Groups::get_group(group_id);
    assert!(group_maybe.is_some());
    let (group, _members) = group_maybe.unwrap();
    group.anonymous_account
}

fn create_audit(
    audit_creator_member: u64,
    audit_creator_group_account: u64,
    auditing_org: u64,
) -> u32 {
    assert_ok!(Groups::propose(
        Origin::signed(audit_creator_member),
        1,
        Box::new(crate::mock::Call::AuditsModule(super::Call::create_audit(
            auditing_org
        ))),
        1,
        100
    ));
    let audit_id = 1u32;
    let audit = Audits::<Test>::get(audit_id);
    assert!(audit.is_some());
    let audit = audit.unwrap();
    assert_eq!(audit.audit_creator, audit_creator_group_account);
    assert_eq!(audit.auditing_org, auditing_org);
    assert_eq!(audit.status, AuditStatus::Requested);
    assert!(<AuditsByCreator<Test>>::contains_key(
        audit.audit_creator,
        audit_id
    ));
    assert!(<AuditsByAuditingOrg<Test>>::contains_key(
        audit.auditing_org,
        audit_id
    ));
    audit_id
}

fn accept_audit(auditing_org_group_id: u32, audit_id: u32, auditing_org_member: u64) {
    assert_ok!(Groups::propose(
        Origin::signed(auditing_org_member),
        auditing_org_group_id,
        Box::new(crate::mock::Call::AuditsModule(super::Call::accept_audit(
            audit_id
        ))),
        1,
        100
    ));
    let audit = Audits::<Test>::get(audit_id);
    assert!(audit.is_some());
    let audit = audit.unwrap();
    assert_eq!(audit.status, AuditStatus::Accepted);
}

fn assign_auditors(
    audit_id: u32,
    auditing_org_group_id: u32,
    auditing_org_member: u64,
    auditors_group_account: u64,
) {
    assert_ok!(Groups::propose(
        Origin::signed(auditing_org_member),
        auditing_org_group_id,
        Box::new(crate::mock::Call::AuditsModule(
            super::Call::assign_auditors(audit_id, auditors_group_account)
        )),
        1,
        100
    ));

    let audit = Audits::<Test>::get(audit_id);
    assert!(audit.is_some());
    let audit = audit.unwrap();
    assert!(audit.auditors == Some(auditors_group_account));
    assert!(<AuditsByAuditors<Test>>::contains_key(
        auditors_group_account,
        audit_id
    ));
}

fn get_proposal_id() -> u32 {
    let last_event = frame_system::Pallet::<Test>::events()
        .last()
        .unwrap()
        .clone();
    let local_event = crate::mock::Event::from(last_event.event);
    let proposal_id = match local_event {
        mock::Event::groups(groups::Event::Approved(_, proposal_id, _, _, _, _)) => proposal_id,
        _ => panic!("unexpected event"),
    };
    proposal_id
}

fn create_observation(auditors_member: u64, auditors_group_id: u32, audit_id: u32) {
    let observation = Observation {
        compliance: Some(Compliance::Compliant),
        procedural_note_hash: Some(blake2_256(b"test note")),
    };
    let control_point_id = 1;

    assert_ok!(Groups::propose(
        Origin::signed(auditors_member),
        auditors_group_id,
        Box::new(crate::mock::Call::AuditsModule(
            super::Call::create_observation(audit_id, control_point_id, observation,)
        )),
        1,
        100
    ));

    let proposal_id = get_proposal_id();
    let observation_by_proposal = ObservationByProposal::<Test>::get(&proposal_id);
    assert!(observation_by_proposal.is_some());
    let observation_by_proposal = observation_by_proposal.unwrap();
    assert_eq!(observation_by_proposal.0, audit_id);
    assert_eq!(observation_by_proposal.1, control_point_id);

    //check observation exists
    let observation_id = observation_by_proposal.2;

    let observation = Observations::<Test>::get((&audit_id, &control_point_id), &observation_id);
    assert!(observation.is_some());
    let observation = observation.unwrap();
    assert_eq!(
        observation,
        Observation {
            compliance: Some(Compliance::Compliant),
            procedural_note_hash: Some(blake2_256(b"test note")),
        }
    );
}

fn create_evidence(auditors_member: u64, auditors_group_id: u32, audit_id: u32) {
    assert_ok!(Groups::propose(
        Origin::signed(auditors_member),
        auditors_group_id,
        Box::new(crate::mock::Call::AuditsModule(
            super::Call::create_evidence(
                audit_id,
                b"name".to_vec(),
                b"image/png".to_vec(),
                Some(b"url".to_vec()),
                b"hash".to_vec(),
            )
        )),
        1,
        100
    ));

    let proposal_id = get_proposal_id();

    let evidence_id = 1;
    let evidence = Evidences::<Test>::get(&audit_id, &evidence_id);
    assert!(evidence.is_some());
    let evidence = evidence.unwrap();
    assert_eq!(
        evidence,
        Evidence {
            proposal_id,
            name: b"name".to_vec().try_into().unwrap(),
            content_type: b"image/png".to_vec().try_into().unwrap(),
            url: Some(b"url".to_vec().try_into().unwrap()),
            hash: b"hash".to_vec().try_into().unwrap(),
        }
    );
}

#[test]
fn create_audit_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org = 2;
        create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
    });
}

#[test]
fn delete_audit_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org = 2;
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );

        //delete audit
        assert_ok!(Groups::propose(
            Origin::signed(audit_creator_member),
            audit_creator_group_id,
            Box::new(crate::mock::Call::AuditsModule(super::Call::delete_audit(
                audit_id
            ))),
            1,
            100
        ));
        assert!(!<Audits<Test>>::contains_key(audit_id));
        assert!(!<AuditsByCreator<Test>>::contains_key(
            audit_creator_member,
            audit_id
        ));
        assert!(!<AuditsByAuditingOrg<Test>>::contains_key(
            auditing_org,
            audit_id
        ));
    });
}

#[test]
fn accept_audit_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org_member = 2;
        let auditing_org_group_id = 2;
        let auditing_org = create_group(auditing_org_member, auditing_org_group_id);
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
        //accept audit
        accept_audit(auditing_org_group_id, audit_id, auditing_org_member);
    });
}

#[test]
fn assign_auditors_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org_member = 2;
        let auditing_org_group_id = 2;
        let auditing_org = create_group(auditing_org_member, auditing_org_group_id);
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
        accept_audit(auditing_org_group_id, audit_id, auditing_org_member);
        //assign auditors
        let auditors_group_id = 3;
        let auditors_member = 3;
        let auditors_group_account = create_group(auditors_member, auditors_group_id);
        assign_auditors(
            audit_id,
            auditing_org_group_id,
            auditing_org_member,
            auditors_group_account,
        );
    });
}

#[test]
fn reassign_auditors_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org_member = 2;
        let auditing_org_group_id = 2;
        let auditing_org = create_group(auditing_org_member, auditing_org_group_id);
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
        accept_audit(auditing_org_group_id, audit_id, auditing_org_member);
        //assign auditors
        let auditors_group_id = 3;
        let auditors_member = 3;
        let auditors_group_account = create_group(auditors_member, auditors_group_id);
        assign_auditors(
            audit_id,
            auditing_org_group_id,
            auditing_org_member,
            auditors_group_account,
        );
        //reassign auditors
        let new_auditors_group_id = 3;
        let new_auditors_member = 3;
        let new_auditors_group_account = create_group(new_auditors_member, new_auditors_group_id);
        assign_auditors(
            audit_id,
            auditing_org_group_id,
            auditing_org_member,
            new_auditors_group_account,
        );
    });
}
#[test]
fn reject_audit_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org_member = 2;
        let auditing_org_group_id = 2;
        let auditing_org = create_group(auditing_org_member, auditing_org_group_id);
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
        //reject audit
        assert_ok!(Groups::propose(
            Origin::signed(auditing_org_member),
            auditing_org_group_id,
            Box::new(crate::mock::Call::AuditsModule(super::Call::reject_audit(
                audit_id
            ))),
            1,
            100
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
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org_member = 2;
        let auditing_org_group_id = 2;
        let auditing_org = create_group(auditing_org_member, auditing_org_group_id);
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
        accept_audit(auditing_org_group_id, audit_id, auditing_org_member);
        //assign auditors
        let auditors_group_id = 3;
        let auditors_member = 3;
        let auditors_group_account = create_group(auditors_member, auditors_group_id);
        assign_auditors(
            audit_id,
            auditing_org_group_id,
            auditing_org_member,
            auditors_group_account,
        );
        create_observation(auditors_member, auditors_group_id, audit_id);

        //complete audit

        assert_ok!(Groups::propose(
            Origin::signed(auditing_org_member),
            auditing_org_group_id,
            Box::new(crate::mock::Call::AuditsModule(
                super::Call::complete_audit(audit_id)
            )),
            1,
            100
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
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org_member = 2;
        let auditing_org_group_id = 2;
        let auditing_org = create_group(auditing_org_member, auditing_org_group_id);
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
        accept_audit(auditing_org_group_id, audit_id, auditing_org_member);
        //assign auditors
        let auditors_group_id = 3;
        let auditors_member = 3;
        let auditors_group_account = create_group(auditors_member, auditors_group_id);
        assign_auditors(
            audit_id,
            auditing_org_group_id,
            auditing_org_member,
            auditors_group_account,
        );

        //create observation
        create_observation(auditors_member, auditors_group_id, audit_id);

        //check audit status updated
        let audit = Audits::<Test>::get(audit_id);
        assert!(audit.is_some());
        let audit = audit.unwrap();
        assert_eq!(audit.status, AuditStatus::InProgress);
    });
}

#[test]
fn creating_evidence_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org_member = 2;
        let auditing_org_group_id = 2;
        let auditing_org = create_group(auditing_org_member, auditing_org_group_id);
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
        accept_audit(auditing_org_group_id, audit_id, auditing_org_member);
        //assign auditors
        let auditors_group_id = 3;
        let auditors_member = 3;
        let auditors_group_account = create_group(auditors_member, auditors_group_id);
        assign_auditors(
            audit_id,
            auditing_org_group_id,
            auditing_org_member,
            auditors_group_account,
        );
        //create evidence
        create_evidence(auditors_member, auditors_group_id, audit_id);
    });
}

#[test]
fn link_evidence_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org_member = 2;
        let auditing_org_group_id = 2;
        let auditing_org = create_group(auditing_org_member, auditing_org_group_id);
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
        accept_audit(auditing_org_group_id, audit_id, auditing_org_member);
        //assign auditors
        let auditors_group_id = 3;
        let auditors_member = 3;
        let auditors_group_account = create_group(auditors_member, auditors_group_id);
        assign_auditors(
            audit_id,
            auditing_org_group_id,
            auditing_org_member,
            auditors_group_account,
        );

        create_observation(auditors_member, auditors_group_id, audit_id);
        let observation_id = 1;
        create_evidence(auditors_member, auditors_group_id, audit_id);
        let evidence_id = 1;
        let control_point_id = 1;

        //link evidence
        assert_ok!(Groups::propose(
            Origin::signed(auditors_member),
            auditors_group_id,
            Box::new(crate::mock::Call::AuditsModule(super::Call::link_evidence(
                audit_id,
                control_point_id,
                observation_id,
                evidence_id
            ))),
            1,
            100
        ));
        assert!(EvidenceLinksByEvidence::<Test>::contains_key(
            evidence_id,
            observation_id
        ));
        assert!(EvidenceLinksByObservation::<Test>::contains_key(
            observation_id,
            evidence_id
        ));
    });
}

#[test]
fn unlink_evidence_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org_member = 2;
        let auditing_org_group_id = 2;
        let auditing_org = create_group(auditing_org_member, auditing_org_group_id);
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
        accept_audit(auditing_org_group_id, audit_id, auditing_org_member);
        //assign auditors
        let auditors_group_id = 3;
        let auditors_member = 3;
        let auditors_group_account = create_group(auditors_member, auditors_group_id);
        assign_auditors(
            audit_id,
            auditing_org_group_id,
            auditing_org_member,
            auditors_group_account,
        );

        create_observation(auditors_member, auditors_group_id, audit_id);
        let observation_id = 1;
        create_evidence(auditors_member, auditors_group_id, audit_id);
        let evidence_id = 1;
        let control_point_id = 1;

        //link evidence
        assert_ok!(Groups::propose(
            Origin::signed(auditors_member),
            auditors_group_id,
            Box::new(crate::mock::Call::AuditsModule(super::Call::link_evidence(
                audit_id,
                control_point_id,
                observation_id,
                evidence_id
            ))),
            1,
            100
        ));
        assert!(EvidenceLinksByEvidence::<Test>::contains_key(
            evidence_id,
            observation_id
        ));
        assert!(EvidenceLinksByObservation::<Test>::contains_key(
            observation_id,
            evidence_id
        ));
        //unlink evidence
        assert_ok!(Groups::propose(
            Origin::signed(auditors_member),
            auditors_group_id,
            Box::new(crate::mock::Call::AuditsModule(
                super::Call::unlink_evidence(
                    audit_id,
                    control_point_id,
                    observation_id,
                    evidence_id
                )
            )),
            1,
            100
        ));
        assert!(!EvidenceLinksByEvidence::<Test>::contains_key(
            evidence_id,
            observation_id
        ));
        assert!(!EvidenceLinksByObservation::<Test>::contains_key(
            observation_id,
            evidence_id
        ));
    });
}

#[test]
fn delete_evidence_should_work() {
    new_test_ext().execute_with(|| {
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org_member = 2;
        let auditing_org_group_id = 2;
        let auditing_org = create_group(auditing_org_member, auditing_org_group_id);
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
        accept_audit(auditing_org_group_id, audit_id, auditing_org_member);
        //assign auditors
        let auditors_group_id = 3;
        let auditors_member = 3;
        let auditors_group_account = create_group(auditors_member, auditors_group_id);
        assign_auditors(
            audit_id,
            auditing_org_group_id,
            auditing_org_member,
            auditors_group_account,
        );

        create_observation(auditors_member, auditors_group_id, audit_id);
        let observation_id = 1;
        create_evidence(auditors_member, auditors_group_id, audit_id);
        let evidence_id = 1;

        //delete evidence
        assert_ok!(Groups::propose(
            Origin::signed(auditors_member),
            auditors_group_id,
            Box::new(crate::mock::Call::AuditsModule(
                super::Call::delete_evidence(audit_id, evidence_id, 0)
            )),
            1,
            100
        ));

        assert!(!Evidences::<Test>::contains_key(audit_id, evidence_id,));
        assert!(!EvidenceLinksByEvidence::<Test>::contains_key(
            evidence_id,
            observation_id
        ));
        assert!(!EvidenceLinksByObservation::<Test>::contains_key(
            observation_id,
            evidence_id
        ));
    });
}

#[test]
fn delete_evidence_should_have_link_limit() {
    new_test_ext().execute_with(|| {
        let audit_creator_member = 1;
        let audit_creator_group_id = 1;
        let audit_creator_group_account =
            create_group(audit_creator_member, audit_creator_group_id);
        let auditing_org_member = 2;
        let auditing_org_group_id = 2;
        let auditing_org = create_group(auditing_org_member, auditing_org_group_id);
        let audit_id = create_audit(
            audit_creator_member,
            audit_creator_group_account,
            auditing_org,
        );
        accept_audit(auditing_org_group_id, audit_id, auditing_org_member);
        //assign auditors
        let auditors_group_id = 3;
        let auditors_member = 3;
        let auditors_group_account = create_group(auditors_member, auditors_group_id);
        assign_auditors(
            audit_id,
            auditing_org_group_id,
            auditing_org_member,
            auditors_group_account,
        );

        create_evidence(auditors_member, auditors_group_id, audit_id);
        let evidence_id = 1;

        let control_point_id = 1;
        for i in 0..<Test as Config>::MaxLinkRemove::get() + 2 {
            //create observation
            create_observation(auditors_member, auditors_group_id, audit_id);
            let observation_id = i + 1;
            //link evidence
            assert_ok!(Groups::propose(
                Origin::signed(auditors_member),
                auditors_group_id,
                Box::new(crate::mock::Call::AuditsModule(super::Call::link_evidence(
                    audit_id,
                    control_point_id,
                    observation_id,
                    evidence_id
                ))),
                1,
                100
            ));
            assert!(EvidenceLinksByEvidence::<Test>::contains_key(
                evidence_id,
                observation_id
            ));
            assert!(EvidenceLinksByObservation::<Test>::contains_key(
                observation_id,
                evidence_id
            ));
        }
        // try to delete evidence with high link_count
        assert_ok!(Groups::propose(
            Origin::signed(auditors_member),
            auditors_group_id,
            Box::new(crate::mock::Call::AuditsModule(
                super::Call::delete_evidence(
                    audit_id,
                    evidence_id,
                    <Test as Config>::MaxLinkRemove::get() + 1
                )
            )),
            1,
            100
        ));

        let last_event = frame_system::Pallet::<Test>::events()
            .last()
            .unwrap()
            .clone();
        let local_event = crate::mock::Event::from(last_event.event);
        let success = match local_event {
            mock::Event::groups(groups::Event::Approved(_, _, _, _, success, _)) => success,
            _ => panic!("unexpected event"),
        };

        assert!(!success);

        // Error::<Test>::RemoveLinkLimitExceeded

        // try to delete evidence with link_count below actual
        assert_ok!(Groups::propose(
            Origin::signed(auditors_member),
            auditors_group_id,
            Box::new(crate::mock::Call::AuditsModule(
                super::Call::delete_evidence(
                    audit_id,
                    evidence_id,
                    <Test as Config>::MaxLinkRemove::get()
                )
            )),
            1,
            100
        ));

        //check that evidence was not actually deleted
        assert!(Evidences::<Test>::contains_key(audit_id, evidence_id));
    });
}

//Make sure weights cannot exceed 10% of total allowance for block.

#[test]
fn weights_should_not_be_excessive() {
    new_test_ext().execute_with(|| {
        const MAXIMUM_ALLOWED_WEIGHT: Weight = 130_000_000_000;

        let weight = <Test as Config>::WeightInfo::create_audit();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::delete_audit();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::link_audit();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::unlink_audit();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::accept_audit();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::assign_auditors_initial_assign();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::assign_auditors_replace();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::reject_audit();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::complete_audit();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::create_observation();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::create_evidence(
            <Test as Config>::NameLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::NameLimit::get(),
            <Test as Config>::NameLimit::get(),
        );
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::link_evidence();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight = <Test as Config>::WeightInfo::unlink_evidence();
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
        let weight =
            <Test as Config>::WeightInfo::delete_evidence(<Test as Config>::MaxLinkRemove::get());
        assert!(weight < MAXIMUM_ALLOWED_WEIGHT);
    });
}
