use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};
use settings_runtime_api::SettingsApi as SettingsRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{AtLeast32BitUnsigned, Block as BlockT},
};
use std::sync::Arc;

#[rpc]
pub trait SettingsApi<BlockHash, ModuleIndex, ExtrinsicIndex, Balance> {
    #[rpc(name = "get_fee_split_ratio")]
    fn get_fee_split_ratio(&self, at: Option<BlockHash>) -> Result<FeeSplitRatioResponse>;

    #[rpc(name = "get_extrinsic_extras")]
    fn get_extrinsic_extras(
        &self,
        at: Option<BlockHash>,
    ) -> Result<ExtrinsicExtraResponse<ModuleIndex, ExtrinsicIndex>>;
}

#[derive(Serialize, Deserialize)]
pub struct FeeSplitRatioResponse {
    pub fee_split_ratio: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ExtrinsicExtraResponse<ModuleIndex, ExtrinsicIndex> {
    //u64 instead of Balance due to bug in serde https://github.com/paritytech/substrate/issues/4641
    //TODO: change once bug is fixed
    pub data: Vec<(ModuleIndex, Vec<(ExtrinsicIndex, u64)>)>,
}

pub struct Settings<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Settings<C, M> {
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
            message: "Error in Settings API".into(),
            data: Some(format!("{:?}", e).into()),
        }
    }};
}

impl<C, Block, ModuleIndex, ExtrinsicIndex, Balance>
    SettingsApi<<Block as BlockT>::Hash, ModuleIndex, ExtrinsicIndex, Balance>
    for Settings<C, (Block, ModuleIndex, ExtrinsicIndex, Balance)>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: SettingsRuntimeApi<Block, ModuleIndex, ExtrinsicIndex, Balance>,
    ModuleIndex: Codec + Send + Sync + 'static,
    ExtrinsicIndex: Codec + Copy + Send + Sync + 'static,
    Balance: Codec + Copy + Send + Sync + AtLeast32BitUnsigned + 'static,
{
    fn get_fee_split_ratio(
        &self,

        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<FeeSplitRatioResponse> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let fee_split_ratio = api.get_fee_split_ratio(&at).map_err(convert_error!())?;
        Ok(FeeSplitRatioResponse { fee_split_ratio })
    }

    fn get_extrinsic_extras(
        &self,

        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<ExtrinsicExtraResponse<ModuleIndex, ExtrinsicIndex>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let data = api.get_extrinsic_extras(&at).map_err(convert_error!())?;

        Ok(ExtrinsicExtraResponse {
            data: data
                .into_iter()
                .map(|(mi, e)| {
                    (
                        mi,
                        e.into_iter()
                            .map(|(ei, b)| (ei, b.unique_saturated_into()))
                            .collect(),
                    )
                })
                .collect(),
        })
    }
}
