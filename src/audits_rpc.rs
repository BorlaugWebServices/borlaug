use audits_runtime_api::AuditsApi as AuditsRuntimeApi;
use codec::Codec;
use frame_support::dispatch::fmt::Display;
use jsonrpsee::{
    core::{Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use pallet_primitives::{Audit, AuditStatus, Compliance, Evidence, Observation};
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc(client, server)]
pub trait AuditsApi<
    BlockHash,
    AccountId,
    ProposalId,
    AuditId,
    ControlPointId,
    EvidenceId,
    ObservationId,
>
{
    #[method(name = "get_audits_by_creator")]
    fn get_audits_by_creator(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<AuditResponse<AccountId, ProposalId, AuditId>>>;

    #[method(name = "get_audits_by_auditing_org")]
    fn get_audits_by_auditing_org(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<AuditResponse<AccountId, ProposalId, AuditId>>>;

    #[method(name = "get_audits_by_auditors")]
    fn get_audits_by_auditors(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<AuditResponse<AccountId, ProposalId, AuditId>>>;

    #[method(name = "get_linked_audits")]
    fn get_linked_audits(
        &self,
        audit_id: AuditId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<AuditResponse<AccountId, ProposalId, AuditId>>>;

    #[method(name = "get_audit")]
    fn get_audit(
        &self,
        audit_id: AuditId,
        at: Option<BlockHash>,
    ) -> RpcResult<AuditResponse<AccountId, ProposalId, AuditId>>;

    #[method(name = "get_audit_by_proposal")]
    fn get_audit_by_proposal(
        &self,
        proposal_id: ProposalId,
        at: Option<BlockHash>,
    ) -> RpcResult<AuditResponse<AccountId, ProposalId, AuditId>>;

    #[method(name = "get_observation")]
    fn get_observation(
        &self,
        audit_id: AuditId,
        control_point_id: ControlPointId,
        observation_id: ObservationId,
        at: Option<BlockHash>,
    ) -> RpcResult<ObservationResponse<ObservationId, EvidenceId, ProposalId>>;

    #[method(name = "get_observation_by_proposal")]
    fn get_observation_by_proposal(
        &self,
        proposal_id: ProposalId,
        at: Option<BlockHash>,
    ) -> RpcResult<ObservationResponse<ObservationId, EvidenceId, ProposalId>>;

    #[method(name = "get_observation_by_control_point")]
    fn get_observation_by_control_point(
        &self,
        audit_id: AuditId,
        control_point_id: ControlPointId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<ObservationResponse<ObservationId, EvidenceId, ProposalId>>>;

    #[method(name = "get_evidence")]
    fn get_evidence(
        &self,
        audit_id: AuditId,
        evidence_id: EvidenceId,
        at: Option<BlockHash>,
    ) -> RpcResult<EvidenceResponse<EvidenceId, ProposalId>>;

    #[method(name = "get_evidence_by_audit")]
    fn get_evidence_by_audit(
        &self,
        audit_id: AuditId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<EvidenceResponse<EvidenceId, ProposalId>>>;

    #[method(name = "get_evidence_by_proposal")]
    fn get_evidence_by_proposal(
        &self,
        proposal_id: ProposalId,
        at: Option<BlockHash>,
    ) -> RpcResult<EvidenceResponse<EvidenceId, ProposalId>>;

    #[method(name = "get_evidence_links_by_evidence")]
    fn get_evidence_links_by_evidence(
        &self,
        evidence_id: EvidenceId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<ObservationId>>;

    //TODO: this is no-longer needed
    #[method(name = "get_evidence_links_by_observation")]
    fn get_evidence_links_by_observation(
        &self,
        observation_id: ObservationId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<EvidenceId>>;
}

#[derive(Serialize, Deserialize)]
pub struct AuditResponse<AccountId, ProposalId, AuditId> {
    pub audit_id: AuditId,
    pub proposal_id: ProposalId,
    pub status: String,
    pub audit_creator: AccountId,
    pub auditing_org: AccountId,
    pub auditors: Option<AccountId>,
}
impl<AccountId, ProposalId, AuditId> From<(AuditId, Audit<AccountId, ProposalId>)>
    for AuditResponse<AccountId, ProposalId, AuditId>
{
    fn from((audit_id, audit): (AuditId, Audit<AccountId, ProposalId>)) -> Self {
        AuditResponse::<AccountId, ProposalId, AuditId> {
            audit_id,
            proposal_id: audit.proposal_id,
            status: match audit.status {
                AuditStatus::Requested => "Requested".to_string(),
                AuditStatus::Accepted => "Accepted".to_string(),
                AuditStatus::Rejected => "Rejected".to_string(),
                AuditStatus::InProgress => "InProgress".to_string(),
                AuditStatus::Completed => "Completed".to_string(),
            },
            audit_creator: audit.audit_creator,
            auditing_org: audit.auditing_org,
            auditors: audit.auditors,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct ObservationResponse<ObservationId, EvidenceId, ProposalId> {
    pub observation_id: ObservationId,
    pub proposal_id: ProposalId,
    pub compliance: Option<String>,
    pub procedural_note_hash: Option<[u8; 32]>,
    pub evidences: Vec<EvidenceResponse<EvidenceId, ProposalId>>,
}
//TODO: send enums as enums not as strings
impl<ObservationId, EvidenceId, ProposalId, BoundedString, BoundedStringUrl>
    From<(
        ObservationId,
        Observation<ProposalId>,
        Vec<(
            EvidenceId,
            Evidence<ProposalId, BoundedString, BoundedStringUrl>,
        )>,
    )> for ObservationResponse<ObservationId, EvidenceId, ProposalId>
where
    BoundedString: Into<Vec<u8>>,
    BoundedStringUrl: Into<Vec<u8>>,
{
    fn from(
        (observation_id, observation, evidences): (
            ObservationId,
            Observation<ProposalId>,
            Vec<(
                EvidenceId,
                Evidence<ProposalId, BoundedString, BoundedStringUrl>,
            )>,
        ),
    ) -> Self {
        ObservationResponse {
            observation_id,
            proposal_id: observation.proposal_id,
            compliance: observation.compliance.map(|compliance| match compliance {
                Compliance::Compliant => "Compliant".to_string(),
                Compliance::NonCompliant => "NonCompliant".to_string(),
                Compliance::NotApplicable => "NotApplicable".to_string(),
            }),
            procedural_note_hash: observation.procedural_note_hash,
            evidences: evidences
                .into_iter()
                .map(|(evidence_id, evidence)| (evidence_id, evidence).into())
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EvidenceResponse<EvidenceId, ProposalId> {
    pub evidence_id: EvidenceId,
    pub proposal_id: ProposalId,
    pub name: String,
    pub content_type: String,
    pub url: Option<String>,
    pub hash: String,
}

impl<EvidenceId, ProposalId, BoundedString, BoundedStringUrl>
    From<(
        EvidenceId,
        Evidence<ProposalId, BoundedString, BoundedStringUrl>,
    )> for EvidenceResponse<EvidenceId, ProposalId>
where
    BoundedString: Into<Vec<u8>>,
    BoundedStringUrl: Into<Vec<u8>>,
{
    fn from(
        (evidence_id, evidence): (
            EvidenceId,
            Evidence<ProposalId, BoundedString, BoundedStringUrl>,
        ),
    ) -> Self {
        EvidenceResponse {
            evidence_id,
            proposal_id: evidence.proposal_id,
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

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
    NotFoundError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
            Error::NotFoundError => 404,
        }
    }
}

static RPC_MODULE: &str = "Audits API";

macro_rules! convert_error {
    () => {{
        |e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                format!("Runtime Error in {}", RPC_MODULE),
                Some(format!("{:?}", e)),
            ))
        }
    }};
}

// macro_rules! decode_error {
//     () => {{
//         |e| {
//             CallError::Custom(ErrorObject::owned(
//                 Error::DecodeError.into(),
//                 format!("Decode Error in {}", RPC_MODULE),
//                 Some(format!("{:?}", e)),
//             ))
//         }
//     }};
// }
macro_rules! not_found_error {
    ($id:expr) => {{
        {
            CallError::Custom(ErrorObject::owned(
                Error::NotFoundError.into(),
                format!("Entity not found Error in {}", RPC_MODULE),
                Some(format!("{}", $id)),
            ))
        }
    }};
}
impl<
        C,
        Block,
        AccountId,
        ProposalId,
        AuditId,
        ControlPointId,
        EvidenceId,
        ObservationId,
        BoundedStringName,
        BoundedStringUrl,
    >
    AuditsApiServer<
        <Block as BlockT>::Hash,
        AccountId,
        ProposalId,
        AuditId,
        ControlPointId,
        EvidenceId,
        ObservationId,
    >
    for Audits<
        C,
        (
            Block,
            AccountId,
            ProposalId,
            AuditId,
            ControlPointId,
            EvidenceId,
            ObservationId,
            BoundedStringName,
            BoundedStringUrl,
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
        ProposalId,
        AuditId,
        ControlPointId,
        EvidenceId,
        ObservationId,
        BoundedStringName,
        BoundedStringUrl,
    >,
    AccountId: Codec + Send + Sync + 'static,
    ProposalId: Codec + Copy + Send + Display + Sync + 'static,
    AuditId: Codec + Copy + Send + Display + Sync + 'static,
    ControlPointId: Codec + Copy + Send + Sync + 'static,
    EvidenceId: Codec + Copy + Send + Display + Sync + 'static,
    ObservationId: Codec + Copy + Send + Display + Sync + 'static,
    BoundedStringName: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
    BoundedStringUrl: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
{
    fn get_audits_by_creator(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<AuditResponse<AccountId, ProposalId, AuditId>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let audits = api
            .get_audits_by_creator(at, account)
            .map_err(convert_error!())?;
        Ok(audits
            .into_iter()
            .map(|(audit_id, audit)| (audit_id, audit).into())
            .collect())
    }

    fn get_audits_by_auditing_org(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<AuditResponse<AccountId, ProposalId, AuditId>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let audits = api
            .get_audits_by_auditing_org(at, account)
            .map_err(convert_error!())?;
        Ok(audits
            .into_iter()
            .map(|(audit_id, audit)| (audit_id, audit).into())
            .collect())
    }

    fn get_audits_by_auditors(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<AuditResponse<AccountId, ProposalId, AuditId>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let audits = api
            .get_audits_by_auditors(at, account)
            .map_err(convert_error!())?;
        Ok(audits
            .into_iter()
            .map(|(audit_id, audit)| (audit_id, audit).into())
            .collect())
    }

    fn get_linked_audits(
        &self,
        audit_id: AuditId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<AuditResponse<AccountId, ProposalId, AuditId>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let audits = api
            .get_linked_audits(at, audit_id)
            .map_err(convert_error!())?;
        Ok(audits
            .into_iter()
            .map(|(audit_id, audit)| (audit_id, audit).into())
            .collect())
    }

    fn get_audit(
        &self,
        audit_id: AuditId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<AuditResponse<AccountId, ProposalId, AuditId>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let audit = api
            .get_audit(at, audit_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!(audit_id))?;
        Ok((audit_id, audit).into())
    }

    fn get_audit_by_proposal(
        &self,
        proposal_id: ProposalId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<AuditResponse<AccountId, ProposalId, AuditId>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let (audit_id, audit) = api
            .get_audit_by_proposal(at, proposal_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!(proposal_id))?;
        Ok((audit_id, audit).into())
    }

    fn get_observation(
        &self,
        audit_id: AuditId,
        control_point_id: ControlPointId,
        observation_id: ObservationId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<ObservationResponse<ObservationId, EvidenceId, ProposalId>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let (observation, evidences) = api
            .get_observation(at, audit_id, control_point_id, observation_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!(observation_id))?;
        Ok((observation_id, observation, evidences).into())
    }

    fn get_observation_by_proposal(
        &self,
        proposal_id: ProposalId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<ObservationResponse<ObservationId, EvidenceId, ProposalId>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let (observation_id, observation, evidences) = api
            .get_observation_by_proposal(at, proposal_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!(proposal_id))?;
        Ok((observation_id, observation, evidences).into())
    }

    fn get_observation_by_control_point(
        &self,
        audit_id: AuditId,
        control_point_id: ControlPointId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<ObservationResponse<ObservationId, EvidenceId, ProposalId>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let observations = api
            .get_observation_by_control_point(at, audit_id, control_point_id)
            .map_err(convert_error!())?;
        Ok(observations
            .into_iter()
            .map(|(observation_id, observation, evidences)| {
                (observation_id, observation, evidences).into()
            })
            .collect())
    }

    fn get_evidence(
        &self,
        audit_id: AuditId,
        evidence_id: EvidenceId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<EvidenceResponse<EvidenceId, ProposalId>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let evidence = api
            .get_evidence(at, audit_id, evidence_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!(evidence_id))?;
        Ok((evidence_id, evidence).into())
    }

    fn get_evidence_by_audit(
        &self,
        audit_id: AuditId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<EvidenceResponse<EvidenceId, ProposalId>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let evidences = api
            .get_evidence_by_audit(at, audit_id)
            .map_err(convert_error!())?;
        Ok(evidences
            .into_iter()
            .map(|(evidence_id, evidence)| (evidence_id, evidence).into())
            .collect())
    }

    fn get_evidence_by_proposal(
        &self,
        proposal_id: ProposalId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<EvidenceResponse<EvidenceId, ProposalId>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let (evidence_id, evidence) = api
            .get_evidence_by_proposal(at, proposal_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!(proposal_id))?;
        Ok((evidence_id, evidence).into())
    }

    fn get_evidence_links_by_evidence(
        &self,
        evidence_id: EvidenceId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<ObservationId>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let evidence_links = api
            .get_evidence_links_by_evidence(at, evidence_id)
            .map_err(convert_error!())?;
        Ok(evidence_links)
    }

    fn get_evidence_links_by_observation(
        &self,
        observation_id: ObservationId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<EvidenceId>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let evidence_links = api
            .get_evidence_links_by_observation(at, observation_id)
            .map_err(convert_error!())?;
        Ok(evidence_links)
    }
}
