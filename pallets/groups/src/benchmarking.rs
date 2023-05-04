//! Groups pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::sp_runtime::traits::Hash;
use frame_support::{
    traits::{Currency, EnsureOrigin, Get, UnfilteredDispatchable},
    weights::Weight,
};
use frame_system::pallet_prelude::OriginFor;
use frame_system::Call as SystemCall;
use frame_system::{self, RawOrigin as SystemOrigin};
use sp_runtime::traits::{Bounded, UniqueSaturatedInto};
use sp_std::{mem::size_of, prelude::*, vec};

#[allow(unused)]
use crate::Pallet as GroupPallet;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    assert_eq!(
        frame_system::Pallet::<T>::events()
            .last()
            .expect("events expected")
            .event,
        generic_event.into()
    );
}

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

benchmarks! {
    create_group {
        let a in 1 .. <T as Config>::NameLimit::get();
        let m in 1 .. T::MaxMembers::get().unique_saturated_into();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let mut members = vec![(caller.clone(),1u32.into())];
        for i in 1 .. m {
            let member = account("member", i, SEED);
            members.push((member,1u32.into()));
        }
        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(caller.clone()), name,members,1u32.into(),1_000_000_000u32.into())

    verify {
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));
        let group=Groups::<T>::get(group_id).unwrap();
        assert_eq!(group.name.len(),a as usize);
        assert_eq!(GroupMembers::<T>::iter_prefix(group_id).count(),m as usize);
    }

    update_group {
        let a in 1 .. <T as Config>::NameLimit::get();
        let n in 1 .. T::MaxMembers::get().unique_saturated_into(); //add_members
        let o in 1 .. T::MaxMembers::get().unique_saturated_into(); //remove_members

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin=SystemOrigin::Signed(caller.clone()).into();

        let mut origional_members = vec![];
        let mut remove=vec![];
        for i in 0 .. o {
            let member:T::AccountId = account("member", i, SEED);
            origional_members.push((member.clone(),1u32.into()));
            remove.push(member);
        }

        GroupPallet::<T>::create_group(origin,vec![42u8; 2 as usize],origional_members, 1u32.into(),1_000_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let mut new_members = vec![];
        //don't overlap members
        //TODO: verify that performance is not worse for overlaps
        for i in o .. o+n {
            let member = account("member", i, SEED);
            new_members.push((member,1u32.into()));
        }
        let name = vec![42u8; a as usize];

        let origin=<T as Config>::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::update_group {
            name:  Some(name),
            add_members: Some(new_members),
            remove_members:   Some(remove),
            threshold:   Some(2u32.into())
        };

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(Groups::<T>::contains_key(group_id));
        let group=Groups::<T>::get(group_id).unwrap();
        assert_eq!(group.name.len(),a as usize);
        assert_eq!(GroupMembers::<T>::iter_prefix(group_id).count(),n as usize);
    }

    create_sub_group {
        let a in 1 .. <T as Config>::NameLimit::get();
        let m in 1 .. T::MaxMembers::get().unique_saturated_into();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin=SystemOrigin::Signed(caller.clone()).into();

        GroupPallet::<T>::create_group(origin,vec![42u8; 2 as usize],vec![(caller.clone(), 1u32.into())], 1u32.into(),1_000_000_000u32.into())?;

        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let mut members = vec![];
        for i in 0 .. m {
            let member = account("member", i, SEED);
            members.push((member,1u32.into()));
        }
        let name = vec![42u8; a as usize];

        let origin=<T as Config>::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();


        let call = Call::<T>::create_sub_group{
            name,
            members,
            threshold: 1u32.into(),
            initial_balance:1_000u32.into()
        };

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let sub_group_id:T::GroupId=2u32.into();
        assert!(Groups::<T>::contains_key(sub_group_id));
        assert_eq!(GroupMembers::<T>::iter_prefix(sub_group_id).count(),m as usize);
    }

    update_sub_group {
        let a in 1 .. <T as Config>::NameLimit::get();
        let n in 1 .. T::MaxMembers::get().unique_saturated_into(); //add_members
        let o in 1 .. T::MaxMembers::get().unique_saturated_into(); //remove_members

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin=SystemOrigin::Signed(caller.clone()).into();

        GroupPallet::<T>::create_group(origin,vec![42u8; 2 as usize],vec![(caller.clone(), 1u32.into())], 1u32.into(),1_000_000_000u32.into())?;

        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let mut origional_members = vec![];
        let mut remove=vec![];
        for i in 0 .. o {
            let member:T::AccountId = account("member", i, SEED);
            origional_members.push((member.clone(),1u32.into()));
            remove.push(member);
        }

        let origin=<T as Config>::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::create_sub_group{
            name:vec![42u8; 2 as usize],
            members:   origional_members,
            threshold:1u32.into(),
            initial_balance:  1_000u32.into()
        };
        call.dispatch_bypass_filter(origin.clone())? ;

        let mut new_members = vec![];
        //don't overlap members
        //TODO: verify that performance is not worse for overlaps
        for i in o .. o+n {
            let member = account("member", i, SEED);
            new_members.push((member,1u32.into()));
        }
        let name = vec![42u8; a as usize];

        let sub_group_id:T::GroupId=2u32.into();

        let call = Call::<T>::update_sub_group{
            sub_group_id,
            name: Some(name),
            add_members:  Some(new_members),
            remove_members: Some(remove),
            threshold:  Some(2u32.into())
            };

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(Groups::<T>::contains_key(sub_group_id));
        let sub_group=Groups::<T>::get(sub_group_id).unwrap();
        assert_eq!(sub_group.name.len(),a as usize);
        assert_eq!(GroupMembers::<T>::iter_prefix(sub_group_id).count(),n as usize);
    }

    remove_group {

        let m in 1 .. T::MaxMembers::get().unique_saturated_into();
        let p in 1 .. T::MaxProposals::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin=SystemOrigin::Signed(caller.clone()).into();

        let mut members = vec![(caller.clone(),1u32.into())];
        for i in 1 .. m {
            let member = account("member", i, SEED);
            members.push((member,1u32.into()));
        }

        GroupPallet::<T>::create_group(origin,vec![42u8; 2 as usize],members, 1u32.into(),1_000_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        for i in 0 .. p {
            let proposal: T::Proposal = SystemCall::<T>::remark{ remark: vec![1; T::MaxProposalLength::get()  as usize]}.into();
            let hash=T::Hashing::hash_of(&proposal);
            let proposal_id:T::ProposalId=i.into();
            Proposals::<T>::insert(group_id, proposal_id, proposal);
            ProposalHashes::<T>::insert(group_id,  hash,());
        }

        let origin=<T as Config>::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::remove_group{
            group_id,
            return_funds_too: caller
        };

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(!Groups::<T>::contains_key(group_id));
        assert!(GroupChildren::<T>::iter_prefix(group_id).next().is_none());
        assert!(GroupMembers::<T>::iter_prefix(group_id).next().is_none());
        assert!(ProposalHashes::<T>::iter_prefix(group_id).next().is_none());
        assert!(Proposals::<T>::iter_prefix(group_id).next().is_none());
    }

    remove_sub_group {

        let m in 1 .. T::MaxMembers::get().unique_saturated_into();
        let p in 1 .. T::MaxProposals::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin=SystemOrigin::Signed(caller.clone()).into();
        GroupPallet::<T>::create_group(origin,vec![42u8; 2 as usize],vec![(caller.clone(),1u32.into())], 1u32.into(),1_000_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let origin=<T as Config>::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();

        let mut members = vec![];
        for i in 0 .. m {
            let member = account("member", i, SEED);
            members.push((member,1u32.into()));
        }

        let call = Call::<T>::create_sub_group{
            name:  vec![42u8; 2 as usize],
            members,
            threshold: 1u32.into(),
            initial_balance:  1_000u32.into()
        };
        call.dispatch_bypass_filter(origin)? ;

        let sub_group_id:T::GroupId=2u32.into();
        assert!(Groups::<T>::contains_key(sub_group_id));

        for i in 0 .. p {
            let proposal: T::Proposal = SystemCall::<T>::remark{remark:vec![1; T::MaxProposalLength::get()  as usize]}.into();
            let hash=T::Hashing::hash_of(&proposal);
            let proposal_id:T::ProposalId=i.into();
            Proposals::<T>::insert(group_id, proposal_id, proposal);
            ProposalHashes::<T>::insert(group_id,  hash,());
        }

        let origin=<T as Config>::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::remove_sub_group{sub_group_id};

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(!Groups::<T>::contains_key(sub_group_id));
        assert!(GroupChildren::<T>::iter_prefix(group_id).next().is_none());
        assert!(GroupMembers::<T>::iter_prefix(sub_group_id).next().is_none());
        assert!(ProposalHashes::<T>::iter_prefix(sub_group_id).next().is_none());
        assert!(Proposals::<T>::iter_prefix(sub_group_id).next().is_none());
    }

    // This tests when execution would happen immediately after proposal
    execute {
        let a in 1 .. <T as Config>::MaxProposalLength::get();

        let bytes_in_storage = a + size_of::<u32>() as u32;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],vec![(caller.clone(),1u32.into())], 1u32.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let proposal: T::Proposal = SystemCall::<T>::remark{remark:vec![1; a as usize]}.into();

    }: execute(SystemOrigin::Signed(caller.clone()), 1u32.into(),Box::new(proposal.clone()),bytes_in_storage)

    verify {
        let proposal_hash = T::Hashing::hash_of(&proposal);

        //Cannot check event because error is not known.
        // assert_last_event::<T>(
        //     Event::Executed(group_id,proposal_hash,caller,false,None).into()
        // );
    }


// This tests when execution would happen immediately after proposal
    propose_execute {
        let a in 1 .. T::MaxProposalLength::get();

        let bytes_in_storage = a + size_of::<u32>() as u32;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let threshold = 1u32.into();

        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],vec![(caller.clone(),1u32.into())], threshold,1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let proposal: T::Proposal = SystemCall::<T>::remark{remark:vec![1; a as usize]}.into();

    }: propose(SystemOrigin::Signed(caller.clone()), 1u32.into(),Box::new(proposal.clone()),threshold,bytes_in_storage)

    verify {
        let proposal_id:T::ProposalId=1u32.into();
        //Cannot check event because error is not known.
        // assert_last_event::<T>(
        //     Event::Approved(group_id,proposal_id,1u32.into(),0u32.into(),false,None).into()
        // );

    }

    // This tests when proposal is created and queued as "proposed"
    propose_proposed {
        let a in 1 .. T::MaxProposalLength::get();

        let bytes_in_storage = a + size_of::<u32>() as u32;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let members = vec![(caller.clone(),1u32.into()),(account("member", 1, SEED),1u32.into())];

        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(), vec![42u8; 2 as usize], members, 2u32.into(), 1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let threshold = 2u32.into();


        let proposal: T::Proposal = SystemCall::<T>::remark{remark:vec![42u8 as u8; a as usize]}.into();

    }: propose(SystemOrigin::Signed(caller.clone()), group_id,Box::new(proposal.clone()),threshold,bytes_in_storage)

    verify {
        let hash=T::Hashing::hash_of(&proposal);
        assert!(ProposalHashes::<T>::contains_key(group_id,hash));
        let proposal_id:T::ProposalId=1u32.into();
        assert_last_event::<T>(Event::Proposed(caller, group_id, proposal_id,threshold).into());
    }

    vote {
        // We choose 5 as a minimum so we always trigger a vote in the voting loop (`for j in ...`)
        let m in 5 .. T::MaxMembers::get().unique_saturated_into();

        let a = T::MaxProposalLength::get();
        let bytes_in_storage = a + size_of::<u32>() as u32;

        let mut members = vec![];
        let proposer: T::AccountId = account("proposer", 0, SEED);
        members.push((proposer.clone(),1u32.into()));
        for i in 1 .. m - 1  {
            let member = account("member", i, SEED);
            members.push((member,1u32.into()));
        }
        let voter: T::AccountId = account("voter", 0, SEED);
        members.push((voter.clone(),1u32.into()));

        T::Currency::make_free_balance_be(&voter, BalanceOf::<T>::max_value());
        GroupPallet::<T>::create_group(SystemOrigin::Signed(voter.clone()).into(),vec![42u8; 2 as usize],members.clone(), m.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        // Threshold is 1 less than the number of members so that one person can vote nay
        let threshold:T::MemberCount = (m - 1).into();

        let proposal: T::Proposal = SystemCall::<T>::remark{remark:vec![1; a as usize]}.into();GroupPallet::<T>::propose(
            SystemOrigin::Signed(proposer.clone()).into(),
            group_id,
            Box::new(proposal.clone()),
            threshold,
            bytes_in_storage,
        )?;

        let proposal_id:T::ProposalId=1u32.into();
        assert!(Proposals::<T>::contains_key(group_id,proposal_id));

        // Have almost everyone vote aye on last proposal, while keeping it from passing.
        for j in 1 .. m - 3 {
            let (other_voter,_) = &members[j as usize];
            let approve = true;
            GroupPallet::<T>::vote(
                SystemOrigin::Signed(other_voter.clone()).into(),
                group_id,
                proposal_id,
                approve,
            )?;
        }

        let votes=Voting::<T>::get(group_id,proposal_id);
        assert!(votes.is_some());
        let votes=votes.unwrap();
        assert_eq!(votes.ayes.len(),(m - 3) as usize);


        // Voter votes aye without resolving the vote.
        let approve = true;
        GroupPallet::<T>::vote(
            SystemOrigin::Signed(voter.clone()).into(),
            group_id,
            proposal_id,
            approve,
        )?;

        // Voter switches vote to nay, but does not kill the vote, just updates + inserts
        let approve = false;

        // Whitelist voter account from further DB operations.
        let voter_key = frame_system::Account::<T>::hashed_key_for(&voter);
        frame_benchmarking::benchmarking::add_to_whitelist(voter_key.into());

    }: _(SystemOrigin::Signed(voter.clone()), group_id, proposal_id, approve)

    verify {
        // All proposals exist and the last proposal has just been updated.
            assert!(Proposals::<T>::contains_key(group_id,proposal_id));
        let voting = GroupPallet::<T>::voting(group_id,proposal_id).ok_or(Error::<T>::ProposalMissing).unwrap();
        assert_eq!(voting.ayes.len(), (m - 3) as usize);
        assert_eq!(voting.nays.len(), 1);
    }


    close_disapproved {
        //start at 2 so that a vote is always required
        let m in 2 .. T::MaxMembers::get().unique_saturated_into();

        let bytes = 100;
        let bytes_in_storage = bytes + size_of::<u32>() as u32;

        let caller: T::AccountId = whitelisted_caller();

        // Construct `members`.
        let mut members = vec![(caller.clone(),1u32.into())];
        for i in 1 .. m {
            let member = account("member", i, SEED);
            members.push((member,1u32.into()));
        }

        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],members.clone(), m.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let threshold = m.into();

        let proposal: T::Proposal = SystemCall::<T>::remark{remark:vec![1; 100 as usize]}.into();GroupPallet::<T>::propose(
            SystemOrigin::Signed(caller.clone()).into(),
            group_id,
            Box::new(proposal.clone()),
            threshold,
            bytes_in_storage,
        )?;

        let proposal_id:T::ProposalId=1u32.into();
        assert!(Proposals::<T>::contains_key(group_id,proposal_id));

        // Everyone except proposer votes nay
        for j in 1 .. m   {
            let (voter,_) = &members[j as usize];
            let approve = false;
            GroupPallet::<T>::vote(
                SystemOrigin::Signed(voter.clone()).into(),
                group_id,
                proposal_id,
                approve,
            )?;
        }

        let votes=Voting::<T>::get(group_id,proposal_id);
        assert!(votes.is_some());
        let votes=votes.unwrap();
        assert_eq!(votes.nays.len(),(m-1) as usize);

    }: close(SystemOrigin::Signed(caller), group_id, proposal_id, Weight::max_value(), bytes_in_storage)

    verify {
        assert!(!Proposals::<T>::contains_key(group_id,proposal_id));
        assert_last_event::<T>(Event::Disapproved(group_id,proposal_id,1u32.into(),(m-1).into()).into());

    }

    close_approved {
        let a in 1 .. T::MaxProposalLength::get();
        //start at 2 so that a vote is always required
        let m in 2 .. T::MaxMembers::get().unique_saturated_into();

        let bytes_in_storage = a + size_of::<u32>() as u32;

        let caller: T::AccountId = whitelisted_caller();

        // Construct `members`.
        let mut members = vec![(caller.clone(),1u32.into())];
        for i in 1 .. m {
            let member = account("member", i, SEED);
            members.push((member,1u32.into()));
        }

        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],members.clone(), m.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        // Threshold is two, so any two ayes will pass the vote
        let threshold = m.into();

        let proposal: T::Proposal = SystemCall::<T>::remark{remark:vec![1; a as usize]}.into();

        GroupPallet::<T>::propose(
            SystemOrigin::Signed(caller.clone()).into(),
            group_id,
            Box::new(proposal.clone()),
            threshold,
            bytes_in_storage,
        )?;

        let proposal_id:T::ProposalId=1u32.into();
        assert!(Proposals::<T>::contains_key(group_id,proposal_id));

         // Everyone except proposer votes yes
         for j in 1 .. m   {
            let (voter,_) = &members[j as usize];
            let approve = true;
            GroupPallet::<T>::vote(
                SystemOrigin::Signed(voter.clone()).into(),
                group_id,
                proposal_id,
                approve,
            )?;
        }

        let votes=Voting::<T>::get(group_id,proposal_id);
        assert!(votes.is_some());
        let votes=votes.unwrap();
        assert_eq!(votes.ayes.len(),m as usize);

    }: close(SystemOrigin::Signed(caller), group_id, proposal_id, Weight::max_value(), bytes_in_storage)

    verify {
        assert!(!Proposals::<T>::contains_key(group_id,proposal_id));
        //Cannot check event because error is not known.
        // assert_last_event::<T>(Event::Approved(group_id,proposal_id,m.into(),0u32.into(),true,None).into());
    }

    veto_disapproved {

        let bytes = 100;
        let bytes_in_storage = bytes + size_of::<u32>() as u32;

        let admin: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&admin, BalanceOf::<T>::max_value());
        GroupPallet::<T>::create_group(SystemOrigin::Signed(admin.clone()).into(),vec![42u8; 2 as usize],vec![(admin.clone(),1u32.into())], 1u32.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        // Construct `members`.
        let mut members = vec![];
        for i in 0 .. 2  {
            let member = account("member", i, SEED);
            members.push((member,1u32.into()));
        }
        let threshold:T::MemberCount = 2u32.into();

        let origin=<T as Config>::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::create_sub_group{
            name:  vec![42u8; 2 as usize],
            members:   members.clone(),
            threshold,
            initial_balance:1_000u32.into()
        };
        call.dispatch_bypass_filter(origin)? ;
        let sub_group_id:T::GroupId=2u32.into();
        assert!(Groups::<T>::contains_key(sub_group_id));

        // Add proposal

        let proposal: T::Proposal = SystemCall::<T>::remark{remark:vec![42u8; bytes as usize]}.into();
        let (proposer,_)=&members[0 as usize];
        GroupPallet::<T>::propose(
            SystemOrigin::Signed(proposer.clone()).into(),
            sub_group_id,
            Box::new(proposal.clone()),
            threshold,
            bytes_in_storage,
        )?;

        let proposal_id:T::ProposalId=1u32.into();

        assert!(Proposals::<T>::contains_key(sub_group_id,proposal_id));

        let votes=Voting::<T>::get(sub_group_id,proposal_id);
        assert!(votes.is_some());
        let votes=votes.unwrap();
        assert_eq!(votes.ayes.len(),1);

    }: veto(SystemOrigin::Signed(admin.clone()), sub_group_id, proposal_id, false,Weight::max_value(), bytes_in_storage)

    verify {
        assert!(!Proposals::<T>::contains_key(sub_group_id,proposal_id));
        assert_last_event::<T>(Event::DisapprovedByVeto(admin,sub_group_id,proposal_id).into());
    }

    veto_approved {
        let a in 1 .. T::MaxProposalLength::get();

        let bytes_in_storage = a + size_of::<u32>() as u32;

        let admin: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&admin, BalanceOf::<T>::max_value());
        GroupPallet::<T>::create_group(SystemOrigin::Signed(admin.clone()).into(),vec![42u8; 2 as usize],vec![(admin.clone(),1u32.into())], 1u32.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        // Construct `members`.
        let mut members = vec![];
        for i in 0 .. 2  {
            let member = account("member", i, SEED);
            members.push((member,1u32.into()));
        }
        let threshold = 2u32.into();

        let origin=<T as Config>::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::create_sub_group{
            name:  vec![42u8; 2 as usize],
            members:    members.clone(),
            threshold,
            initial_balance:   1_000u32.into()
        };
        call.dispatch_bypass_filter(origin)? ;
        let sub_group_id:T::GroupId=2u32.into();
        assert!(Groups::<T>::contains_key(sub_group_id));

        // Add proposal

            let proposal: T::Proposal = SystemCall::<T>::remark{remark:vec![42u8; a as usize]}.into();
            let (proposer,_)=&members[0 as usize];
            GroupPallet::<T>::propose(
                SystemOrigin::Signed(proposer.clone()).into(),
                sub_group_id,
                Box::new(proposal.clone()),
                threshold,
                bytes_in_storage,
            )?;


            let proposal_id:T::ProposalId=1u32.into();

            assert!(Proposals::<T>::contains_key(sub_group_id,proposal_id));

        let votes=Voting::<T>::get(sub_group_id,proposal_id);
        assert!(votes.is_some());
        let votes=votes.unwrap();
        assert_eq!(votes.nays.len(),0 as usize);


    }: veto(SystemOrigin::Signed(admin.clone()), sub_group_id, proposal_id, true,Weight::max_value(), bytes_in_storage)

    verify {
        assert!(!Proposals::<T>::contains_key(sub_group_id,proposal_id));
        //Cannot check event because error is not known.
        // assert_last_event::<T>(Event::ApprovedByVeto(admin,sub_group_id,proposal_id,false,None).into());
    }

    withdraw_funds_group {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, 3_000_000u32.into());

        let origin=SystemOrigin::Signed(caller.clone()).into();

        let members = vec![(caller.clone(),1u32.into())];

        GroupPallet::<T>::create_group(origin,vec![42u8; 2 as usize],members, 1u32.into(),2_000_000u32.into())?;

        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let origin=<T as Config>::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::withdraw_funds_group{
            target_account: caller.clone(),
            amount:  1_000_000u32.into()
        };

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let group=Groups::<T>::get(group_id).unwrap();
        //minimum_balance = 500
        assert_eq!(T::Currency::free_balance(&group.anonymous_account),1_000_500u32.into());
        //minimum_balance = 500
        assert_eq!(T::Currency::free_balance(&caller),1_999_500u32.into());
    }

    withdraw_funds_sub_group {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:OriginFor<T> = SystemOrigin::Signed(caller.clone()).into();
        GroupPallet::<T>::create_group(origin.clone(),vec![42u8; 2 as usize],vec![(caller.clone(), 1u32.into())], 1u32.into(),3_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let origin=<T as Config>::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();

        let call = Call::<T>::create_sub_group{
            name: vec![42u8; 2 as usize],
            members:  vec![(caller.clone(),   1u32.into())],
            threshold:  1u32.into(),
            initial_balance:  2_000_000u32.into()
        };
        call.dispatch_bypass_filter(origin.clone())? ;

        let sub_group_id:T::GroupId=2u32.into();
        assert!(Groups::<T>::contains_key(sub_group_id));


        let call = Call::<T>::withdraw_funds_sub_group{
            sub_group_id,
            amount:1_000_000u32.into()
        };

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let group=Groups::<T>::get(group_id).unwrap();
        let sub_group=Groups::<T>::get(sub_group_id).unwrap();
        assert_eq!(T::Currency::free_balance(&group.anonymous_account),2_000_000u32.into());
        //minimum_balance = 500
        assert_eq!(T::Currency::free_balance(&sub_group.anonymous_account),1_000_500u32.into() );
    }

    send_funds_to_sub_group {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:OriginFor<T> = SystemOrigin::Signed(caller.clone()).into();
        GroupPallet::<T>::create_group(origin.clone(),vec![42u8; 2 as usize],vec![(caller.clone(), 1u32.into())], 1u32.into(),3_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let origin=<T as Config>::GroupsOriginByGroupThreshold::try_successful_origin().unwrap();
        let call = Call::<T>::create_sub_group{
            name:   vec![42u8; 2 as usize],
            members:  vec![(caller.clone(), 1u32.into())],
            threshold:     1u32.into(),
            initial_balance:    1_000_000u32.into()
        };
        call.dispatch_bypass_filter(origin.clone())? ;

        let sub_group_id:T::GroupId=2u32.into();
        assert!(Groups::<T>::contains_key(sub_group_id));


        let call = Call::<T>::send_funds_to_sub_group{
            sub_group_id,
            amount: 1_000_000u32.into()
        };

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let group=Groups::<T>::get(group_id).unwrap();
        let sub_group=Groups::<T>::get(sub_group_id).unwrap();
        assert_eq!(T::Currency::free_balance(&group.anonymous_account),1_000_000u32.into());
        //minimum_balance = 500
        assert_eq!(T::Currency::free_balance(&sub_group.anonymous_account),2_000_500u32.into());
    }


}

impl_benchmark_test_suite!(GroupPallet, crate::mock::new_test_ext(), crate::mock::Test,);
