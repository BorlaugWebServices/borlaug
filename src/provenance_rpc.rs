use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_primitives::Fact;
use provenance_runtime_api::ProvenanceApi as ProvenanceRuntimeApi;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait ProvenanceApi<
    BlockHash,
    RegistryId,
    DefinitionId,
    ProcessId,
    GroupId,
    MemberCount,
    DefinitionStepIndex,
>
{
    #[rpc(name = "get_registries")]
    fn get_registries(
        &self,
        group_id: GroupId,
        at: Option<BlockHash>,
    ) -> Result<Vec<RegistryResponse<RegistryId>>>;

    #[rpc(name = "get_registry")]
    fn get_registry(
        &self,
        group_id: GroupId,
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
            DefinitionResponse<RegistryId, DefinitionId, GroupId, MemberCount, DefinitionStepIndex>,
        >,
    >;

    #[rpc(name = "get_definition")]
    fn get_definition(
        &self,
        registry_id: RegistryId,
        definition_id: DefinitionId,
        at: Option<BlockHash>,
    ) -> Result<
        DefinitionResponse<RegistryId, DefinitionId, GroupId, MemberCount, DefinitionStepIndex>,
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

#[derive(Serialize, Deserialize)]
pub struct DefinitionResponse<RegistryId, DefinitionId, GroupId, MemberCount, DefinitionStepIndex> {
    pub registry_id: RegistryId,
    pub definition_id: DefinitionId,
    pub name: String,
    pub definition_steps:
        Option<Vec<DefinitionStepResponse<GroupId, MemberCount, DefinitionStepIndex>>>,
}
#[derive(Serialize, Deserialize)]
pub struct DefinitionStepResponse<GroupId, MemberCount, DefinitionStepIndex> {
    pub definition_step_index: DefinitionStepIndex,
    pub name: String,
    pub group_id: Option<GroupId>,
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
#[derive(Serialize, Deserialize)]
pub struct FactResponse {
    #[serde(rename = "type")]
    pub data_type: String,
    pub value: String,
}

impl<BoundedString> From<Fact<BoundedString>> for FactResponse
where
    BoundedString: Into<Vec<u8>>,
{
    fn from(fact: Fact<BoundedString>) -> Self {
        match fact {
            Fact::Bool(value) => FactResponse {
                data_type: String::from("Bool"),
                value: value.to_string(),
            },
            Fact::Text(value) => FactResponse {
                data_type: String::from("Text"),
                value: String::from_utf8_lossy(&value.into()).to_string(),
            },
            Fact::U8(value) => FactResponse {
                data_type: String::from("U8"),
                value: value.to_string(),
            },
            Fact::U16(value) => FactResponse {
                data_type: String::from("U16"),
                value: value.to_string(),
            },
            Fact::U32(value) => FactResponse {
                data_type: String::from("U32"),
                value: value.to_string(),
            },
            Fact::U128(value) => FactResponse {
                data_type: String::from("U128"),
                value: value.to_string(),
            },
            Fact::Date(year, month, day) => {
                let date = NaiveDate::from_ymd(i32::from(year), u32::from(month), u32::from(day));
                FactResponse {
                    data_type: String::from("Date"),
                    value: date.to_string(),
                }
            }
            //TODO: check that this conversion is correct
            Fact::Iso8601(year, month, day, hour, minute, second, timezone) => {
                let timezone = String::from_utf8_lossy(&timezone).to_string();
                let date = NaiveDate::from_ymd(i32::from(year), u32::from(month), u32::from(day));
                let time =
                    NaiveTime::from_hms(u32::from(hour), u32::from(minute), u32::from(second));
                let dt = NaiveDateTime::new(date, time);
                FactResponse {
                    data_type: String::from("Iso8601"),
                    value: format!("{}{}", dt, timezone),
                }
            }
        }
    }
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
        RegistryId,
        DefinitionId,
        ProcessId,
        GroupId,
        MemberCount,
        DefinitionStepIndex,
        BoundedString,
    >
    ProvenanceApi<
        <Block as BlockT>::Hash,
        RegistryId,
        DefinitionId,
        ProcessId,
        GroupId,
        MemberCount,
        DefinitionStepIndex,
    >
    for Provenance<
        C,
        (
            Block,
            RegistryId,
            DefinitionId,
            ProcessId,
            GroupId,
            MemberCount,
            DefinitionStepIndex,
            BoundedString,
        ),
    >
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: ProvenanceRuntimeApi<
        Block,
        RegistryId,
        DefinitionId,
        ProcessId,
        GroupId,
        MemberCount,
        DefinitionStepIndex,
        BoundedString,
    >,

    RegistryId: Codec + Copy + Send + Sync + 'static,
    DefinitionId: Codec + Copy + Send + Sync + 'static,
    ProcessId: Codec + Copy + Send + Sync + 'static,
    GroupId: Codec + Copy + Send + Sync + 'static,
    MemberCount: Codec + Copy + Send + Sync + 'static,
    DefinitionStepIndex: Codec + Copy + Send + Sync + 'static,
    BoundedString: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
{
    fn get_registries(
        &self,
        group_id: GroupId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<RegistryResponse<RegistryId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let registries = api
            .get_registries(&at, group_id)
            .map_err(convert_error!())?;
        Ok(registries
            .into_iter()
            .map(|(registry_id, registry)| RegistryResponse::<RegistryId> {
                registry_id,
                name: String::from_utf8_lossy(&registry.name.into()).to_string(),
            })
            .collect())
    }

    fn get_registry(
        &self,
        group_id: GroupId,
        registry_id: RegistryId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RegistryResponse<RegistryId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let registry = api
            .get_registry(&at, group_id, registry_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;

        Ok(RegistryResponse::<RegistryId> {
            registry_id,
            name: String::from_utf8_lossy(&registry.name.into()).to_string(),
        })
    }

    fn get_definitions(
        &self,
        registry_id: RegistryId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<
        Vec<
            DefinitionResponse<RegistryId, DefinitionId, GroupId, MemberCount, DefinitionStepIndex>,
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
                RegistryId,
                DefinitionId,
                GroupId,
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
        DefinitionResponse<RegistryId, DefinitionId, GroupId, MemberCount, DefinitionStepIndex>,
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

        Ok(DefinitionResponse::<
            RegistryId,
            DefinitionId,
            GroupId,
            MemberCount,
            DefinitionStepIndex,
        > {
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
                            group_id: definition_step.group_id,
                            threshold: definition_step.threshold,
                        },
                    )
                    .collect(),
            ),
        })
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
            .map(
                |(process_id, process)| ProcessResponse::<RegistryId, DefinitionId, ProcessId> {
                    registry_id,
                    definition_id,
                    process_id,
                    name: String::from_utf8_lossy(&process.name.into()).to_string(),
                    process_steps: None,
                    status: match process.status {
                        pallet_primitives::ProcessStatus::Completed => "Completed".to_string(),
                        pallet_primitives::ProcessStatus::InProgress => "InProgress".to_string(),
                    },
                },
            )
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

        Ok(ProcessResponse::<RegistryId, DefinitionId, ProcessId> {
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
        })
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
