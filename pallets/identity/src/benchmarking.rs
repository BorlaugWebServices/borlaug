//! Identity pallet benchmarking.

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
use crate::Pallet as IdentityPallet;

type BalanceOf<T> =
    <<T as groups::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

fn create_properties(
    property_count: u32,
    property_name_len: u32,
    property_fact_len: u32,
    property_set: u8, //make this unique over different calls to ensure unique property names.
) -> Vec<DidProperty<Vec<u8>, Vec<u8>>> {
    let mut properties = Vec::new();
    assert!(property_name_len >= 5);
    for x in 0..property_count {
        //using x for first 4 chars ensures name is unique across returned vec
        let mut name = x.to_le_bytes().to_vec();
        name.push(property_set);
        name.extend(vec![42u8; (property_name_len - 4) as usize]);
        properties.push(DidProperty {
            name,
            fact: Fact::Text(vec![42u8; property_fact_len as usize]),
        });
    }
    properties
}

fn create_statements(
    statement_count: u32,
    statement_name_len: u32,
    statement_fact_len: u32,
    statement_set: u8, //make this unique over different calls to ensure unique statement names.
) -> Vec<Statement<Vec<u8>, Vec<u8>>> {
    let mut statements = Vec::new();
    assert!(statement_name_len >= 5);
    for x in 0..statement_count {
        //using x for first 4 chars ensures name is unique across returned vec
        let mut name = x.to_le_bytes().to_vec();
        name.push(statement_set);
        name.extend(vec![42u8; (statement_name_len - 4) as usize]);
        statements.push(Statement {
            name,
            fact: Fact::Text(vec![42u8; statement_fact_len as usize]),
            for_issuer: true,
        });
    }
    statements
}

//provide different seed to get new unique set
fn create_accounts<T: Config>(n: u32, seed: u32) -> Vec<T::AccountId> {
    let mut accounts = vec![];
    for i in 0..n {
        accounts.push(account("account", i, seed));
    }
    accounts
}

