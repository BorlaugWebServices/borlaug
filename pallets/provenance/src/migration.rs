use super::*;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use primitives::{bounded_vec::BoundedVec, DefinitionStep, ProcessStep};

pub mod deprecated {
    use codec::{Decode, Encode};
    use frame_support::dispatch::Vec;
    use primitives::Attribute;
    use sp_runtime::RuntimeDebug;

    // #[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
    // pub struct OldProcessStep<BoundedStringName, BoundedStringFact> {
    //     pub attributes: Vec<Attribute<BoundedStringName, BoundedStringFact>>,
    // }

    #[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
    pub struct OldProcessStep<ProposalId, BoundedStringName, BoundedStringFact> {
        pub proposal_id: Option<ProposalId>,
        pub attributes: Vec<Attribute<BoundedStringName, BoundedStringFact>>,
    }
    #[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
    pub struct OldDefinitionStep<AccountId, MemberCount, BoundedString> {
        pub name: BoundedString,
        pub attestor: AccountId,
        pub threshold: MemberCount,
    }
}

// #[allow(clippy::unnecessary_cast)]
// pub fn migrate_to_v2<T: Config>() -> Weight {
//     let mut weight: Weight = 0;

//     let storage_version_maybe = <StorageVersion<T>>::get();

//     if storage_version_maybe.is_none() || storage_version_maybe.unwrap() == Releases::V1 {
//         <ProcessSteps<T>>::translate::<
//             deprecated::OldProcessStep<
//                 BoundedVec<u8, <T as Config>::NameLimit>,
//                 BoundedVec<u8, <T as Config>::FactStringLimit>,
//             >,
//             _,
//         >(|(_, _, _), _, old| {
//             weight += T::DbWeight::get().reads_writes(1 as Weight, 1 as Weight);
//             let new = ProcessStep {
//                 proposal_id: None,
//                 attributes: old.attributes,
//             };
//             Some(new)
//         });

//         <StorageVersion<T>>::set(Some(Releases::V2));
//     } else {
//         frame_support::debug::info!(" >>> Unused migration!");
//     }

//     weight
// }

#[allow(clippy::unnecessary_cast)]
pub fn migrate_to_v3<T: Config>() -> Weight {
    let mut weight: Weight = 0;

    let storage_version_maybe = <StorageVersion<T>>::get();

    if storage_version_maybe.is_none() || storage_version_maybe.unwrap() == Releases::V2 {
        <ProcessSteps<T>>::translate::<
            deprecated::OldProcessStep<
                T::ProposalId,
                BoundedVec<u8, <T as Config>::NameLimit>,
                BoundedVec<u8, <T as Config>::FactStringLimit>,
            >,
            _,
        >(|(_, _, _), _, old| {
            weight += T::DbWeight::get().reads_writes(1 as Weight, 1 as Weight);
            let new = ProcessStep {
                proposal_id: old.proposal_id,
                attested: <timestamp::Module<T>>::get(),
                attributes: old.attributes,
            };
            Some(new)
        });

        <StorageVersion<T>>::set(Some(Releases::V3));
    } else {
        frame_support::debug::info!(" >>> Unused migration!");
    }

    weight
}

#[allow(clippy::unnecessary_cast)]
pub fn migrate_to_v4<T: Config>() -> Weight {
    let mut weight: Weight = 0;

    let storage_version_maybe = <StorageVersion<T>>::get();

    if storage_version_maybe.is_none() || storage_version_maybe.unwrap() == Releases::V3 {
        <DefinitionSteps<T>>::translate::<
            deprecated::OldDefinitionStep<
                T::AccountId,
                T::MemberCount,
                BoundedVec<u8, <T as Config>::NameLimit>,
            >,
            _,
        >(|(_, _), _, old| {
            weight += T::DbWeight::get().reads_writes(1 as Weight, 1 as Weight);
            let new = DefinitionStep {
                name: old.name,
                attestor: old.attestor,
                required: true,
                threshold: old.threshold,
            };
            Some(new)
        });

        <StorageVersion<T>>::set(Some(Releases::V4));
    } else {
        frame_support::debug::info!(" >>> Unused migration!");
    }

    weight
}
