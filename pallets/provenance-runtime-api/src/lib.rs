#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::Vec;
use primitives::{Definition, DefinitionStep, Process, ProcessStatus, ProcessStep, Registry};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait ProvenanceApi<AccountId,RegistryId,DefinitionId,ProcessId,ProposalId,Moment,MemberCount,DefinitionStepIndex,BoundedStringName, BoundedStringFact>
    where
    AccountId: Codec,
    RegistryId: Codec,
    DefinitionId: Codec,
    ProcessId: Codec,
    ProposalId: Codec,
    Moment: Codec,
    MemberCount: Codec,
    DefinitionStepIndex: Codec,
    BoundedStringName: Codec + Into<Vec<u8>>,
    BoundedStringFact: Codec + Into<Vec<u8>>
     {
        fn get_registries(account_id: AccountId) -> Vec<(RegistryId, Registry<BoundedStringName>)>;

        fn get_registry(account_id: AccountId,registry_id:RegistryId) -> Option<Registry<BoundedStringName>>;

        fn get_definitions(registry_id:RegistryId) -> Vec<(DefinitionId,Definition<BoundedStringName>)>;

        fn get_definition(registry_id:RegistryId,definition_id:DefinitionId) -> Option<Definition<BoundedStringName>>;

        fn get_definition_step(registry_id:RegistryId,definition_id:DefinitionId,step_index: DefinitionStepIndex) -> Option<DefinitionStep<AccountId,MemberCount,BoundedStringName>>;

        fn get_definition_steps(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(DefinitionStepIndex,DefinitionStep<AccountId,MemberCount,BoundedStringName>)>;

        fn get_available_definitions(account_id: AccountId) -> Vec<(RegistryId,DefinitionId,Definition<BoundedStringName>)>;

        fn get_processes(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(ProcessId,Process<BoundedStringName>)>;

        fn get_process(registry_id:RegistryId,definition_id:DefinitionId,process_id: ProcessId) -> Option<Process<BoundedStringName>>;

        fn get_processes_for_attestor_by_status(account_id: AccountId,status: ProcessStatus) -> Vec<(RegistryId,DefinitionId,ProcessId,Process<BoundedStringName>)>;

        fn get_processes_for_attestor_pending(account_id: AccountId) -> Vec<(RegistryId,DefinitionId,ProcessId,Process<BoundedStringName>)>;

        fn get_process_steps(registry_id:RegistryId,definition_id:DefinitionId,process_id: ProcessId) -> Vec<(DefinitionStepIndex,ProcessStep<ProposalId,Moment,BoundedStringName, BoundedStringFact>)>;

        fn get_process_step(registry_id:RegistryId,definition_id:DefinitionId,process_id: ProcessId,definition_step_index:DefinitionStepIndex) -> Option<(DefinitionStepIndex,ProcessStep<ProposalId,Moment,BoundedStringName, BoundedStringFact>)>;

        fn get_definition_children(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(DefinitionId,Definition<BoundedStringName>)>;

        fn get_definition_parents(registry_id:RegistryId,definition_id:DefinitionId) -> Vec<(DefinitionId,Definition<BoundedStringName>)>;

        fn can_view_definition( account_id: AccountId,registry_id:RegistryId,definition_id:DefinitionId) -> bool;

        fn is_attestor( account_id: AccountId,registry_id:RegistryId,definition_id:DefinitionId,definition_step_index: DefinitionStepIndex ) -> bool;

    }
}
