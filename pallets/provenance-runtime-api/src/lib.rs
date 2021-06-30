#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::{
    definition::Definition, definition_step::DefinitionStep, process::Process,
    process_step::ProcessStep, registry::Registry,
};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait ProvenanceApi<RegistryId,DefinitionId,ProcessId,GroupId,MemberCount,DefinitionStepIndex>
    where
    RegistryId: Codec,
    DefinitionId: Codec,
    ProcessId: Codec,
    GroupId: Codec,
    MemberCount: Codec,
    DefinitionStepIndex: Codec,
     {
        fn get_registries(group_id: GroupId) -> Vec<(RegistryId, Registry)>;
        fn get_registry(group_id: GroupId,registry_id:RegistryId) -> Option<Registry>;
        fn get_definitions(registry_id:RegistryId) -> Vec<(DefinitionId,Definition)>;
        fn get_definition(registry_id:RegistryId,definition_id:DefinitionId) -> Option<Definition>;
        fn get_definition_steps(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(DefinitionStepIndex,DefinitionStep<GroupId,MemberCount>)>;
        fn get_processes(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(ProcessId,Process)>;
        fn get_process(registry_id:RegistryId,definition_id:DefinitionId,process_id: ProcessId) -> Option<Process>;
        fn get_process_steps(registry_id:RegistryId,definition_id:DefinitionId,process_id: ProcessId) -> Vec<ProcessStep>;
        fn get_process_step(registry_id:RegistryId,definition_id:DefinitionId,process_id: ProcessId,definition_step_index:DefinitionStepIndex) -> Option<ProcessStep>;


    }
}
