use codec::Codec;
use groups_runtime_api::GroupsApi as GroupsRuntimeApi;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_primitives::{Group, Votes};
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait GroupsApi<BlockHash, AccountId, GroupId, MemberCount, ProposalId, Hash> {
    #[rpc(name = "member_of")]
    fn member_of(&self, account: AccountId, at: Option<BlockHash>) -> Result<Vec<GroupId>>;

    #[rpc(name = "get_group")]
    fn get_group(
        &self,
        group_id: GroupId,
        at: Option<BlockHash>,
    ) -> Result<GroupResponse<GroupId, AccountId, MemberCount>>;

    #[rpc(name = "get_sub_groups")]
    fn get_sub_groups(
        &self,
        group_id: GroupId,
        at: Option<BlockHash>,
    ) -> Result<Vec<GroupResponse<GroupId, AccountId, MemberCount>>>;

    #[rpc(name = "get_proposal")]
    fn get_proposal(
        &self,
        group_id: GroupId,
        proposal_id: ProposalId,
        at: Option<BlockHash>,
    ) -> Result<ProposalResponse<ProposalId>>;

    #[rpc(name = "get_proposals")]
    fn get_proposals(
        &self,
        group_id: GroupId,
        at: Option<BlockHash>,
    ) -> Result<Vec<ProposalResponse<ProposalId>>>;

    #[rpc(name = "get_voting")]
    fn get_voting(
        &self,
        group_id: GroupId,
        proposal_id: ProposalId,
        at: Option<BlockHash>,
    ) -> Result<VoteResponse<AccountId, ProposalId, MemberCount>>;
}

#[derive(Serialize, Deserialize)]
pub struct GroupResponse<GroupId, AccountId, MemberCount> {
    pub group_id: GroupId,
    pub name: String,
    pub total_vote_weight: MemberCount,
    pub members: Vec<(AccountId, MemberCount)>,
    pub threshold: MemberCount,
    pub anonymous_account: AccountId,
    pub parent: Option<GroupId>,
}
impl<GroupId, AccountId, MemberCount, BoundedString>
    From<(
        GroupId,
        Group<GroupId, AccountId, MemberCount, BoundedString>,
        Vec<(AccountId, MemberCount)>,
    )> for GroupResponse<GroupId, AccountId, MemberCount>
where
    BoundedString: Into<Vec<u8>>,
{
    fn from(
        (group_id, group, members): (
            GroupId,
            Group<GroupId, AccountId, MemberCount, BoundedString>,
            Vec<(AccountId, MemberCount)>,
        ),
    ) -> Self {
        GroupResponse {
            group_id,
            name: String::from_utf8_lossy(&group.name.into()).to_string(),
            total_vote_weight: group.total_vote_weight,
            members: members,
            threshold: group.threshold,
            anonymous_account: group.anonymous_account,
            parent: group.parent,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProposalResponse<ProposalId> {
    pub proposal_id: ProposalId,
    pub hash: String,
    pub proposal_len: u32,
}
impl<ProposalId, Hash> From<(ProposalId, Hash, u32)> for ProposalResponse<ProposalId>
where
    Hash: AsRef<[u8]>,
{
    fn from((proposal_id, hash, proposal_len): (ProposalId, Hash, u32)) -> Self {
        ProposalResponse {
            proposal_id,
            hash: String::from_utf8_lossy(hash.as_ref()).to_string(),
            proposal_len,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VoteResponse<AccountId, ProposalId, MemberCount> {
    pub proposal_id: ProposalId,
    pub threshold: MemberCount,
    pub ayes: Vec<(AccountId, MemberCount)>,
    pub nays: Vec<(AccountId, MemberCount)>,
}
impl<AccountId, ProposalId, MemberCount> From<(ProposalId, Votes<AccountId, MemberCount>)>
    for VoteResponse<AccountId, ProposalId, MemberCount>
{
    fn from((proposal_id, vote): (ProposalId, Votes<AccountId, MemberCount>)) -> Self {
        VoteResponse {
            proposal_id,
            threshold: vote.threshold,
            ayes: vote.ayes,
            nays: vote.nays,
        }
    }
}

pub struct Groups<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Groups<C, M> {
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

impl<C, Block, AccountId, GroupId, MemberCount, ProposalId, Hash, BoundedString>
    GroupsApi<<Block as BlockT>::Hash, AccountId, GroupId, MemberCount, ProposalId, Hash>
    for Groups<
        C,
        (
            Block,
            AccountId,
            GroupId,
            MemberCount,
            ProposalId,
            Hash,
            BoundedString,
        ),
    >
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api:
        GroupsRuntimeApi<Block, AccountId, GroupId, MemberCount, ProposalId, Hash, BoundedString>,
    GroupId: Codec + Copy + Send + Sync + 'static,
    MemberCount: Codec + Copy + Send + Sync + 'static,
    AccountId: Codec + Send + Sync + 'static,
    ProposalId: Codec + Copy + Send + Sync + 'static,
    Hash: Codec + Clone + Send + Sync + 'static + AsRef<[u8]>,
    BoundedString: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
{
    fn member_of(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<GroupId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.member_of(&at, account);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_group(
        &self,
        group_id: GroupId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<GroupResponse<GroupId, AccountId, MemberCount>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let (group, members) = api
            .get_group(&at, group_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok((group_id, group, members).into())
    }
    fn get_sub_groups(
        &self,
        group_id: GroupId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<GroupResponse<GroupId, AccountId, MemberCount>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let groups = api
            .get_sub_groups(&at, group_id)
            .map_err(convert_error!())?;

        Ok(groups
            .into_iter()
            .map(|(sub_group_id, group, members)| (sub_group_id, group, members).into())
            .collect())
    }

    fn get_proposal(
        &self,
        group_id: GroupId,
        proposal_id: ProposalId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<ProposalResponse<ProposalId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let (hash, proposal_len) = api
            .get_proposal(&at, group_id, proposal_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;

        Ok((proposal_id, hash, proposal_len).into())
    }

    fn get_proposals(
        &self,
        group_id: GroupId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<ProposalResponse<ProposalId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let proposals = api.get_proposals(&at, group_id).map_err(convert_error!())?;

        Ok(proposals
            .into_iter()
            .map(|(proposal_id, hash, proposal_len)| (proposal_id, hash, proposal_len).into())
            .collect())
    }

    fn get_voting(
        &self,
        group_id: GroupId,
        proposal_id: ProposalId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<VoteResponse<AccountId, ProposalId, MemberCount>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let vote = api
            .get_voting(&at, group_id, proposal_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok((proposal_id, vote).into())
    }
}
