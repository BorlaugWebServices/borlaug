use crate::identity_rpc::Did;
use asset_registry_runtime_api::AssetRegistryApi as AssetRegistryRuntimeApi;
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
// use pallet_primitives::Registry;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait AssetRegistryApi<BlockHash, AccountId, RegistryId, AssetId, LeaseId, BoundedStringName> {
    #[rpc(name = "get_registries")]
    fn get_registries(
        &self,
        did: Did,
        at: Option<BlockHash>,
    ) -> Result<Vec<RegistryResponse<RegistryId>>>;
}

#[derive(Serialize, Deserialize)]
pub struct RegistryResponse<RegistryId> {
    pub registry_id: RegistryId,
    pub name: String,
}

pub struct AssetRegistry<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> AssetRegistry<C, M> {
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
            message: "Error in AssetRegistry API".into(),
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

impl<C, Block, AccountId, RegistryId, AssetId, LeaseId, BoundedStringName>
    AssetRegistryApi<
        <Block as BlockT>::Hash,
        AccountId,
        RegistryId,
        AssetId,
        LeaseId,
        BoundedStringName,
    >
    for AssetRegistry<
        C,
        (
            Block,
            AccountId,
            RegistryId,
            AssetId,
            LeaseId,
            BoundedStringName,
        ),
    >
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api:
        AssetRegistryRuntimeApi<Block, AccountId, RegistryId, AssetId, LeaseId, BoundedStringName>,
    AccountId: Codec + Send + Sync + 'static,
    RegistryId: Codec + Copy + Send + Sync + 'static,
    AssetId: Codec + Copy + Send + Sync + 'static,
    LeaseId: Codec + Copy + Send + Sync + 'static,
    BoundedStringName: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
{
    fn get_registries(
        &self,
        did: Did,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<RegistryResponse<RegistryId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let registries = api
            .get_registries(&at, did.into())
            .map_err(convert_error!())?;
        Ok(registries
            .into_iter()
            .map(|(registry_id, registry)| RegistryResponse::<RegistryId> {
                registry_id,
                name: String::from_utf8_lossy(&registry.name.into()).to_string(),
            })
            .collect())
    }
}
