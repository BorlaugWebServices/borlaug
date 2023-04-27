pub mod deprecated {
    use codec::{Decode, Encode};
    use primitives::Compliance;
    use sp_runtime::RuntimeDebug;

    #[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
    pub struct OldObservation {
        pub compliance: Option<Compliance>,
        pub procedural_note_hash: Option<[u8; 32]>,
    }
}

// #[allow(clippy::unnecessary_cast)]
// pub fn migrate_to_v2<T: Config>() -> Weight {
//     let mut weight: Weight = 0;

//     let storage_version_maybe = <StorageVersion<T>>::get();

//     if storage_version_maybe.is_none() || storage_version_maybe.unwrap() == Releases::V1 {
//         <ObservationByProposal<T>>::iter().for_each(
//             |(proposal_id, (_audit_id, _control_point_id, _observation_id))| {
//                 //TODO: This is wrong.
//                 <Observations<T>>::translate::<deprecated::OldObservation, _>(
//                     |(_audit_id, _control_point_id), _observation_id, old_observation| {
//                         weight += T::DbWeight::get().reads_writes(1 as Weight, 1 as Weight);
//                         let new_observation = Observation {
//                             proposal_id,
//                             compliance: old_observation.compliance,
//                             procedural_note_hash: old_observation.procedural_note_hash,
//                         };
//                         Some(new_observation)
//                     },
//                 );
//             },
//         );
//     } else {
//         frame_support::debug::info!(" >>> Unused migration!");
//     }
//     weight
// }
