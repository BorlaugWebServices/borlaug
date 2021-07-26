use audits_runtime_api::AuditsApi as AuditsRuntimeApi;
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_primitives::{AuditStatus, Compliance, Evidence, Observation};
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

    #[rpc(name = "get_observation")]
    fn get_observation(
        &self,
        audit_id: AuditId,
        control_point_id: ControlPointId,
        observation_id: ObservationId,
        at: Option<BlockHash>,
    ) -> Result<ObservationResponse<ObservationId>>;

    #[rpc(name = "get_observation_by_control_point")]
    fn get_observation_by_control_point(
        &self,
        audit_id: AuditId,
        control_point_id: ControlPointId,
        at: Option<BlockHash>,
    ) -> Result<Vec<ObservationResponse<ObservationId>>>;

    #[rpc(name = "get_evidence")]
    fn get_evidence(
        &self,
        audit_id: AuditId,
        evidence_id: EvidenceId,
        at: Option<BlockHash>,
    ) -> Result<EvidenceResponse<EvidenceId>>;

    #[rpc(name = "get_evidence_by_audit")]
    fn get_evidence_by_audit(
        &self,
        audit_id: AuditId,
        at: Option<BlockHash>,
    ) -> Result<Vec<EvidenceResponse<EvidenceId>>>;

    #[rpc(name = "get_evidence_links_by_evidence")]
    fn get_evidence_links_by_evidence(
        &self,
        evidence_id: EvidenceId,
        at: Option<BlockHash>,
    ) -> Result<Vec<ObservationId>>;

    #[rpc(name = "get_evidence_links_by_observation")]
    fn get_evidence_links_by_observation(
        &self,
        observation_id: ObservationId,
        at: Option<BlockHash>,
    ) -> Result<Vec<EvidenceId>>;
}

#[derive(Serialize, Deserialize)]
pub struct AuditResponse<AccountId, AuditId> {
    pub audit_id: AuditId,
    pub status: String,
    pub audit_creator: AccountId,
    pub auditor: AccountId,
}

#[derive(Serialize, Deserialize)]
pub struct ObservationResponse<ObservationId> {
    pub observation_id: ObservationId,
    pub compliance: Option<String>,
    pub procedural_note: Option<String>,
}

impl<ObservationId> From<(ObservationId, Observation)> for ObservationResponse<ObservationId> {
    fn from((observation_id, observation): (ObservationId, Observation)) -> Self {
        ObservationResponse {
            observation_id,
            compliance: observation.compliance.map(|compliance| match compliance {
                Compliance::Compliant => "Compliant".to_string(),
                Compliance::NonCompliant => "NonCompliant".to_string(),
                Compliance::NotApplicable => "NotApplicable".to_string(),
            }),
            procedural_note: observation
                .procedural_note
                .map(|procedural_note| String::from_utf8_lossy(&procedural_note).to_string()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EvidenceResponse<EvidenceId> {
    pub evidence_id: EvidenceId,
    pub name: String,
    pub content_type: String,
    pub url: Option<String>,
    pub hash: String,
}

impl<EvidenceId, BoundedString> From<(EvidenceId, Evidence<BoundedString>)>
    for EvidenceResponse<EvidenceId>
where
    BoundedString: Into<Vec<u8>>,
{
    fn from((evidence_id, evidence): (EvidenceId, Evidence<BoundedString>)) -> Self {
        EvidenceResponse {
            evidence_id,
            name: String::from_utf8_lossy(&evidence.name.into()).to_string(),
            content_type: String::from_utf8_lossy(&evidence.content_type.into()).to_string(),
            url: evidence
                .url
                .map(|url| String::from_utf8_lossy(&url.into()).to_string()),
            hash: String::from_utf8_lossy(&evidence.hash.into()).to_string(),
        }
    }
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
                    AuditStatus::Requested => "Requested".to_string(),
                    AuditStatus::Accepted => "Accepted".to_string(),
                    AuditStatus::Rejected => "Rejected".to_string(),
                    AuditStatus::InProgress => "InProgress".to_string(),
                    AuditStatus::Completed => "Completed".to_string(),
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
                    AuditStatus::Requested => "Requested".to_string(),
                    AuditStatus::Accepted => "Accepted".to_string(),
                    AuditStatus::Rejected => "Rejected".to_string(),
                    AuditStatus::InProgress => "InProgress".to_string(),
                    AuditStatus::Completed => "Completed".to_string(),
                },
            })
            .collect())
    }

    fn get_observation(
        &self,
        audit_id: AuditId,
        control_point_id: ControlPointId,
        observation_id: ObservationId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<ObservationResponse<ObservationId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let observation = api
            .get_observation(&at, audit_id, control_point_id, observation_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok((observation_id, observation).into())
    }

    fn get_observation_by_control_point(
        &self,
        audit_id: AuditId,
        control_point_id: ControlPointId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<ObservationResponse<ObservationId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let observations = api
            .get_observation_by_control_point(&at, audit_id, control_point_id)
            .map_err(convert_error!())?;
        Ok(observations
            .into_iter()
            .map(|(observation_id, observation)| (observation_id, observation).into())
            .collect())
    }

    fn get_evidence(
        &self,
        audit_id: AuditId,
        evidence_id: EvidenceId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<EvidenceResponse<EvidenceId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let evidence = api
            .get_evidence(&at, audit_id, evidence_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok((evidence_id, evidence).into())
    }

    fn get_evidence_by_audit(
        &self,
        audit_id: AuditId,

        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<EvidenceResponse<EvidenceId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let evidences = api
            .get_evidence_by_audit(&at, audit_id)
            .map_err(convert_error!())?;
        Ok(evidences
            .into_iter()
            .map(|(evidence_id, evidence)| (evidence_id, evidence).into())
            .collect())
    }

    fn get_evidence_links_by_evidence(
        &self,
        evidence_id: EvidenceId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<ObservationId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let evidence_links = api
            .get_evidence_links_by_evidence(&at, evidence_id)
            .map_err(convert_error!())?;
        Ok(evidence_links)
    }

    fn get_evidence_links_by_observation(
        &self,
        observation_id: ObservationId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<EvidenceId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let evidence_links = api
            .get_evidence_links_by_observation(&at, observation_id)
            .map_err(convert_error!())?;
        Ok(evidence_links)
    }
}
