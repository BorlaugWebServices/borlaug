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
        let a in 1 .. <T as Config>::NameLimit::get();

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
        let a in 1 .. <T as Config>::NameLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
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

        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;

        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

    }: _(SystemOrigin::Signed(caller.clone()),registry_id)

    verify {
        assert!(!<Registries<T>>::contains_key(caller.clone(),registry_id));
    }

    create_definition {
        let a in 1 .. <T as Config>::NameLimit::get(); //definition name
        let b in 1 .. <T as Config>::NameLimit::get(); //step name
        let c in 1 .. <T as Config>::DefinitionStepLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let name = vec![42u8; a as usize];

        let mut steps=vec![];
        for i in 0..c {
                let name = vec![42u8; b as usize];
                let attestor = whitelisted_caller();
                let required=true;
                let threshold=T::MemberCount::unique_saturated_from(1u32);
                steps.push((name,attestor,required,threshold));
        }

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,name,steps)

    verify {
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let definition=<Definitions<T>>::get(registry_id, definition_id);
        assert!(definition.is_some());
        let definition=definition.unwrap();
        assert_eq!(definition.name.len(),a as usize);
        assert_eq!(<DefinitionSteps<T>>::iter_prefix((registry_id, definition_id)).count(),c as usize);
    }

    set_definition_active {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let name = vec![42u8; <T as Config>::NameLimit::get() as usize];
        let attestor = whitelisted_caller();
        let required=true;
        let threshold=T::MemberCount::unique_saturated_from(1u32);


        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8],vec![(name,attestor,required,threshold)])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);

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
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let name = vec![42u8; <T as Config>::NameLimit::get() as usize];
        let attestor = whitelisted_caller();
        let required=true;
        let threshold=T::MemberCount::unique_saturated_from(1u32);

        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8],vec![(name,attestor,required,threshold)])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id)

    verify {
        let definition=<Definitions<T>>::get(registry_id, definition_id);
        assert!(definition.is_some());
        let definition=definition.unwrap();
        assert_eq!(definition.status,DefinitionStatus::Inactive);
    }


    remove_definition {

        let a in 1 .. <T as Config>::DefinitionStepLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let mut steps=vec![];
        for i in 0..a {
                let name = vec![42u8; <T as Config>::NameLimit::get() as usize];
                let attestor = whitelisted_caller();
                let required=true;
                let threshold=T::MemberCount::unique_saturated_from(1u32);
                steps.push((name,attestor,required,threshold));
        }

        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8],steps)?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);


    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id)

    verify {
        assert!(!<Definitions<T>>::contains_key(registry_id, definition_id));
        assert!(!<DefinitionSteps<T>>::contains_key((registry_id, definition_id),T::DefinitionStepIndex::unique_saturated_from(0u32)));
    }

    update_definition_step {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));


        let name = vec![42u8; <T as Config>::NameLimit::get() as usize];
        let attestor = whitelisted_caller();
        let required=true;
        let threshold=T::MemberCount::unique_saturated_from(1u32);

        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8],vec![(name,attestor,required,threshold)])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);

        let attestor:T::AccountId = whitelisted_caller();
        let threshold=T::MemberCount::unique_saturated_from(1u32);

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,definition_step_index,Some(attestor.clone()),Some(threshold))

    verify {
        let definition_step=<DefinitionSteps<T>>::get((registry_id, definition_id),definition_step_index);
        assert!(definition_step.is_some());
        let definition_step=definition_step.unwrap();
        assert_eq!(definition_step.attestor,attestor);
        assert_eq!(definition_step.threshold,threshold);
    }

    create_process {
        let a in 1 .. <T as Config>::NameLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let mut steps=vec![];
        for i in 0..2 {
                let name = vec![42u8; <T as Config>::NameLimit::get() as usize];
                let attestor = whitelisted_caller();
                let required=true;
                let threshold=T::MemberCount::unique_saturated_from(1u32);
                steps.push((name,attestor,required,threshold));
        }
        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8],steps)?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let attestor:T::AccountId = whitelisted_caller();

        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(attestor.clone()),registry_id,definition_id,name.clone())

    verify {
        let process_id=T::ProcessId::unique_saturated_from(1u32);
        let process=<Processes<T>>::get((registry_id, definition_id),process_id);
        assert!(process.is_some());
        let process=process.unwrap();
        assert_eq!(process.name.len(),name.len());
        assert_eq!(process.status,ProcessStatus::InProgress);

    }

    update_process {
        let a in 1 .. <T as Config>::NameLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let mut steps=vec![];
        for i in 0..2 {
                let name = vec![42u8; <T as Config>::NameLimit::get() as usize];
                let attestor = whitelisted_caller();
                let required=true;
                let threshold=T::MemberCount::unique_saturated_from(1u32);
                steps.push((name,attestor,required,threshold));
        }

        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8],steps)?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);
        let attestor:T::AccountId = whitelisted_caller();
        let attestor_origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(attestor.clone()).into();


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
        let a in 1 .. <T as Config>::DefinitionStepLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let mut steps=vec![];
        for i in 0..a {
                let name = vec![42u8; <T as Config>::NameLimit::get() as usize];
                let attestor = whitelisted_caller();
                let required=true;
                let threshold=T::MemberCount::unique_saturated_from(1u32);
                steps.push((name,attestor,required,threshold));
        }

        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8],steps)?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);
        let attestor:T::AccountId = whitelisted_caller();
        let attestor_origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(attestor.clone()).into();



        ProvenancePallet::<T>::set_definition_active(origin.clone(),registry_id,definition_id)?;

        ProvenancePallet::<T>::create_process(attestor_origin,registry_id,definition_id,vec![42u8])?;

        let process_id=T::ProcessId::unique_saturated_from(1u32);


    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,process_id)

    verify {
        assert!(!<Processes<T>>::contains_key((registry_id, definition_id),process_id));
        assert_eq!(<ProcessSteps<T>>::iter_prefix((registry_id, definition_id,process_id)).count(),0 as usize);
    }


    add_child_definition {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let name = vec![42u8; <T as Config>::NameLimit::get() as usize];
        let attestor:T::AccountId = whitelisted_caller();
        let required=true;
        let threshold=T::MemberCount::unique_saturated_from(1u32);

        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8],vec![(name.clone(),attestor.clone(),required,threshold)])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        assert!(<Definitions<T>>::contains_key(registry_id,definition_id));
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let child_registry_id=T::RegistryId::unique_saturated_from(2u32);
        ProvenancePallet::<T>::create_definition(origin.clone(),child_registry_id,vec![42u8],vec![(name,attestor,required,threshold)])?;
        let child_definition_id=T::DefinitionId::unique_saturated_from(2u32);
        assert!(<Definitions<T>>::contains_key(child_registry_id,child_definition_id));
    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,child_registry_id,child_definition_id)

    verify {
        let definition_child_maybe=<DefinitionChildren<T>>::get((registry_id, definition_id),child_definition_id);
        assert!(definition_child_maybe.is_some());
        assert_eq!(definition_child_maybe.unwrap(),child_registry_id);
        let definition_parent_maybe=<DefinitionParents<T>>::get((child_registry_id, child_definition_id),definition_id);
        assert!(definition_parent_maybe.is_some());
        assert_eq!(definition_parent_maybe.unwrap(),registry_id);
    }

    remove_child_definition {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let name = vec![42u8; <T as Config>::NameLimit::get() as usize];
        let attestor:T::AccountId = whitelisted_caller();
        let required=true;
        let threshold=T::MemberCount::unique_saturated_from(1u32);

        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8],vec![(name.clone(),attestor.clone(),required,threshold)])?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        assert!(<Definitions<T>>::contains_key(registry_id,definition_id));
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let child_registry_id=T::RegistryId::unique_saturated_from(2u32);
        ProvenancePallet::<T>::create_definition(origin.clone(),child_registry_id,vec![42u8],vec![(name,attestor,required,threshold)])?;
        let child_definition_id=T::DefinitionId::unique_saturated_from(2u32);
        assert!(<Definitions<T>>::contains_key(child_registry_id,child_definition_id));

        ProvenancePallet::<T>::add_child_definition(origin.clone(),registry_id,definition_id,child_registry_id,child_definition_id)?;

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,child_registry_id,child_definition_id)

    verify {
        assert!(!<DefinitionChildren<T>>::contains_key((registry_id, definition_id),child_definition_id));
        assert!(!<DefinitionParents<T>>::contains_key((child_registry_id, child_definition_id),definition_id));
    }



    //TODO: test different possible code paths.
    attest_process_step {

        let a in 1 .. <T as Config>::AttributeLimit::get();
        let b in 1 .. <T as Config>::NameLimit::get();
        let c in 1 .. <T as Config>::FactStringLimit::get();



        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let mut steps=vec![];
        for i in 0..2 {
                let name = vec![42u8; <T as Config>::NameLimit::get() as usize];
                let attestor = whitelisted_caller();
                let required=true;
                let threshold=T::MemberCount::unique_saturated_from(1u32);
                steps.push((name,attestor,required,threshold));
        }

        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8],steps)?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);
        let attestor:T::AccountId = whitelisted_caller();
        let attestor_origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(attestor.clone()).into();

        ProvenancePallet::<T>::create_process(attestor_origin,registry_id,definition_id,vec![42u8])?;

        let process_id=T::ProcessId::unique_saturated_from(1u32);

        let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(0u32);

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


    complete_process {

        let a in 2 .. <T as Config>::DefinitionStepLimit::get();

        let caller:T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        ProvenancePallet::<T>::create_registry(origin.clone(), vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        assert!(<Registries<T>>::contains_key(caller.clone(),registry_id));

        let mut steps=vec![];
        for i in 0..a {
                let name = vec![42u8; <T as Config>::NameLimit::get() as usize];
                let required=false;
                let threshold=T::MemberCount::unique_saturated_from(1u32);
                steps.push((name,caller.clone(),required,threshold));
        }

        ProvenancePallet::<T>::create_definition(origin.clone(),registry_id,vec![42u8],steps)?;
        let definition_id=T::DefinitionId::unique_saturated_from(1u32);

        ProvenancePallet::<T>::create_process(origin.clone(),registry_id,definition_id,vec![42u8])?;

        let process_id=T::ProcessId::unique_saturated_from(1u32);

        for i in 0..(a-1) {
            let definition_step_index=T::DefinitionStepIndex::unique_saturated_from(i);
            ProvenancePallet::<T>::attest_process_step(origin.clone(),registry_id,definition_id,process_id,definition_step_index,vec![])?;
        }

        let process_maybe=<Processes<T>>::get((registry_id, definition_id),process_id);
        assert!(process_maybe.is_some());
        assert_eq!(process_maybe.unwrap().status,ProcessStatus::InProgress);

    }: _(SystemOrigin::Signed(caller.clone()),registry_id,definition_id,process_id)

    verify {
        let process_maybe=<Processes<T>>::get((registry_id, definition_id),process_id);
        assert!(process_maybe.is_some());
        assert_eq!(process_maybe.unwrap().status,ProcessStatus::Completed);
    }

}

impl_benchmark_test_suite!(
    ProvenancePallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