benchmarks! {
    register_did {
        let a in 5 .. <T as Config>::NameLimit::get();//property name length
        let b in 1 .. <T as Config>::FactStringLimit::get();//property fact length
        let c in 1 .. <T as Config>::PropertyLimit::get();//property count

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let properties=create_properties(c,a,b,1);

    }: _(SystemOrigin::Signed(caller.clone()), Some(properties))

    verify {
        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let mut dids_by_subject=Vec::new();
        <DidBySubject<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_subject.push(did);
        });
        assert_eq!(dids_by_subject.len(), 1);
        assert_eq!(<DidDocumentProperties<T>>::iter_prefix(&dids_by_subject[0]).count(), c as usize);
    }

    register_did_for {
        let a in 5 .. <T as Config>::NameLimit::get();//property name length
        let b in 1 .. <T as Config>::FactStringLimit::get();//property fact length
        let c in 1 .. <T as Config>::PropertyLimit::get();//property count

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let subject:<T as frame_system::Config>::AccountId = whitelisted_caller();

        let properties=create_properties(c,a,b,1);

    }: _(SystemOrigin::Signed(caller.clone()),subject.clone(), Some(properties))

    verify {
        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let mut dids_by_subject=Vec::new();
        <DidByController<T>>::iter_prefix(&subject).for_each(|(did, _)| {
            dids_by_subject.push(did);
        });
        assert_eq!(dids_by_subject.len(), 1);
        assert_eq!(<DidDocumentProperties<T>>::iter_prefix(&dids_by_subject[0]).count(), c as usize);
    }

    add_did_properties {
        let a in 5 .. <T as Config>::NameLimit::get();
        let b in 1 .. <T as Config>::FactStringLimit::get();
        let c in 1 .. <T as Config>::PropertyLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::register_did(origin,None)?;

        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        let properties=create_properties(c,a,b,2);
        assert_eq!(properties.len(), c as usize);

    }: _(SystemOrigin::Signed(caller.clone()),did, properties.clone())

    verify {
        assert!(<DidDocuments<T>>::contains_key(&did));
        assert_eq!(<DidDocumentProperties<T>>::iter_prefix(&did).count(), c as usize);
    }

    remove_did_properties {
        let a in 5 .. <T as Config>::NameLimit::get();
        let b in 1 .. <T as Config>::PropertyLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        //these will be removed
        let origional_properties=create_properties(b,a,<T as Config>::FactStringLimit::get()-1,1);
        assert_eq!(origional_properties.len(),b as usize);

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::register_did(origin,Some(origional_properties.clone()))?;

        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        assert_eq!(<DidDocumentProperties<T>>::iter_prefix(&did).count(), b as usize);

        let remove_keys=origional_properties.into_iter().map(|property|property.name).collect();

    }: _(SystemOrigin::Signed(caller.clone()),did, remove_keys)

    verify {
        assert!(<DidDocuments<T>>::contains_key(&did));
        assert_eq!(<DidDocumentProperties<T>>::iter_prefix(&did).count(),0);
    }

    manage_controllers {
        let a in 1 .. <T as Config>::ControllerLimit::get(); //origional controller count (will be removed)
        let b in 1 .. <T as Config>::ControllerLimit::get(); //add controller count

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        //these will be removed.
        let origional_controllers=create_accounts::<T>(a,1);

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::register_did(origin.clone(),None)?;

        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        IdentityPallet::<T>::manage_controllers(origin,did,Some(origional_controllers.clone()),None)?;

        let add_controllers=create_accounts::<T>(b,2);

    }: _(SystemOrigin::Signed(caller.clone()),did,  Some(add_controllers.clone()),Some(origional_controllers.clone()))

    verify {
        let mut stored_controllers=Vec::new();
        <DidControllers<T>>::iter_prefix(&did).for_each(|(_, controller)| {
            stored_controllers.push(controller);
        });
        assert_eq!(stored_controllers.len(), add_controllers.len()+1);
    }

    authorize_claim_consumers {
        let a in 1 .. <T as Config>::ClaimConsumerLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::register_did(origin.clone(),None)?;

        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        let claim_consumers=create_accounts::<T>(a,1);
        let now= <timestamp::Module<T>>::get();
        let now=now /(T::Moment::unique_saturated_from(1_000u32)) ;
        let claim_consumers:Vec<ClaimConsumer<T::AccountId,T::Moment>>=claim_consumers.into_iter().map(|account| ClaimConsumer{consumer: account,expiration:  now}).collect();

    }: _(SystemOrigin::Signed(caller.clone()),did,  claim_consumers.clone())

    verify {
        let mut stored_consumers=Vec::new();
        <ClaimConsumers<T>>::iter_prefix(&did).for_each(|(_, consumer)| {
            stored_consumers.push(consumer);
        });
        assert_eq!(stored_consumers.len(), claim_consumers.len());
    }

    revoke_claim_consumers {
        let a in 1 .. <T as Config>::ClaimConsumerLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::register_did(origin.clone(),None)?;

        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        let claim_consumers=create_accounts::<T>(a,1);
        let now= <timestamp::Module<T>>::get();
        let now=now /(T::Moment::unique_saturated_from(1_000u32)) ;
        let claim_consumers_to_add:Vec<ClaimConsumer<T::AccountId,T::Moment>>=claim_consumers.clone().into_iter().map(|account| ClaimConsumer{consumer: account,expiration: now}).collect();

        IdentityPallet::<T>::authorize_claim_consumers(origin.clone(),did,  claim_consumers_to_add)?;

    }: _(SystemOrigin::Signed(caller.clone()),did,  claim_consumers)

    verify {
        let mut stored_consumers=Vec::new();
        <ClaimConsumers<T>>::iter_prefix(&did).for_each(|(_, consumer)| {
            stored_consumers.push(consumer);
        });
        assert_eq!(stored_consumers.len(), 0);
    }

    authorize_claim_issuers {
        let a in 1 .. <T as Config>::ClaimIssuerLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::register_did(origin.clone(),None)?;

        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        let claim_issuers=create_accounts::<T>(a,1);
        let now= <timestamp::Module<T>>::get();
        let now=now /(T::Moment::unique_saturated_from(1_000u32)) ;
        let claim_issuers:Vec<ClaimIssuer<T::AccountId,T::Moment>>=claim_issuers.into_iter().map(|account| ClaimIssuer{issuer: account,expiration: now}).collect();

    }: _(SystemOrigin::Signed(caller.clone()),did,  claim_issuers.clone())

    verify {
        let mut stored_issuers=Vec::new();
        <ClaimIssuers<T>>::iter_prefix(&did).for_each(|(_, issuer)| {
            stored_issuers.push(issuer);
        });
        assert_eq!(stored_issuers.len(), claim_issuers.len());
    }

    revoke_claim_issuers {
        let a in 1 .. <T as Config>::ClaimIssuerLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::register_did(origin.clone(),None)?;

        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        let claim_issuers=create_accounts::<T>(a,1);
        let now= <timestamp::Module<T>>::get();
        let now=now /(T::Moment::unique_saturated_from(1_000u32)) ;
        let claim_issuers_to_add:Vec<ClaimIssuer<T::AccountId,T::Moment>>=claim_issuers.clone().into_iter().map(|account| ClaimIssuer{issuer: account,expiration: now}).collect();

        IdentityPallet::<T>::authorize_claim_issuers(origin.clone(),did,  claim_issuers_to_add)?;

    }: _(SystemOrigin::Signed(caller.clone()),did,  claim_issuers)

    verify {
        let mut stored_issuers=Vec::new();
        <ClaimIssuers<T>>::iter_prefix(&did).for_each(|(_, issuer)| {
            stored_issuers.push(issuer);
        });
        assert_eq!(stored_issuers.len(), 0);
    }

    make_claim {
        let a in 1 .. <T as Config>::NameLimit::get();
        let b in 1 .. <T as Config>::StatementLimit::get();
        let c in 5 .. <T as Config>::NameLimit::get();
        let d in 1 .. <T as Config>::FactStringLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::register_did(origin.clone(),None)?;

        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        let consumer:T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let claim_consumers=vec![consumer.clone()];
        let now= <timestamp::Module<T>>::get();
        let now_plus=now /(T::Moment::unique_saturated_from(1_000u32))+T::Moment::unique_saturated_from(1_000_000u32) ;
        let claim_consumers_to_add:Vec<ClaimConsumer<T::AccountId,T::Moment>>=claim_consumers.clone().into_iter().map(|account| ClaimConsumer{consumer: account,expiration: now_plus}).collect();

        IdentityPallet::<T>::authorize_claim_consumers(origin.clone(),did,  claim_consumers_to_add)?;

        let description=vec![42u8; a as usize];

        let statements=create_statements(b,c,d,1);

        let threshold=T::MemberCount::unique_saturated_from(1u32);

    }: _(SystemOrigin::Signed(consumer.clone()),did,  description,statements,threshold)

    verify {
        let mut claims=Vec::new();
        <Claims<T>>::iter_prefix(&did).for_each(|(claim_id,claim)| {
            claims.push(claim);
        });
        assert_eq!(claims.len(), 1);
        let claim=&claims[0];
        assert_eq!(claim.description.len() , a as usize );
        assert_eq!(claim.statements.len(), b as usize);
    }



    attest_claim {
        let a in 1 .. <T as Config>::StatementLimit::get();   //existing statement
        let b in 1 .. <T as Config>::StatementLimit::get();   //additional statements
        let c in 5 .. <T as Config>::NameLimit::get();
        let d in 1 .. <T as Config>::FactStringLimit::get();

        //TODO:test with group attestation as that has an extra db read

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::register_did(origin.clone(),None)?;

        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        let consumer = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let claim_consumers=vec![consumer];
        let now= <timestamp::Module<T>>::get();
        let now_plus=now /(T::Moment::unique_saturated_from(1_000u32))+T::Moment::unique_saturated_from(1_000_000u32) ;
        let claim_consumers_to_add:Vec<ClaimConsumer<T::AccountId,T::Moment>>=claim_consumers.clone().into_iter().map(|account| ClaimConsumer{consumer: account,expiration: now_plus}).collect();
        IdentityPallet::<T>::authorize_claim_consumers(origin.clone(),did,  claim_consumers_to_add)?;

        let existing_statements=create_statements(a,5,5,1);
        let threshold=T::MemberCount::unique_saturated_from(1u32);

        IdentityPallet::<T>::make_claim(origin.clone(),did,vec![42u8],  existing_statements,threshold)?;

        let mut claims=Vec::new();
        <Claims<T>>::iter_prefix(&did).for_each(|(claim_id,claim)| {
            claims.push((claim_id,claim));
        });
        assert_eq!(claims.len(), 1);
        let (claim_id,claim)=claims[0].clone();

        let issuer:T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let claim_issuers=vec![issuer.clone()];
        let now= <timestamp::Module<T>>::get();
        let now_plus=now /(T::Moment::unique_saturated_from(1_000u32))+T::Moment::unique_saturated_from(1_000_000u32) ;

        let claim_issuers_to_add:Vec<ClaimIssuer<T::AccountId,T::Moment>>=claim_issuers.clone().into_iter().map(|account| ClaimIssuer{issuer: account,expiration: now_plus}).collect();
        IdentityPallet::<T>::authorize_claim_issuers(origin.clone(),did,  claim_issuers_to_add)?;

        let attestor_statements=create_statements(b,c,d,2);

    }: _(SystemOrigin::Signed(issuer.clone()),did, claim_id, attestor_statements,now_plus)

    verify {
        let claim=<Claims<T>>::get(did, claim_id);
        assert!(claim.is_some());
        let claim=claim.unwrap();
        assert_eq!(claim.statements.len(), (a+b) as usize);
        assert!(claim.attestation.is_some());
    }

    revoke_attestation {
        let a in 1 .. <T as Config>::StatementLimit::get();   //existing statement
        let b in 5 .. <T as Config>::NameLimit::get();
        let c in 1 .. <T as Config>::FactStringLimit::get();

        //TODO:test with group attestation as that has an extra db read

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::register_did(origin.clone(),None)?;

        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), 1);
        let did=dids_by_controller[0];

        let consumer = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let claim_consumers=vec![consumer];
        let now= <timestamp::Module<T>>::get();
        let now_plus=now /(T::Moment::unique_saturated_from(1_000u32))+T::Moment::unique_saturated_from(1_000_000u32) ;
        let claim_consumers_to_add:Vec<ClaimConsumer<T::AccountId,T::Moment>>=claim_consumers.clone().into_iter().map(|account| ClaimConsumer{consumer: account,expiration: now_plus}).collect();
        IdentityPallet::<T>::authorize_claim_consumers(origin.clone(),did,  claim_consumers_to_add)?;

        let existing_statements=create_statements(a,b,c,1);
        let threshold=T::MemberCount::unique_saturated_from(1u32);

        IdentityPallet::<T>::make_claim(origin.clone(),did,vec![42u8],  existing_statements,threshold)?;

        let mut claims=Vec::new();
        <Claims<T>>::iter_prefix(&did).for_each(|(claim_id,claim)| {
            claims.push((claim_id,claim));
        });
        assert_eq!(claims.len(), 1);
        let (claim_id,claim)=claims[0].clone();

        let issuer:T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let claim_issuers=vec![issuer.clone()];
        let now= <timestamp::Module<T>>::get();
        let now_plus=now /(T::Moment::unique_saturated_from(1_000u32))+T::Moment::unique_saturated_from(1_000_000u32) ;

        let claim_issuers_to_add:Vec<ClaimIssuer<T::AccountId,T::Moment>>=claim_issuers.clone().into_iter().map(|account| ClaimIssuer{issuer: account,expiration: now_plus}).collect();
        IdentityPallet::<T>::authorize_claim_issuers(origin.clone(),did,  claim_issuers_to_add)?;

        let attestor_origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(issuer.clone()).into();
        IdentityPallet::<T>::attest_claim(attestor_origin.clone(),did,  claim_id, vec![],now_plus)?;

        let claim=<Claims<T>>::get(did, claim_id);
        assert!(claim.is_some());
        let claim=claim.unwrap();
        assert_eq!(claim.statements.len(), a as usize);
        assert!(claim.attestation.is_some());

    }: _(SystemOrigin::Signed(issuer.clone()),did, claim_id)

    verify {
        let claim=<Claims<T>>::get(did, claim_id);
        assert!(claim.is_some());
        let claim=claim.unwrap();
        assert_eq!(claim.statements.len(), a as usize);
        assert!(claim.attestation.is_none());
    }

    create_catalog {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

    }: _(SystemOrigin::Signed(caller.clone()))

    verify {
        let catalog_id=T::CatalogId::unique_saturated_from(1u32);
        assert!(<Catalogs<T>>::contains_key(caller,catalog_id));
    }

    remove_catalog {

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::create_catalog(origin.clone())?;

        let catalog_id=T::CatalogId::unique_saturated_from(1u32);
        assert!(<Catalogs<T>>::contains_key(caller.clone(),catalog_id));

    }: _(SystemOrigin::Signed(caller.clone()),catalog_id)

    verify {
        assert!(!<Catalogs<T>>::contains_key(caller,catalog_id));
    }

    add_dids_to_catalog {

        let a in 1 .. <T as Config>::CatalogDidLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::create_catalog(origin.clone())?;

        let catalog_id=T::CatalogId::unique_saturated_from(1u32);

        for _ in 0..a {
            IdentityPallet::<T>::register_did(origin.clone(),None)?;
        }
        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), a as usize);

    }: _(SystemOrigin::Signed(caller.clone()),catalog_id,dids_by_controller)

    verify {
        let mut catalog_dids=Vec::new();
        <DidsByCatalog<T>>::iter_prefix(&catalog_id).for_each(|(did,_)| {
            catalog_dids.push(did);
        });
        assert_eq!(catalog_dids.len(),a as usize);
    }

    remove_dids_from_catalog {

        let a in 1 .. <T as Config>::CatalogDidLimit::get();

        let caller = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

        let origin:<T as frame_system::Config>::Origin=SystemOrigin::Signed(caller.clone()).into();
        IdentityPallet::<T>::create_catalog(origin.clone())?;

        let catalog_id=T::CatalogId::unique_saturated_from(1u32);

        for _ in 0..a {
            IdentityPallet::<T>::register_did(origin.clone(),None)?;
        }
        let mut dids_by_controller=Vec::new();
        <DidByController<T>>::iter_prefix(&caller).for_each(|(did, _)| {
            dids_by_controller.push(did);
        });
        assert_eq!(dids_by_controller.len(), a as usize);

        IdentityPallet::<T>::add_dids_to_catalog(origin.clone(), catalog_id, dids_by_controller.clone())?;

    }: _(SystemOrigin::Signed(caller.clone()),catalog_id,dids_by_controller)

    verify {
        let mut dids_in_catalog=Vec::new();
        <DidsByCatalog<T>>::iter_prefix(&catalog_id).for_each(|(did, _)| {
            dids_in_catalog.push(did);
        });
        assert_eq!(dids_in_catalog.len(), 0 as usize);
    }

}

impl_benchmark_test_suite!(
    IdentityPallet,
    crate::mock::new_test_ext(),
    crate::mock::Test,
);
