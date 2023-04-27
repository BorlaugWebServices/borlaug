//! Audits pallet benchmarking.

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
use crate::Pallet as AssetRegistryPallet;

type BalanceOf<T> =
    <<T as groups::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

benchmarks! {
    create_registry {
        let a in 1 .. (<T as Config>::NameLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        identity::Pallet::<T>::register_did(origin, None)?;
        let mut dids_by_controller=Vec::new();
        <identity::DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(caller.clone()),did.clone(),name.clone())

    verify {
        let registry_id=T::RegistryId::unique_saturated_from(1u32);
        let registry=<Registries<T>>::get(did,registry_id);
        assert!(registry.is_some());
        assert_eq!(registry.unwrap().name.len(),name.len());
    }

    update_registry {
        let a in 1 .. (<T as Config>::NameLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        identity::Pallet::<T>::register_did(origin.clone(),None)?;
        let mut dids_by_controller=Vec::new();
        <identity::DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        AssetRegistryPallet::<T>::create_registry(origin, did.clone(),vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);

        let name = vec![42u8; a as usize];

    }: _(SystemOrigin::Signed(caller.clone()),did.clone(),registry_id,name.clone())

    verify {
        let registry=<Registries<T>>::get(did,registry_id);
        assert!(registry.is_some());
        assert_eq!(registry.unwrap().name.len(),name.len());
    }

    delete_registry {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        identity::Pallet::<T>::register_did(origin.clone(),None)?;
        let mut dids_by_controller=Vec::new();
        <identity::DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        AssetRegistryPallet::<T>::create_registry(origin, did.clone(),vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);

    }: _(SystemOrigin::Signed(caller.clone()),did.clone(),registry_id)

    verify {
        assert!(!<Registries<T>>::contains_key(did,registry_id));
    }

    create_asset {

        let a in 1 .. (<T as Config>::NameLimit::get()-1);
        let b in 1 .. (<T as Config>::NameLimit::get()-1);
        let c in 1 .. (<T as Config>::NameLimit::get()-1);
        let d in 1 .. (<T as Config>::NameLimit::get()-1);
        let e in 1 .. (<T as Config>::FactStringLimit::get()-1);
        let f in 1 .. (<T as Config>::AssetPropertyLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        identity::Pallet::<T>::register_did(origin.clone(), None)?;
        let mut dids_by_controller=Vec::new();
        <identity::DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        AssetRegistryPallet::<T>::create_registry(origin, did.clone(),vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);

        let mut properties = vec![];
        for i in 0 .. f {
            let property = AssetProperty{
                name:vec![42u8; d as usize],
                fact:Fact::Text(vec![42u8; e as usize]),
            };
            properties.push(property);
        }

        let asset=Asset{
            properties:properties,
            name: vec![42u8; a as usize],
            asset_number: Some(vec![42u8; b as usize]),
            status: AssetStatus::Draft,
            serial_number: Some(vec![42u8; c as usize]),
            total_shares: 100u64,
            residual_value: Some(<T as pallet::Config>::Balance::unique_saturated_from(100u64)),
            purchase_value: Some(<T as pallet::Config>::Balance::unique_saturated_from(100u64)),
            acquired_date: Some(T::Moment::unique_saturated_from(100u32)),
        };

    }: _(SystemOrigin::Signed(caller.clone()),did.clone(),registry_id,asset.clone())

    verify {
        let asset_id=T::AssetId::unique_saturated_from(1u32);
        let stored_asset=<Assets<T>>::get(registry_id,asset_id);
        assert!(stored_asset.is_some());
        let stored_asset=stored_asset.unwrap();
        assert_eq!(stored_asset.name.len(),asset.name.len());
    }

    update_asset {

        let a in 1 .. (<T as Config>::NameLimit::get()-1);
        let b in 1 .. (<T as Config>::NameLimit::get()-1);
        let c in 1 .. (<T as Config>::NameLimit::get()-1);
        let d in 1 .. (<T as Config>::NameLimit::get()-1);
        let e in 1 .. (<T as Config>::FactStringLimit::get()-1);
        let f in 1 .. (<T as Config>::AssetPropertyLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        identity::Pallet::<T>::register_did(origin.clone(), None)?;
        let mut dids_by_controller=Vec::new();
        <identity::DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        AssetRegistryPallet::<T>::create_registry(origin.clone(), did.clone(),vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);

        AssetRegistryPallet::<T>::create_asset(origin, did.clone(),registry_id,Asset{
            properties:vec![],
            name:vec![],
            asset_number: None,
            status: AssetStatus::Draft,
            serial_number:None,
            total_shares:100u64,
            residual_value: None,
            purchase_value: None,
            acquired_date: None,
        })?;
        let asset_id=T::AssetId::unique_saturated_from(1u32);

        let mut properties = vec![];
        for i in 0 .. f {
            let property = AssetProperty{
                name:vec![42u8; d as usize],
                fact:Fact::Text(vec![42u8; e as usize]),
            };
            properties.push(property);
        }

        let asset=Asset{
            properties:properties,
            name: vec![42u8; a as usize],
            asset_number: Some(vec![42u8; b as usize]),
            status: AssetStatus::Draft,
            serial_number: Some(vec![42u8; c as usize]),
            total_shares: 100u64,
            residual_value: Some(<T as pallet::Config>::Balance::unique_saturated_from(100u64)),
            purchase_value: Some(<T as pallet::Config>::Balance::unique_saturated_from(100u64)),
            acquired_date: Some(T::Moment::unique_saturated_from(100u32)),
        };

    }: _(SystemOrigin::Signed(caller.clone()),did.clone(),registry_id,asset_id,asset.clone())

    verify {
        let stored_asset=<Assets<T>>::get(registry_id,asset_id);
        assert!(stored_asset.is_some());
        let stored_asset=stored_asset.unwrap();
        assert_eq!(stored_asset.name.len(),asset.name.len());
    }

    delete_asset {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        identity::Pallet::<T>::register_did(origin.clone(), None)?;
        let mut dids_by_controller=Vec::new();
        <identity::DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        AssetRegistryPallet::<T>::create_registry(origin.clone(), did.clone(),vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);

        AssetRegistryPallet::<T>::create_asset(origin, did.clone(),registry_id,Asset{
            properties:vec![],
            name:vec![],
            asset_number: None,
            status: AssetStatus::Draft,
            serial_number:None,
            total_shares:100u64,
            residual_value: None,
            purchase_value: None,
            acquired_date: None,
        })?;
        let asset_id=T::AssetId::unique_saturated_from(1u32);

    }: _(SystemOrigin::Signed(caller.clone()),did.clone(),registry_id,asset_id)

    verify {
        assert!(!<Assets<T>>::contains_key(registry_id,asset_id));
    }


    new_lease {

        let a in 1 .. (<T as Config>::NameLimit::get()-1);
        let b in 1 .. (<T as Config>::LeaseAssetLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        identity::Pallet::<T>::register_did(origin.clone(), None)?;
        let mut dids_by_controller=Vec::new();
        <identity::DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        AssetRegistryPallet::<T>::create_registry(origin.clone(), did.clone(),vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);

        let asset=Asset{
            properties:vec![],
            name:vec![],
            asset_number: None,
            status: AssetStatus::Draft,
            serial_number:None,
            total_shares:100u64,
            residual_value: None,
            purchase_value: None,
            acquired_date: None,
        };

        let mut allocations=vec![];
        for i in 0..b {
            AssetRegistryPallet::<T>::create_asset(origin.clone(), did.clone(),registry_id,asset.clone())?;
            let asset_id=T::AssetId::unique_saturated_from(i+1);
            assert!(<Assets<T>>::contains_key(registry_id,asset_id));
            allocations.push(AssetAllocation{
                registry_id,
                asset_id,
                allocated_shares:10u64
            })
        }

        let lease=LeaseAgreement{
            proposal_id:None,
            contract_number: vec![42u8; a as usize],
            lessor: did.clone(),
            lessee: did.clone(),
            effective_ts: T::Moment::unique_saturated_from(100u32),
            expiry_ts: T::Moment::unique_saturated_from(100u32),
            allocations: allocations,
        };

    }: _(SystemOrigin::Signed(caller.clone()),lease.clone())

    verify {
        let lease_id=T::LeaseId::unique_saturated_from(1u32);
        let stored_lease=<LeaseAgreements<T>>::get(did,lease_id);
        assert!(stored_lease.is_some());
        let stored_lease=stored_lease.unwrap();
        assert_eq!(stored_lease.contract_number.len(),lease.contract_number.len());
    }

    void_lease {

        let a in 1 .. (<T as Config>::LeaseAssetLimit::get()-1);

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::RuntimeOrigin=SystemOrigin::Signed(caller.clone()).into();
        identity::Pallet::<T>::register_did(origin.clone(), None)?;
        let mut dids_by_controller=Vec::new();
        <identity::DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        AssetRegistryPallet::<T>::create_registry(origin.clone(), did.clone(),vec![42u8])?;
        let registry_id=T::RegistryId::unique_saturated_from(1u32);

        let asset=Asset{
            properties:vec![],
            name:vec![],
            asset_number: None,
            status: AssetStatus::Draft,
            serial_number:None,
            total_shares:100u64,
            residual_value: None,
            purchase_value: None,
            acquired_date: None,
        };

        let mut allocations=vec![];
        for i in 0..a {
            AssetRegistryPallet::<T>::create_asset(origin.clone(), did.clone(),registry_id,asset.clone())?;
            let asset_id=T::AssetId::unique_saturated_from(i+1);
            assert!(<Assets<T>>::contains_key(registry_id,asset_id));
            allocations.push(AssetAllocation{
                registry_id,
                asset_id,
                allocated_shares:10u64
            })
        }

        let lease=LeaseAgreement{
            proposal_id:None,
            contract_number: vec![42u8],
            lessor: did.clone(),
            lessee: did.clone(),
            effective_ts: T::Moment::unique_saturated_from(100u32),
            expiry_ts: T::Moment::unique_saturated_from(100u32),
            allocations: allocations,
        };

        AssetRegistryPallet::<T>::new_lease(origin.clone(), lease.clone())?;
        let lease_id=T::LeaseId::unique_saturated_from(1u32);

    }: _(SystemOrigin::Signed(caller.clone()),did,lease_id)

    verify {
        assert!(!<LeaseAgreements<T>>::contains_key(did,lease_id));
    }
}

impl_benchmark_test_suite!(
    AssetRegistryPallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
