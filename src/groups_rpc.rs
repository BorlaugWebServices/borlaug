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
    fn member_of(
        &self,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Vec<GroupResponse<GroupId, AccountId, MemberCount>>>;

    #[rpc(name = "is_member")]
    fn is_member(
        &self,
        group_id: GroupId,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> Result<bool>;

    #[rpc(name = "get_group_account")]
    fn get_group_account(&self, group_id: GroupId, at: Option<BlockHash>) -> Result<AccountId>;

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
    ) -> Result<ProposalResponse<ProposalId, AccountId, MemberCount>>;

    #[rpc(name = "get_proposals_by_group")]
    fn get_proposals_by_group(
        &self,
        group_id: GroupId,
        at: Option<BlockHash>,
    ) -> Result<Vec<ProposalResponse<ProposalId, AccountId, MemberCount>>>;

    #[rpc(name = "get_proposals_by_account")]
    fn get_proposals_by_account(
        &self,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> Result<
        Vec<(
            GroupId,
            Vec<ProposalResponse<ProposalId, AccountId, MemberCount>>,
        )>,
    >;
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
pub struct ProposalResponse<ProposalId, AccountId, MemberCount> {
    pub proposal_id: ProposalId,
    pub hash: Option<String>,
    pub proposal_len: Option<u32>,
    pub votes: VotesResponse<AccountId, MemberCount>,
}
impl<ProposalId, Hash, AccountId, MemberCount>
    From<(
        ProposalId,
        Option<(Hash, u32)>,
        Votes<AccountId, MemberCount>,
    )> for ProposalResponse<ProposalId, AccountId, MemberCount>
where
    Hash: AsRef<[u8]>,
{
    fn from(
        (proposal_id, proposal, votes): (
            ProposalId,
            Option<(Hash, u32)>,
            Votes<AccountId, MemberCount>,
        ),
    ) -> Self {
        ProposalResponse {
            proposal_id,
            hash: proposal
                .as_ref()
                .map(|(hash, _len)| String::from_utf8_lossy(hash.as_ref()).to_string()),
            proposal_len: proposal.map(|(_hash, len)| len),
            votes: votes.into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VotesResponse<AccountId, MemberCount> {
    pub threshold: MemberCount,
    pub total_vote_weight: MemberCount,
    pub ayes: Vec<(AccountId, MemberCount)>,
    pub nays: Vec<(AccountId, MemberCount)>,
}
impl<AccountId, MemberCount> From<Votes<AccountId, MemberCount>>
    for VotesResponse<AccountId, MemberCount>
{
    fn from(votes: Votes<AccountId, MemberCount>) -> Self {
        VotesResponse {
            threshold: votes.threshold,
            total_vote_weight: votes.total_vote_weight,
            ayes: votes.ayes,
            nays: votes.nays,
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
            message: "Error in Groups API".into(),
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
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<GroupResponse<GroupId, AccountId, MemberCount>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let groups = api.member_of(&at, account_id).map_err(convert_error!())?;
        Ok(groups.into_iter().map(|g| g.into()).collect())
    }

    fn is_member(
        &self,
        group_id: GroupId,
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<bool> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let is_member = api
            .is_member(&at, group_id, account_id)
            .map_err(convert_error!())?;
        Ok(is_member)
    }

    fn get_group_account(
        &self,
        group_id: GroupId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<AccountId> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let account_id = api
            .get_group_account(&at, group_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok(account_id)
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
    ) -> Result<ProposalResponse<ProposalId, AccountId, MemberCount>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let (proposal_id, proposal, votes) = api
            .get_proposal(&at, group_id, proposal_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;

        Ok((proposal_id, proposal, votes).into())
    }

    fn get_proposals_by_group(
        &self,
        group_id: GroupId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<ProposalResponse<ProposalId, AccountId, MemberCount>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let proposals = api
            .get_proposals_by_group(&at, group_id)
            .map_err(convert_error!())?;

        Ok(proposals
            .into_iter()
            .map(|(proposal_id, proposal, votes)| (proposal_id, proposal, votes).into())
            .collect())
    }

    fn get_proposals_by_account(
        &self,
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<
        Vec<(
            GroupId,
            Vec<ProposalResponse<ProposalId, AccountId, MemberCount>>,
        )>,
    > {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let proposals = api
            .get_proposals_by_account(&at, account_id)
            .map_err(convert_error!())?;

        Ok(proposals
            .into_iter()
            .map(|(group_id, proposals)| {
                (
                    group_id,
                    proposals
                        .into_iter()
                        .map(|(proposal_id, proposal, votes)| (proposal_id, proposal, votes).into())
                        .collect(),
                )
            })
            .collect())
    }
}
