use crate::identity_rpc::FactResponse;
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_primitives::{Process, ProcessStatus, ProcessStep, Registry};
use provenance_runtime_api::ProvenanceApi as ProvenanceRuntimeApi;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait ProvenanceApi<
    BlockHash,
    AccountId,
    RegistryId,
    DefinitionId,
    ProcessId,
    MemberCount,
    DefinitionStepIndex,
>
{
    #[rpc(name = "get_definition_registries")]
    fn get_registries(
        &self,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Vec<RegistryResponse<RegistryId>>>;

    #[rpc(name = "get_definition_registry")]
    fn get_registry(
        &self,
        account_id: AccountId,
        registry_id: RegistryId,
        at: Option<BlockHash>,
    ) -> Result<RegistryResponse<RegistryId>>;

    #[rpc(name = "get_definitions")]
    fn get_definitions(
        &self,
        registry_id: RegistryId,
        at: Option<BlockHash>,
    ) -> Result<
        Vec<
            DefinitionResponse<
                AccountId,
                RegistryId,
                DefinitionId,
                MemberCount,
                DefinitionStepIndex,
            >,
        >,
    >;

    #[rpc(name = "get_definition")]
    fn get_definition(
        &self,
        registry_id: RegistryId,
        definition_id: DefinitionId,
        at: Option<BlockHash>,
    ) -> Result<
        DefinitionResponse<AccountId, RegistryId, DefinitionId, MemberCount, DefinitionStepIndex>,
    >;

    #[rpc(name = "get_available_definitions")]
    fn get_available_definitions(
        &self,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> Result<
        Vec<
            DefinitionResponse<
                AccountId,
                RegistryId,
                DefinitionId,
                MemberCount,
                DefinitionStepIndex,
            >,
        >,
    >;

    #[rpc(name = "get_processes")]
    fn get_processes(
        &self,
        registry_id: RegistryId,
        definition_id: DefinitionId,
        at: Option<BlockHash>,
    ) -> Result<Vec<ProcessResponse<RegistryId, DefinitionId, ProcessId>>>;

    #[rpc(name = "get_process")]
    fn get_process(
        &self,
        registry_id: RegistryId,
        definition_id: DefinitionId,
        process_id: ProcessId,
        at: Option<BlockHash>,
    ) -> Result<ProcessResponse<RegistryId, DefinitionId, ProcessId>>;

    #[rpc(name = "get_processes_for_attestor_by_status")]
    fn get_processes_for_attestor_by_status(
        &self,
        account_id: AccountId,
        status: String,
        at: Option<BlockHash>,
    ) -> Result<Vec<ProcessResponse<RegistryId, DefinitionId, ProcessId>>>;

    #[rpc(name = "get_processes_for_attestor_pending")]
    fn get_processes_for_attestor_pending(
        &self,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Vec<ProcessResponse<RegistryId, DefinitionId, ProcessId>>>;

    #[rpc(name = "get_process_step")]
    fn get_process_step(
        &self,
        registry_id: RegistryId,
        definition_id: DefinitionId,
        process_id: ProcessId,
        definition_step_index: DefinitionStepIndex,
        at: Option<BlockHash>,
    ) -> Result<ProcessStepResponse>;
}

#[derive(Serialize, Deserialize)]
pub struct RegistryResponse<RegistryId> {
    pub registry_id: RegistryId,
    pub name: String,
}

impl<RegistryId, BoundedStringName> From<(RegistryId, Registry<BoundedStringName>)>
    for RegistryResponse<RegistryId>
where
    BoundedStringName: Into<Vec<u8>>,
{
    fn from((registry_id, registry): (RegistryId, Registry<BoundedStringName>)) -> Self {
        RegistryResponse {
            registry_id,
            name: String::from_utf8_lossy(&registry.name.into()).to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DefinitionResponse<AccountId, RegistryId, DefinitionId, MemberCount, DefinitionStepIndex>
{
    pub registry_id: RegistryId,
    pub definition_id: DefinitionId,
    pub name: String,
    pub definition_steps:
        Option<Vec<DefinitionStepResponse<AccountId, MemberCount, DefinitionStepIndex>>>,
}
#[derive(Serialize, Deserialize)]
pub struct DefinitionStepResponse<AccountId, MemberCount, DefinitionStepIndex> {
    pub definition_step_index: DefinitionStepIndex,
    pub name: String,
    pub attestor: Option<AccountId>,
    pub threshold: MemberCount,
}
#[derive(Serialize, Deserialize)]
pub struct ProcessResponse<RegistryId, DefinitionId, ProcessId> {
    pub registry_id: RegistryId,
    pub definition_id: DefinitionId,
    pub process_id: ProcessId,
    pub name: String,
    pub process_steps: Option<Vec<ProcessStepResponse>>,
    pub status: String,
}
impl<RegistryId, DefinitionId, ProcessId, BoundedStringName>
    From<(
        RegistryId,
        DefinitionId,
        ProcessId,
        Process<BoundedStringName>,
    )> for ProcessResponse<RegistryId, DefinitionId, ProcessId>
where
    BoundedStringName: Into<Vec<u8>>,
{
    fn from(
        (registry_id, definition_id, process_id, process): (
            RegistryId,
            DefinitionId,
            ProcessId,
            Process<BoundedStringName>,
        ),
    ) -> Self {
        ProcessResponse::<RegistryId, DefinitionId, ProcessId> {
            registry_id,
            definition_id,
            process_id,
            name: String::from_utf8_lossy(&process.name.into()).to_string(),
            process_steps: None,
            status: match process.status {
                pallet_primitives::ProcessStatus::Completed => "Completed".to_string(),
                pallet_primitives::ProcessStatus::InProgress => "InProgress".to_string(),
            },
        }
    }
}
impl<RegistryId, DefinitionId, ProcessId, BoundedStringName, BoundedStringFact>
    From<(
        RegistryId,
        DefinitionId,
        ProcessId,
        Process<BoundedStringName>,
        Vec<ProcessStep<BoundedStringName, BoundedStringFact>>,
    )> for ProcessResponse<RegistryId, DefinitionId, ProcessId>
where
    BoundedStringName: Into<Vec<u8>>,
    BoundedStringFact: Into<Vec<u8>>,
{
    fn from(
        (registry_id, definition_id, process_id, process, process_steps): (
            RegistryId,
            DefinitionId,
            ProcessId,
            Process<BoundedStringName>,
            Vec<ProcessStep<BoundedStringName, BoundedStringFact>>,
        ),
    ) -> Self {
        ProcessResponse::<RegistryId, DefinitionId, ProcessId> {
            registry_id,
            definition_id,
            process_id,
            name: String::from_utf8_lossy(&process.name.into()).to_string(),
            process_steps: Some(
                process_steps
                    .into_iter()
                    .map(|process_step| ProcessStepResponse {
                        attested: process_step.attested,
                        attributes: process_step
                            .attributes
                            .into_iter()
                            .map(|attribute| AttributeResponse {
                                name: String::from_utf8_lossy(&attribute.name.into()).to_string(),
                                fact: attribute.fact.into(),
                            })
                            .collect(),
                    })
                    .collect(),
            ),
            status: match process.status {
                pallet_primitives::ProcessStatus::Completed => "Completed".to_string(),
                pallet_primitives::ProcessStatus::InProgress => "InProgress".to_string(),
            },
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct ProcessStepResponse {
    pub attested: bool,
    pub attributes: Vec<AttributeResponse>,
}
#[derive(Serialize, Deserialize)]
pub struct AttributeResponse {
    pub name: String,
    pub fact: FactResponse,
}

pub struct Provenance<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Provenance<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

macro_rules! convert_error {
    () => {{
        |e| RpcError {
            code: ErrorCode::ServerError(1),
            message: "Error in Provenance API".into(),
            data: Some(format!("{:?}", e).into()),
        }
    }};
}

macro_rules! not_found_error {
    () => {{
        RpcError {
            code: ErrorCode::ServerError(404),
            message: "Entity not found".into(),
            data: Some("Entity not found".into()),
        }
    }};
}

impl<
        C,
        Block,
        AccountId,
        RegistryId,
        DefinitionId,
        ProcessId,
        MemberCount,
        DefinitionStepIndex,
        BoundedStringName,
        BoundedStringFact,
    >
    ProvenanceApi<
        <Block as BlockT>::Hash,
        AccountId,
        RegistryId,
        DefinitionId,
        ProcessId,
        MemberCount,
        DefinitionStepIndex,
    >
    for Provenance<
        C,
        (
            Block,
            AccountId,
            RegistryId,
            DefinitionId,
            ProcessId,
            MemberCount,
            DefinitionStepIndex,
            BoundedStringName,
            BoundedStringFact,
        ),
    >
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: ProvenanceRuntimeApi<
        Block,
        AccountId,
        RegistryId,
        DefinitionId,
        ProcessId,
        MemberCount,
        DefinitionStepIndex,
        BoundedStringName,
        BoundedStringFact,
    >,
    AccountId: Codec + Send + Sync + 'static,
    RegistryId: Codec + Copy + Send + Sync + 'static,
    DefinitionId: Codec + Copy + Send + Sync + 'static,
    ProcessId: Codec + Copy + Send + Sync + 'static,
    MemberCount: Codec + Copy + Send + Sync + 'static,
    DefinitionStepIndex: Codec + Copy + Send + Sync + 'static,
    BoundedStringName: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
    BoundedStringFact: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
{
    fn get_registries(
        &self,
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<RegistryResponse<RegistryId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let registries = api
            .get_registries(&at, account_id)
            .map_err(convert_error!())?;
        Ok(registries
            .into_iter()
            .map(|(registry_id, registry)| (registry_id, registry).into())
            .collect())
    }

    fn get_registry(
        &self,
        account_id: AccountId,
        registry_id: RegistryId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RegistryResponse<RegistryId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let registry = api
            .get_registry(&at, account_id, registry_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;

        Ok((registry_id, registry).into())
    }

    fn get_definitions(
        &self,
        registry_id: RegistryId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<
        Vec<
            DefinitionResponse<
                AccountId,
                RegistryId,
                DefinitionId,
                MemberCount,
                DefinitionStepIndex,
            >,
        >,
    > {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let definitions = api
            .get_definitions(&at, registry_id)
            .map_err(convert_error!())?;

        Ok(definitions
            .into_iter()
            .map(|(definition_id, definition)| DefinitionResponse::<
                AccountId,
                RegistryId,
                DefinitionId,
                MemberCount,
                DefinitionStepIndex,
            > {
                registry_id,
                definition_id,
                name: String::from_utf8_lossy(&definition.name.into()).to_string(),
                definition_steps: None,
            })
            .collect())
    }

    fn get_definition(
        &self,
        registry_id: RegistryId,
        definition_id: DefinitionId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<
        DefinitionResponse<AccountId, RegistryId, DefinitionId, MemberCount, DefinitionStepIndex>,
    > {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let definition = api
            .get_definition(&at, registry_id, definition_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;

        let definition_steps = api
            .get_definition_steps(&at, registry_id, definition_id)
            .map_err(convert_error!())?;

        Ok(DefinitionResponse {
            registry_id,
            definition_id,
            name: String::from_utf8_lossy(&definition.name.into()).to_string(),
            definition_steps: Some(
                definition_steps
                    .into_iter()
                    .map(
                        |(definition_step_index, definition_step)| DefinitionStepResponse {
                            definition_step_index,
                            name: String::from_utf8_lossy(&definition_step.name.into()).to_string(),
                            attestor: definition_step.attestor,
                            threshold: definition_step.threshold,
                        },
                    )
                    .collect(),
            ),
        })
    }

    fn get_available_definitions(
        &self,
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<
        Vec<
            DefinitionResponse<
                AccountId,
                RegistryId,
                DefinitionId,
                MemberCount,
                DefinitionStepIndex,
            >,
        >,
    > {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let definitions = api
            .get_available_definitions(&at, account_id)
            .map_err(convert_error!())?;

        Ok(definitions
            .into_iter()
            .map(
                |(registry_id, definition_id, definition)| DefinitionResponse {
                    registry_id,
                    definition_id,
                    name: String::from_utf8_lossy(&definition.name.into()).to_string(),
                    definition_steps: None,
                },
            )
            .collect())
    }

    fn get_processes(
        &self,
        registry_id: RegistryId,
        definition_id: DefinitionId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<ProcessResponse<RegistryId, DefinitionId, ProcessId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let processes = api
            .get_processes(&at, registry_id, definition_id)
            .map_err(convert_error!())?;

        Ok(processes
            .into_iter()
            .map(|(process_id, process)| (registry_id, definition_id, process_id, process).into())
            .collect())
    }

    fn get_process(
        &self,
        registry_id: RegistryId,
        definition_id: DefinitionId,
        process_id: ProcessId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<ProcessResponse<RegistryId, DefinitionId, ProcessId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let process = api
            .get_process(&at, registry_id, definition_id, process_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;

        let process_steps = api
            .get_process_steps(&at, registry_id, definition_id, process_id)
            .map_err(convert_error!())?;

        Ok((
            registry_id,
            definition_id,
            process_id,
            process,
            process_steps,
        )
            .into())
    }

    fn get_processes_for_attestor_by_status(
        &self,
        account_id: AccountId,
        status: String,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<ProcessResponse<RegistryId, DefinitionId, ProcessId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let status = match status.as_str() {
            "InProgress" => ProcessStatus::InProgress,
            "Completed" => ProcessStatus::Completed,
            _ => {
                return Err(RpcError {
                    code: ErrorCode::ServerError(1),
                    message: "Unknown status".into(),
                    data: Some("Unknown status".into()),
                })
            }
        };
        let processes = api
            .get_processes_for_attestor_by_status(&at, account_id, status)
            .map_err(convert_error!())?;

        Ok(processes
            .into_iter()
            .map(|(registry_id, definition_id, process_id, process)| {
                (registry_id, definition_id, process_id, process).into()
            })
            .collect())
    }

    fn get_processes_for_attestor_pending(
        &self,
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<ProcessResponse<RegistryId, DefinitionId, ProcessId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let processes = api
            .get_processes_for_attestor_pending(&at, account_id)
            .map_err(convert_error!())?;

        Ok(processes
            .into_iter()
            .map(|(registry_id, definition_id, process_id, process)| {
                (registry_id, definition_id, process_id, process).into()
            })
            .collect())
    }

    fn get_process_step(
        &self,
        registry_id: RegistryId,
        definition_id: DefinitionId,
        process_id: ProcessId,
        definition_step_index: DefinitionStepIndex,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<ProcessStepResponse> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let process_step = api
            .get_process_step(
                &at,
                registry_id,
                definition_id,
                process_id,
                definition_step_index,
            )
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;

        Ok(ProcessStepResponse {
            attested: process_step.attested,
            attributes: process_step
                .attributes
                .into_iter()
                .map(|attribute| AttributeResponse {
                    name: String::from_utf8_lossy(&attribute.name.into()).to_string(),
                    fact: attribute.fact.into(),
                })
                .collect(),
        })
    }
}
