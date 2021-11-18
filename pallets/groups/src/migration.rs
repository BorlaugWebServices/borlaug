use super::*;
use frame_support::traits::Get;
use frame_support::weights::Weight;

#[allow(clippy::unnecessary_cast)]
pub fn migrate_to_v2<T: Config>() -> Weight {
    let mut weight: Weight = 0;

    let storage_version_maybe = <StorageVersion<T>>::get();

    if storage_version_maybe.is_none() || storage_version_maybe.unwrap() == Releases::V1 {
        <Proposals<T>>::iter().for_each(|(group_id, proposal_id, _)| {
            <GroupByProposal<T>>::insert(proposal_id, group_id);
            weight += T::DbWeight::get().reads_writes(1 as Weight, 1 as Weight);
        });

        <StorageVersion<T>>::set(Some(Releases::V2));
    } else {
        frame_support::debug::info!(" >>> Unused migration!");
    }

    weight
}
