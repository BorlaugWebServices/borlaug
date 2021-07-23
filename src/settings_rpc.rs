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
    Perbill,
};
use std::sync::Arc;

#[rpc]
pub trait SettingsApi<BlockHash, ModuleIndex, ExtrinsicIndex, Balance> {
    #[rpc(name = "get_weight_to_fee_coefficients")]
    fn get_weight_to_fee_coefficients(
        &self,
        at: Option<BlockHash>,
    ) -> Result<WeightToFeeCoefficientsResponse>;

    #[rpc(name = "get_transaction_byte_fee")]
    fn get_transaction_byte_fee(&self, at: Option<BlockHash>)
        -> Result<TransactionByteFeeResponse>;

    #[rpc(name = "get_fee_split_ratio")]
    fn get_fee_split_ratio(&self, at: Option<BlockHash>) -> Result<FeeSplitRatioResponse>;

    #[rpc(name = "get_extrinsic_extra")]
    fn get_extrinsic_extra(
        &self,
        module_index: ModuleIndex,
        extrinsic_index: ExtrinsicIndex,
        at: Option<BlockHash>,
    ) -> Result<ExtrinsicExtraResponse>;
    #[rpc(name = "get_extrinsic_extras")]
    fn get_extrinsic_extras(
        &self,
        at: Option<BlockHash>,
    ) -> Result<ExtrinsicExtrasResponse<ModuleIndex, ExtrinsicIndex>>;
}

#[derive(Serialize, Deserialize)]
pub struct WeightToFeeCoefficientsResponse {
    //u64 instead of Balance due to bug in serde https://github.com/paritytech/substrate/issues/4641
    pub weight_to_fee_coefficients: Vec<(u64, Perbill, bool, u8)>,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionByteFeeResponse {
    //u64 instead of Balance due to bug in serde https://github.com/paritytech/substrate/issues/4641
    pub transaction_byte_fee: u64,
}

#[derive(Serialize, Deserialize)]
pub struct FeeSplitRatioResponse {
    pub fee_split_ratio: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ExtrinsicExtraResponse {
    //u64 instead of Balance due to bug in serde https://github.com/paritytech/substrate/issues/4641
    //TODO: change once bug is fixed
    pub fee: Option<u64>,
}
#[derive(Serialize, Deserialize)]
pub struct ExtrinsicExtrasResponse<ModuleIndex, ExtrinsicIndex> {
    //u64 instead of Balance due to bug in serde https://github.com/paritytech/substrate/issues/4641
    //TODO: change once bug is fixed
    pub fees: Vec<(ModuleIndex, Vec<(ExtrinsicIndex, u64)>)>,
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
    fn get_weight_to_fee_coefficients(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<WeightToFeeCoefficientsResponse> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let weight_to_fee_coefficients = api
            .get_weight_to_fee_coefficients(&at)
            .map_err(convert_error!())?;

        Ok(WeightToFeeCoefficientsResponse {
            weight_to_fee_coefficients,
        })
    }

    fn get_transaction_byte_fee(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<TransactionByteFeeResponse> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let transaction_byte_fee = api
            .get_transaction_byte_fee(&at)
            .map_err(convert_error!())?;
        Ok(TransactionByteFeeResponse {
            transaction_byte_fee: transaction_byte_fee.unique_saturated_into(),
        })
    }

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

    fn get_extrinsic_extra(
        &self,
        module_index: ModuleIndex,
        extrinsic_index: ExtrinsicIndex,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<ExtrinsicExtraResponse> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let fee = api
            .get_extrinsic_extra(&at, module_index, extrinsic_index)
            .map_err(convert_error!())?;

        Ok(ExtrinsicExtraResponse {
            fee: fee.map(|f| f.unique_saturated_into()),
        })
    }

    fn get_extrinsic_extras(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<ExtrinsicExtrasResponse<ModuleIndex, ExtrinsicIndex>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let fees = api.get_extrinsic_extras(&at).map_err(convert_error!())?;

        Ok(ExtrinsicExtrasResponse {
            fees: fees
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
