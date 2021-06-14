use codec::Codec;
use groups_runtime_api::GroupsApi as GroupsRuntimeApi;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_primitives::Group;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait GroupsApi<BlockHash, AccountId, GroupId, MemberCount> {
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
}

#[derive(Serialize, Deserialize)]
pub struct GroupResponse<GroupId, AccountId, MemberCount> {
    pub group_id: GroupId,
    pub name: String,
    pub members: Vec<AccountId>,
    pub threshold: MemberCount,
    pub funding_account: AccountId,
    pub anonymous_account: AccountId,
    pub parent: Option<GroupId>,
}
impl<GroupId, AccountId, MemberCount> From<(GroupId, Group<GroupId, AccountId, MemberCount>)>
    for GroupResponse<GroupId, AccountId, MemberCount>
{
    fn from((group_id, group): (GroupId, Group<GroupId, AccountId, MemberCount>)) -> Self {
        GroupResponse {
            group_id,
            name: String::from_utf8_lossy(&group.name).to_string(),
            members: group.members,
            threshold: group.threshold,
            funding_account: group.funding_account,
            anonymous_account: group.anonymous_account,
            parent: group.parent,
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

impl<C, Block, AccountId, GroupId, MemberCount>
    GroupsApi<<Block as BlockT>::Hash, AccountId, GroupId, MemberCount>
    for Groups<C, (Block, AccountId, GroupId, MemberCount)>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: GroupsRuntimeApi<Block, AccountId, GroupId, MemberCount>,
    GroupId: Codec + Copy + Send + Sync + 'static,
    MemberCount: Codec + Copy + Send + Sync + 'static,
    AccountId: Codec + Send + Sync + 'static,
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

        let group = api
            .get_group(&at, group_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok((group_id, group).into())
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
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;

        Ok(groups
            .into_iter()
            .map(|(sub_group_id, group)| (sub_group_id, group).into())
            .collect())
    }
}
