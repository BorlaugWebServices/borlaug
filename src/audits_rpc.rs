use audits_runtime_api::AuditsApi as AuditsRuntimeApi;
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait AuditsApi<
    BlockHash,
    AccountId,
    AuditId,
    ControlPointId,
    EvidenceId,
    ObservationId,
    BoundedStringName,
>
{
    #[rpc(name = "get_audits_by_creator")]
    fn get_audits_by_creator(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Vec<AuditResponse<AccountId, AuditId>>>;

    #[rpc(name = "get_audits_by_auditor")]
    fn get_audits_by_auditor(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Vec<AuditResponse<AccountId, AuditId>>>;
}

#[derive(Serialize, Deserialize)]
pub struct AuditResponse<AccountId, AuditId> {
    pub audit_id: AuditId,
    pub status: String,
    pub audit_creator: AccountId,
    pub auditor: AccountId,
}

pub struct Audits<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Audits<C, M> {
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
            message: "Error in Audits API".into(),
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
        AuditId,
        ControlPointId,
        EvidenceId,
        ObservationId,
        BoundedStringName,
    >
    AuditsApi<
        <Block as BlockT>::Hash,
        AccountId,
        AuditId,
        ControlPointId,
        EvidenceId,
        ObservationId,
        BoundedStringName,
    >
    for Audits<
        C,
        (
            Block,
            AccountId,
            AuditId,
            ControlPointId,
            EvidenceId,
            ObservationId,
            BoundedStringName,
        ),
    >
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: AuditsRuntimeApi<
        Block,
        AccountId,
        AuditId,
        ControlPointId,
        EvidenceId,
        ObservationId,
        BoundedStringName,
    >,
    AccountId: Codec + Send + Sync + 'static,
    AuditId: Codec + Copy + Send + Sync + 'static,
    ControlPointId: Codec + Copy + Send + Sync + 'static,
    EvidenceId: Codec + Copy + Send + Sync + 'static,
    ObservationId: Codec + Copy + Send + Sync + 'static,
    BoundedStringName: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
{
    fn get_audits_by_creator(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<AuditResponse<AccountId, AuditId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let audits = api
            .get_audits_by_creator(&at, account)
            .map_err(convert_error!())?;
        Ok(audits
            .into_iter()
            .map(|(audit_id, audit)| AuditResponse::<AccountId, AuditId> {
                audit_id,
                audit_creator: audit.audit_creator,
                auditor: audit.auditor,
                //TODO: can this be done neater?
                status: match audit.status {
                    pallet_primitives::AuditStatus::Requested => "Requested".to_string(),
                    pallet_primitives::AuditStatus::Accepted => "Accepted".to_string(),
                    pallet_primitives::AuditStatus::Rejected => "Rejected".to_string(),
                    pallet_primitives::AuditStatus::InProgress => "InProgress".to_string(),
                    pallet_primitives::AuditStatus::Completed => "Completed".to_string(),
                },
            })
            .collect())
    }

    fn get_audits_by_auditor(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<AuditResponse<AccountId, AuditId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let audits = api
            .get_audits_by_auditor(&at, account)
            .map_err(convert_error!())?;
        Ok(audits
            .into_iter()
            .map(|(audit_id, audit)| AuditResponse::<AccountId, AuditId> {
                audit_id,
                audit_creator: audit.audit_creator,
                auditor: audit.auditor,
                status: match audit.status {
                    pallet_primitives::AuditStatus::Requested => "Requested".to_string(),
                    pallet_primitives::AuditStatus::Accepted => "Accepted".to_string(),
                    pallet_primitives::AuditStatus::Rejected => "Rejected".to_string(),
                    pallet_primitives::AuditStatus::InProgress => "InProgress".to_string(),
                    pallet_primitives::AuditStatus::Completed => "Completed".to_string(),
                },
            })
            .collect())
    }
}
