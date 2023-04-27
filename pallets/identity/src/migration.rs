pub mod deprecated {
    use codec::{Decode, Encode};
    use frame_support::dispatch::Vec;
    use primitives::Statement;
    use sp_runtime::RuntimeDebug;

    #[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
    pub struct OldClaim<AccountId, MemberCount, Moment, BoundedStringName, BoundedStringFact> {
        /// A claim description
        pub description: BoundedStringName,
        /// Statements contained in this claim
        pub statements: Vec<Statement<BoundedStringName, BoundedStringFact>>,
        /// Claim consumer creates a claim
        pub created_by: AccountId,
        /// Attestation by claim verifier
        pub attestation: Option<OldAttestation<AccountId, Moment>>,
        /// Minimum number of votes required for attestation
        pub threshold: MemberCount,
    }

    #[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
    pub struct OldAttestation<AccountId, Timestamp> {
        /// Claim verifier attests a claim
        pub attested_by: AccountId,
        /// Attesttation valid until
        pub valid_until: Timestamp,
    }
}

// #[allow(clippy::unnecessary_cast)]
// pub fn migrate_to_v2<T: Config>() -> Weight {
//     let mut weight: Weight = 0;

//     let storage_version_maybe = <StorageVersion<T>>::get();

//     if storage_version_maybe.is_none() || storage_version_maybe.unwrap() == Releases::V1 {
//         frame_support::debug::info!(" >>> Migrating storage to V2");
//         <DidsByCatalog<T>>::iter().for_each(|(catalog_id, did, _)| {
//             <DidCatalogs<T>>::insert(&did, catalog_id, ());
//             weight += T::DbWeight::get().reads_writes(1 as Weight, 1 as Weight);
//         });
//         <StorageVersion<T>>::set(Some(Releases::V2));
//     } else {
//         frame_support::debug::info!(" >>> Unused migration!");
//     }
//     weight
// }

// #[allow(clippy::unnecessary_cast)]
// pub fn migrate_to_v3<T: Config>() -> Weight {
//     let mut weight: Weight = 0;

//     let storage_version_maybe = <StorageVersion<T>>::get();

//     if storage_version_maybe.is_some() && storage_version_maybe.unwrap() == Releases::V2 {
//         frame_support::debug::info!(" >>> Migrating storage to V3");
//         <Claims<T>>::translate::<
//             deprecated::OldClaim<
//                 T::AccountId,
//                 T::MemberCount,
//                 T::Moment,
//                 BoundedVec<u8, <T as Config>::NameLimit>,
//                 BoundedVec<u8, <T as Config>::FactStringLimit>,
//             >,
//             _,
//         >(|_, _, old| {
//             weight += T::DbWeight::get().reads_writes(1 as Weight, 1 as Weight);
//             let new = Claim {
//                 description: old.description,
//                 statements: old.statements,
//                 created_by: old.created_by,
//                 attestation: old.attestation.map(|old_attestation| Attestation {
//                     attested_by: old_attestation.attested_by,
//                     issued: <timestamp::Pallet<T>>::get(),
//                     valid_until: old_attestation.valid_until,
//                 }),
//                 threshold: old.threshold,
//             };
//             Some(new)
//         });
//         <StorageVersion<T>>::set(Some(Releases::V3));
//     } else {
//         frame_support::debug::info!(" >>> Unused migration!");
//     }
//     weight
// }
