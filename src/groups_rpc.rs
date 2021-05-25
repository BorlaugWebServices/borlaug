use codec::Codec;
use groups_runtime_api::GroupsApi as GroupsRuntimeApi;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait GroupsApi<BlockHash, AccountId, GroupId> {
    #[rpc(name = "member_of")]
    fn member_of(&self, account: AccountId, at: Option<BlockHash>) -> Result<Vec<GroupId>>;
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

impl<C, Block, AccountId, GroupId> GroupsApi<<Block as BlockT>::Hash, AccountId, GroupId>
    for Groups<C, (Block, AccountId, GroupId)>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: GroupsRuntimeApi<Block, AccountId, GroupId>,
    GroupId: Codec + Send + Sync + 'static,
    AccountId: Codec + Send + Sync + 'static,
{
    fn member_of(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<GroupId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.member_of(&at, account);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
