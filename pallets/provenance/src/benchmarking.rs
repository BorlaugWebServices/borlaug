//! Provenance pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    dispatch::Vec,
    traits::{Currency, Get},
};
use frame_system::{self, RawOrigin as SystemOrigin};
use primitives::*;
use sp_runtime::traits::{Bounded, UniqueSaturatedFrom};
use sp_std::{prelude::*, vec};

#[allow(unused)]
use crate::Pallet as ProvenancePallet;

type BalanceOf<T> =
    <<T as groups::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

benchmarks! {
    create_registry {
        let a in 1 .. (<T as Config>::NameLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(caller.clone()),name.clone())

    verify {
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        let registry=<Registries<T>>::get(caller,registry_id);
        assert!(registry.is_some());
        assert_eq!(registry.unwrap().name.len(),name.len());
    }

    update_registry {
        let a in 1 .. (<T as Config>::NameLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;

        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,name.clone())

    verify {
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        let registry=<Registries<T>>::get(caller,registry_id);
        assert!(registry.is_some());
        assert_eq!(registry.unwrap().name.len(),name.len());
    }

    remove_registry {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;

        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

    }: _(SystemOrigin::Signed(caller.clone()),registry_id)

    verify {
        assert!(!<Registries<T>>::contains_key(caller.clone(),registry_id));
    }

    create_definition {

        let a in 1 .. (<T as Config>::NameLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,name)

    verify {
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let definition=<Definitions<T>>::get(registry_id, definition_id);
        assert!(definition.is_some());
        let definition=definition.unwrap();
        assert_eq!(definition.name.len(),a as usize);
    }

    update_definition {

        let a in 1 .. (<T as Config>::NameLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);

        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,name)

    verify {
        let definition=<Definitions<T>>::get(registry_id, definition_id);
        assert!(definition.is_some());
        let definition=definition.unwrap();
        assert_eq!(definition.name.len(),a as usize);

    }

    set_definition_active {

        let a in 1 .. (<T as Config>::DefinitionStepLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);

        for i in 0..a {
            ProvenancePallet::<T>::create_definition_step(origin.clone(),registry_id,definition_id,T::DefinitionStepIndex::unique_saturated_from(i),
            vec![42u8],Some(caller.clone()),T::MemberCount::unique_saturated_from(1u32))?;
        }

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id)

    verify {
        let definition=<Definitions<T>>::get(registry_id, definition_id);
        assert!(definition.is_some());
        let definition=definition.unwrap();
        assert_eq!(definition.status,DefinitionStatus::Active);
    }

    set_definition_inactive {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id)

    verify {
        let definition=<Definitions<T>>::get(registry_id, definition_id);
        assert!(definition.is_some());
        let definition=definition.unwrap();
        assert_eq!(definition.status,DefinitionStatus::Inactive);
    }

    remove_definition {

        let a in 1 .. (<T as Config>::DefinitionStepLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        for i in 0..a {
            ProvenancePallet::<T>::create_definition_step(origin.clone(),registry_id,definition_id,T::DefinitionStepIndex::unique_saturated_from(i),
            vec![42u8],Some(caller.clone()),T::MemberCount::unique_saturated_from(1u32))?;
        }

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id)

    verify {
        assert!(!<Definitions<T>>::contains_key(registry_id, definition_id));
        assert!(!<DefinitionSteps<T>>::contains_key((registry_id, definition_id),T::DefinitionStepIndex::unique_saturated_from(0u32)));
    }

    create_definition_step {
        let a in 1 .. (<T as Config>::NameLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);

        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);
        let name = vec![42u8; a as usize];
        let attestor = Some(whitelisted_caller());
        let threshold=T::MemberCount::unique_saturated_from(1u32);

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,definition_step_index,name.clone(),attestor.clone(),threshold)

    verify {
        let definition_step=<DefinitionSteps<T>>::get((registry_id, definition_id),definition_step_index);
        assert!(definition_step.is_some());
        let definition_step=definition_step.unwrap();
        assert_eq!(definition_step.name.len(),name.len());
        assert_eq!(definition_step.attestor,attestor);
        assert_eq!(definition_step.threshold,threshold);
    }

    update_definition_step {
        let a in 1 .. (<T as Config>::NameLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);
        ProvenancePallet::<T>::create_definition_step(origin.clone(),registry_id,definition_id,definition_step_index,vec![42u8],Some(whitelisted_caller()),T::MemberCount::unique_saturated_from(2u32))?;

        let name = vec![42u8; a as usize];
        let attestor = Some(whitelisted_caller());
        let threshold=T::MemberCount::unique_saturated_from(1u32);

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,definition_step_index,Some(name.clone()),Some(attestor.clone()),Some(threshold))

    verify {
        let definition_step=<DefinitionSteps<T>>::get((registry_id, definition_id),definition_step_index);
        assert!(definition_step.is_some());
        let definition_step=definition_step.unwrap();
        assert_eq!(definition_step.name.len(),name.len());
        assert_eq!(definition_step.attestor,attestor);
        assert_eq!(definition_step.threshold,threshold);
    }

    delete_definition_step {
        let a in 1 .. (<T as Config>::DefinitionStepLimit::get()-1); //existing steps

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);

        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        for i in 0..a {
            ProvenancePallet::<T>::create_definition_step(origin.clone(),registry_id,definition_id,T::DefinitionStepIndex::unique_saturated_from(i),
            vec![42u8],Some(caller.clone()),T::MemberCount::unique_saturated_from(1u32))?;
        }

        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,definition_step_index)

    verify {
        assert!(!<DefinitionSteps<T>>::contains_key((registry_id, definition_id),T::DefinitionStepIndex::unique_saturated_from(a)));
    }

    create_process {
        let a in 1 .. (<T as Config>::NameLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);
        let attestor:T::AccountId = whitelisted_caller();
        ProvenancePallet::<T>::create_definition_step(origin.clone(),registry_id,definition_id,definition_step_index,vec![42u8],Some(attestor.clone()),T::MemberCount::unique_saturated_from(1u32))?;
        ProvenancePallet::<T>::set_definition_active(origin.clone(),registry_id,definition_id)?;

        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(attestor.clone()),registry_id,definition_id,name.clone())

    verify {
        let process_id=T::ProcessId::unique_saturated_from(1u32);
        let process=<Processes<T>>::get((registry_id, definition_id),process_id);
        assert!(process.is_some());
        let process=process.unwrap();
        assert_eq!(process.name.len(),name.len());
        assert_eq!(process.status,ProcessStatus::InProgress);
        assert!(<ProcessSteps<T>>::contains_key((registry_id, definition_id,process_id),definition_step_index));
    }

    update_process {
        let a in 1 .. (<T as Config>::NameLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);
        let attestor:T::AccountId = whitelisted_caller();
        let attestor_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(attestor.clone()).into();
        ProvenancePallet::<T>::create_definition_step(origin.clone(),registry_id,definition_id,definition_step_index,vec![42u8],Some(attestor),T::MemberCount::unique_saturated_from(1u32))?;
        ProvenancePallet::<T>::set_definition_active(origin.clone(),registry_id,definition_id)?;
        ProvenancePallet::<T>::create_process(attestor_origin,registry_id,definition_id,vec![42u8])?;

        let process_id=T::ProcessId::unique_saturated_from(1u32);

        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,process_id,name.clone())

    verify {
        let process=<Processes<T>>::get((registry_id, definition_id),process_id);
        assert!(process.is_some());
        let process=process.unwrap();
        assert_eq!(process.name.len(),name.len());
    }

    remove_process {
        let a in 1 .. (<T as Config>::DefinitionStepLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);
        let attestor:T::AccountId = whitelisted_caller();
        let attestor_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(attestor.clone()).into();

        for i in 0..a {
            ProvenancePallet::<T>::create_definition_step(origin.clone(),registry_id,definition_id,T::DefinitionStepIndex::unique_saturated_from(i),
            vec![42u8],Some(caller.clone()),T::MemberCount::unique_saturated_from(1u32))?;
        }
        ProvenancePallet::<T>::set_definition_active(origin.clone(),registry_id,definition_id)?;

        ProvenancePallet::<T>::create_process(attestor_origin,registry_id,definition_id,vec![42u8])?;

        let process_id=T::ProcessId::unique_saturated_from(1u32);


    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,process_id)

    verify {
        assert!(!<Processes<T>>::contains_key((registry_id, definition_id),process_id));
        assert_eq!(<ProcessSteps<T>>::iter_prefix((registry_id, definition_id,process_id)).count(),0 as usize);
    }

    update_process_step {
        let a in 1 .. (<T as Config>::AttributeLimit::get()-1);
        let b in 1 .. (<T as Config>::NameLimit::get()-1);
        let c in 1 .. (<T as Config>::FactStringLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);
        let attestor:T::AccountId = whitelisted_caller();
        let attestor_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(attestor.clone()).into();
        ProvenancePallet::<T>::create_definition_step(origin.clone(),registry_id,definition_id,definition_step_index,vec![42u8],Some(attestor),T::MemberCount::unique_saturated_from(1u32))?;
        ProvenancePallet::<T>::set_definition_active(origin.clone(),registry_id,definition_id)?;
        ProvenancePallet::<T>::create_process(attestor_origin,registry_id,definition_id,vec![42u8])?;

        let process_id=T::ProcessId::unique_saturated_from(1u32);

        let mut attributes=Vec::new();
        for _ in 0..a {
            attributes.push(Attribute{
                name:vec![42u8; b as usize],
                fact:Fact::Text(vec![42u8; c as usize]),
            })
        }


    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,process_id,definition_step_index,attributes)

    verify {
        let process_step=<ProcessSteps<T>>::get((registry_id, definition_id,process_id),definition_step_index);
        assert!(process_step.is_some());
        let process_step=process_step.unwrap();
        assert_eq!(process_step.attributes.len(),a as usize);
    }

    //TODO: test both possible code paths.
    attest_process_step {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let attestor:T::AccountId = whitelisted_caller();
        let attestor_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(attestor.clone()).into();
        ProvenancePallet::<T>::create_definition_step(origin.clone(),registry_id,definition_id,T::DefinitionStepIndex::unique_saturated_from(0u32),vec![42u8],Some(attestor.clone()),T::MemberCount::unique_saturated_from(1u32))?;
        ProvenancePallet::<T>::create_definition_step(origin.clone(),registry_id,definition_id,T::DefinitionStepIndex::unique_saturated_from(1u32),vec![42u8],Some(attestor),T::MemberCount::unique_saturated_from(1u32))?;
        ProvenancePallet::<T>::set_definition_active(origin.clone(),registry_id,definition_id)?;
        ProvenancePallet::<T>::create_process(attestor_origin,registry_id,definition_id,vec![42u8])?;

        let process_id=T::ProcessId::unique_saturated_from(1u32);

        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,process_id,definition_step_index)

    verify {
        let process_step=<ProcessSteps<T>>::get((registry_id, definition_id,process_id),definition_step_index);
        assert!(process_step.is_some());
        let process_step=process_step.unwrap();
        assert!(process_step.attested);
    }

}

impl_benchmark_test_suite!(
    ProvenancePallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
