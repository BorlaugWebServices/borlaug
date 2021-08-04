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

//! Groups pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    dispatch::Vec,
    traits::{Currency, EnsureOrigin, Get, UnfilteredDispatchable},
    weights::Weight,
};
use frame_system::Call as SystemCall;
use frame_system::{self, RawOrigin as SystemOrigin};
use sp_runtime::traits::{Bounded, Hash, UniqueSaturatedInto};
use sp_std::{mem::size_of, prelude::*, vec};

#[allow(unused)]
use crate::Pallet as GroupPallet;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
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

        let mut members = vec![];
        for i in 0 .. m - 1 {
            let member = account("member", i, SEED);
            members.push(member);
        }
        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(caller.clone()), name,members,1u32.into(),1_000_000_000u32.into())

    verify {
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));
        let group=Groups::<T>::get(group_id).unwrap();
        assert_eq!(group.name.len(),a as usize);
        assert_eq!(group.members.len(),m as usize);
    }

    create_sub_group {
        let a in 1 .. (<T as Config>::NameLimit::get());
        let m in 1 .. T::MaxMembers::get().unique_saturated_into();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();

        GroupPallet::<T>::create_group(origin,vec![42u8; 2 as usize],vec![], 1u32.into(),1_000_000_000u32.into())?;

        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let mut members = vec![];
        for i in 0 .. m {
            let member = account("member", i, SEED);
            members.push(member);
        }
        let name = vec![42u8; a as usize];

        let origin=<T as Config>::GroupsOriginByGroupThreshold::successful_origin();
        let call = Call::<T>::create_sub_group(name,members,1u32.into(),1_000u32.into());

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        let sub_group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(sub_group_id));
    }

    update_group {
        let a in 1 .. (<T as Config>::NameLimit::get());
        let m in 1 .. T::MaxMembers::get().unique_saturated_into();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        GroupPallet::<T>::create_group(origin,vec![42u8; 2 as usize],vec![], 1u32.into(),1_000_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let mut members = vec![];
        for i in 0 .. m {
            let member = account("member", i, SEED);
            members.push(member);
        }
        let name = vec![42u8; a as usize];

        let origin=<T as Config>::GroupsOriginByGroupThreshold::successful_origin();
        let call = Call::<T>::update_group(group_id, Some(name),Some(members),Some(2u32.into()));

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(Groups::<T>::contains_key(group_id));
        let group=Groups::<T>::get(group_id).unwrap();
        assert_eq!(group.name.len(),a as usize);
        assert_eq!(group.members.len(),m as usize);
        assert_eq!(group.threshold,2u32.into());
    }

    remove_group {

        let p in 1 .. T::MaxProposals::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        GroupPallet::<T>::create_group(origin,vec![42u8; 2 as usize],vec![], 1u32.into(),1_000_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let mut proposal_hashes=vec![];
        for i in 0 .. p {
            let proposal: T::Proposal = SystemCall::<T>::remark(vec![1; T::MaxProposalLength::get()  as usize]).into();
            proposal_hashes.push(T::Hashing::hash_of(&proposal));
            let proposal_id:T::ProposalId=(p+1).into();
            Proposals::<T>::insert(group_id, proposal_id, proposal);
        }
        ProposalHashes::<T>::insert(group_id,proposal_hashes);

        let origin=<T as Config>::GroupsOriginByGroupThreshold::successful_origin();
        let call = Call::<T>::remove_group(group_id, caller);

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(!Groups::<T>::contains_key(group_id));
        assert!(!GroupChildren::<T>::contains_key(group_id));
        assert!(!ProposalHashes::<T>::contains_key(group_id));
        assert!(!Proposals::<T>::iter_prefix(group_id).next().is_some());
    }

    remove_sub_group {

        let p in 1 .. T::MaxProposals::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        GroupPallet::<T>::create_group(origin,vec![42u8; 2 as usize],vec![], 1u32.into(),1_000_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let origin=<T as Config>::GroupsOriginByGroupThreshold::successful_origin();
        let call = Call::<T>::create_sub_group(vec![42u8; 2 as usize],vec![account("member", 1, SEED)],1u32.into(),1_000u32.into());
        call.dispatch_bypass_filter(origin)? ;

        let sub_group_id:T::GroupId=2u32.into();
        assert!(Groups::<T>::contains_key(sub_group_id));

        let mut proposal_hashes=vec![];
        for i in 0 .. p {
            let proposal: T::Proposal = SystemCall::<T>::remark(vec![1; T::MaxProposalLength::get()  as usize]).into();
            proposal_hashes.push(T::Hashing::hash_of(&proposal));
            let proposal_id:T::ProposalId=(p+1).into();
            Proposals::<T>::insert(sub_group_id, proposal_id, proposal);
        }
        ProposalHashes::<T>::insert(sub_group_id,proposal_hashes);

        let origin=<T as Config>::GroupsOriginByGroupThreshold::successful_origin();
        let call = Call::<T>::remove_sub_group(sub_group_id);

    }: { call.dispatch_bypass_filter(origin)? }

    verify {
        assert!(!Groups::<T>::contains_key(sub_group_id));
        assert!(!GroupChildren::<T>::contains_key(sub_group_id));
        assert!(!ProposalHashes::<T>::contains_key(sub_group_id));
        assert!(!Proposals::<T>::iter_prefix(sub_group_id).next().is_some());
    }

    // This tests when execution would happen immediately after proposal
    execute {
        let a in 1 .. (<T as Config>::MaxProposalLength::get());
        let m in 1 .. T::MaxMembers::get().unique_saturated_into();

        let bytes_in_storage = a + size_of::<u32>() as u32;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let mut members = vec![];
        for i in 0 .. m - 1 {
            let member = account("member", i, SEED);
            members.push(member);
        }

        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],members, 1u32.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let proposal: T::Proposal = SystemCall::<T>::remark(vec![1; a as usize]).into();


    }: execute(SystemOrigin::Signed(caller.clone()), 1u32.into(),Box::new(proposal.clone()),bytes_in_storage)

    verify {
        let proposal_hash = T::Hashing::hash_of(&proposal);
        assert_last_event::<T>(
            Event::Executed(group_id,proposal_hash,caller,false).into()
        );
    }


// This tests when execution would happen immediately after proposal
    propose_execute {
        let a in 1 .. T::MaxProposalLength::get();
        let m in 1 .. T::MaxMembers::get().unique_saturated_into();

        let bytes_in_storage = a + size_of::<u32>() as u32;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let mut members = vec![];
        for i in 0 .. m - 1 {
            let member = account("member", i, SEED);
            members.push(member);
        }

        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],members, 1u32.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let proposal: T::Proposal = SystemCall::<T>::remark(vec![1; a as usize]).into();
        let threshold = 1u32.into();

    }: propose(SystemOrigin::Signed(caller.clone()), 1u32.into(),Box::new(proposal.clone()),threshold,bytes_in_storage)

    verify {
        let proposal_id:T::ProposalId=1u32.into();
        assert_last_event::<T>(
            Event::Approved(group_id,proposal_id,1u32.into(),0u32.into(),false).into()
        );

    }

    // This tests when proposal is created and queued as "proposed"
    propose_proposed {
        let a in 1 .. T::MaxProposalLength::get();
        let m in 2 .. T::MaxMembers::get().unique_saturated_into();
        let p in 1 .. T::MaxProposals::get();

        let bytes_in_storage = a + size_of::<u32>() as u32;

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let mut members = vec![];
        for i in 0 .. m - 1 {
            let member = account("member", i, SEED);
            members.push(member);
        }
        members.push(caller.clone());

        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],members, m.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let threshold = m.into();

        // Add previous proposals.
        for i in 0 .. p - 1 {
            // Proposals should be different so that different proposal hashes are generated
            let proposal: T::Proposal = SystemCall::<T>::remark(vec![i as u8; a as usize]).into();
            GroupPallet::<T>::propose(
                SystemOrigin::Signed(caller.clone()).into(),group_id,
                Box::new(proposal),
                threshold,
                bytes_in_storage,
            )?;
        }

        let proposal_hashes = <ProposalHashes<T>>::get(group_id);
        assert!(proposal_hashes.is_some());
        let proposal_hashes =proposal_hashes.unwrap();
        assert_eq!(proposal_hashes.len(), (p - 1) as usize);

        let proposal: T::Proposal = SystemCall::<T>::remark(vec![p as u8; a as usize]).into();

    }: propose(SystemOrigin::Signed(caller.clone()), group_id,Box::new(proposal.clone()),threshold,bytes_in_storage)

    verify {
        let proposal_hashes = <ProposalHashes<T>>::get(group_id);
        assert!(proposal_hashes.is_some());
        let proposal_hashes =proposal_hashes.unwrap();
        assert_eq!(proposal_hashes.len(), p as usize);
        let proposal_id:T::ProposalId=p.into();
        assert_last_event::<T>(Event::Proposed(caller, group_id, proposal_id,threshold).into());
    }

    vote {
        // We choose 5 as a minimum so we always trigger a vote in the voting loop (`for j in ...`)
        let m in 5 .. T::MaxMembers::get().unique_saturated_into();

        let p = T::MaxProposals::get();
        let a = T::MaxProposalLength::get();
        let bytes_in_storage = a + size_of::<u32>() as u32;

        let mut members = vec![];
        let proposer: T::AccountId = account("proposer", 0, SEED);
        members.push(proposer.clone());
        for i in 1 .. m - 1  {
            let member = account("member", i, SEED);
            members.push(member);
        }
        let voter: T::AccountId = account("voter", 0, SEED);
        members.push(voter.clone());

        T::Currency::make_free_balance_be(&voter, BalanceOf::<T>::max_value());
        GroupPallet::<T>::create_group(SystemOrigin::Signed(voter.clone()).into(),vec![42u8; 2 as usize],members.clone(), m.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        // Threshold is 1 less than the number of members so that one person can vote nay
        let threshold = (m - 1).into();

        // Add previous proposals
        for i in 0 .. p {
            // Proposals should be different so that different proposal hashes are generated
            let proposal: T::Proposal = SystemCall::<T>::remark(vec![i as u8; a as usize]).into();
            GroupPallet::<T>::propose(
                SystemOrigin::Signed(proposer.clone()).into(),
                group_id,
                Box::new(proposal.clone()),
                threshold,
                bytes_in_storage,
            )?;
        }

        for i in 0 .. p {
            let p_id:T::ProposalId=(i+1).into();
            assert!(Proposals::<T>::contains_key(group_id,p_id));
        }

        let proposal_id:T::ProposalId=p.into();

        // Have almost everyone vote aye on last proposal, while keeping it from passing.
        for j in 1 .. m - 3 {
            let other_voter = &members[j as usize];
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
        for i in 0 .. p {
            let p_id:T::ProposalId=(i+1).into();
            assert!(Proposals::<T>::contains_key(group_id,p_id));
        }
        let voting = GroupPallet::<T>::voting(group_id,proposal_id).ok_or(Error::<T>::ProposalMissing)?;
        assert_eq!(voting.ayes.len(), (m - 3) as usize);
        assert_eq!(voting.nays.len(), 1);
    }


    close_disapproved {
        //start at 2 so that a vote is always required
        let m in 2 .. T::MaxMembers::get().unique_saturated_into();
        let p in 1 .. T::MaxProposals::get();

        let bytes = 100;
        let bytes_in_storage = bytes + size_of::<u32>() as u32;

        // Construct `members`.
        let mut members = vec![];
        for i in 0 .. m - 1 {
            let member = account("member", i, SEED);
            members.push(member);
        }
        let caller: T::AccountId = whitelisted_caller();
        members.push(caller.clone());

        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],members.clone(), m.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        let threshold = m.into();

        // Add proposals
        for i in 0 .. p {
            // Proposals should be different so that different proposal hashes are generated
            let proposal: T::Proposal = SystemCall::<T>::remark(vec![i as u8; bytes as usize]).into();
            GroupPallet::<T>::propose(
                SystemOrigin::Signed(caller.clone()).into(),
                group_id,
                Box::new(proposal.clone()),
                threshold,
                bytes_in_storage,
            )?;
        }

        for i in 0 .. p {
            let p_id:T::ProposalId=(i+1).into();
            assert!(Proposals::<T>::contains_key(group_id,p_id));
        }

        let proposal_id:T::ProposalId=p.into();

        // Everyone except proposer votes nay
        for j in 0 .. m - 1  {
            let voter = &members[j as usize];
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
        let p in 1 .. T::MaxProposals::get();

        let bytes_in_storage = a + size_of::<u32>() as u32;


        // Construct `members`.
        let mut members = vec![];
        for i in 0 .. m - 1 {
            let member = account("member", i, SEED);
            members.push(member);
        }
        let caller: T::AccountId = whitelisted_caller();
        members.push(caller.clone());

        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],members.clone(), m.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        // Threshold is two, so any two ayes will pass the vote
        let threshold = m.into();

        // Add proposals
        for i in 0 .. p {
            // Proposals should be different so that different proposal hashes are generated
            let proposal: T::Proposal = SystemCall::<T>::remark(vec![i as u8; a as usize]).into();
            GroupPallet::<T>::propose(
                SystemOrigin::Signed(caller.clone()).into(),
                group_id,
                Box::new(proposal.clone()),
                threshold,
                bytes_in_storage,
            )?;
        }

        for i in 0 .. p {
            let p_id:T::ProposalId=(i+1).into();
            assert!(Proposals::<T>::contains_key(group_id,p_id));
        }

        let proposal_id:T::ProposalId=p.into();

         // Everyone except proposer votes yes
         for j in 0 .. m - 1  {
            let voter = &members[j as usize];
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
        assert_last_event::<T>(Event::Approved(group_id,proposal_id,m.into(),0u32.into(),false).into());
    }

    veto_approved {
        let a in 1 .. T::MaxProposalLength::get();
        //start at 2 so that a vote is always required
        let m in 2 .. T::MaxMembers::get().unique_saturated_into();
        let p in 1 .. T::MaxProposals::get();

        let bytes_in_storage = a + size_of::<u32>() as u32;


        // Construct `members`.
        let mut members = vec![];
        for i in 0 .. m - 1 {
            let member = account("member", i, SEED);
            members.push(member);
        }
        let caller: T::AccountId = whitelisted_caller();
        members.push(caller.clone());

        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        GroupPallet::<T>::create_group(SystemOrigin::Signed(caller.clone()).into(),vec![42u8; 2 as usize],members.clone(), m.into(),1_000_000u32.into())?;
        let group_id:T::GroupId=1u32.into();
        assert!(Groups::<T>::contains_key(group_id));

        // Threshold is two, so any two ayes will pass the vote
        let threshold = m.into();

        // Add proposals
        for i in 0 .. p {
            // Proposals should be different so that different proposal hashes are generated
            let proposal: T::Proposal = SystemCall::<T>::remark(vec![i as u8; a as usize]).into();
            GroupPallet::<T>::propose(
                SystemOrigin::Signed(caller.clone()).into(),
                group_id,
                Box::new(proposal.clone()),
                threshold,
                bytes_in_storage,
            )?;
        }

        for i in 0 .. p {
            let p_id:T::ProposalId=(i+1).into();
            assert!(Proposals::<T>::contains_key(group_id,p_id));
        }

        let proposal_id:T::ProposalId=p.into();

         // Everyone except proposer votes no
         for j in 0 .. m - 1  {
            let voter = &members[j as usize];
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
        assert_eq!(votes.ayes.len(),m as usize);


    }: close(SystemOrigin::Signed(caller), group_id, proposal_id, Weight::max_value(), bytes_in_storage)

    verify {
        assert!(!Proposals::<T>::contains_key(group_id,proposal_id));
        assert_last_event::<T>(Event::Approved(group_id,proposal_id,m.into(),0u32.into(),false).into());
    }


}

impl_benchmark_test_suite!(GroupPallet, crate::mock::new_test_ext(), crate::mock::Test,);
