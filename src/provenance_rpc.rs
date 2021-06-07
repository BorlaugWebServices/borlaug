use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use provenance_runtime_api::ProvenanceApi as ProvenanceRuntimeApi;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait ProvenanceApi<BlockHash, AccountId, RegistryId> {
    #[rpc(name = "get_registries")]
    fn get_registries(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Vec<Registry<RegistryId>>>;

    #[rpc(name = "get_registry")]
    fn get_registry(
        &self,
        account: AccountId,
        registry_id: RegistryId,
        at: Option<BlockHash>,
    ) -> Result<Registry<RegistryId>>;
}

#[derive(Serialize, Deserialize)]
pub struct Registry<RegistryId> {
    pub registry_id: RegistryId,
    pub name: String,
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

impl<C, Block, AccountId, RegistryId> ProvenanceApi<<Block as BlockT>::Hash, AccountId, RegistryId>
    for Provenance<C, (Block, AccountId, RegistryId)>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: ProvenanceRuntimeApi<Block, AccountId, RegistryId>,
    AccountId: Codec + Send + Sync + 'static,
    RegistryId: Codec + Copy + Send + Sync + 'static,
{
    fn get_registries(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<Registry<RegistryId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.get_registries(&at, account);
        let runtime_api_result = runtime_api_result.map(|registries| {
            registries
                .into_iter()
                .map(|(registry_id, registry)| Registry::<RegistryId> {
                    registry_id: registry_id,
                    name: String::from_utf8_lossy(&registry.name).to_string(),
                })
                .collect()
        });

        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(1),
            message: "Error in get_registries".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_registry(
        &self,
        account: AccountId,
        registry_id: RegistryId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Registry<RegistryId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_registry(&at, account, registry_id);
        let runtime_api_result = runtime_api_result.map(|registry| Registry::<RegistryId> {
            registry_id: registry_id,
            name: String::from_utf8_lossy(&registry.name).to_string(),
        });

        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(2),
            message: "Error in get_registry".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
