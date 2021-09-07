use crate::identity_rpc::{Did, FactResponse};
use asset_registry_runtime_api::AssetRegistryApi as AssetRegistryRuntimeApi;
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_primitives::{
    Asset, AssetAllocation, AssetProperty, AssetStatus, LeaseAgreement, Registry,
};
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
    generic::BlockId,
    traits::{AtLeast32BitUnsigned, Block as BlockT},
};
use std::sync::Arc;

#[rpc]
pub trait AssetRegistryApi<
    BlockHash,
    AccountId,
    ProposalId,
    RegistryId,
    AssetId,
    LeaseId,
    Moment,
    Balance,
    BoundedStringName,
    BoundedStringFact,
>
{
    #[rpc(name = "get_asset_registries")]
    fn get_registries(
        &self,
        did: Did,
        at: Option<BlockHash>,
    ) -> Result<Vec<RegistryResponse<RegistryId>>>;
    #[rpc(name = "get_asset_registry")]
    fn get_registry(
        &self,
        did: Did,
        registry_id: RegistryId,
        at: Option<BlockHash>,
    ) -> Result<RegistryResponse<RegistryId>>;
    #[rpc(name = "get_assets")]
    fn get_assets(
        &self,
        registry_id: RegistryId,
        at: Option<BlockHash>,
    ) -> Result<Vec<AssetResponse<AssetId, Moment>>>;
    #[rpc(name = "get_asset")]
    fn get_asset(
        &self,
        registry_id: RegistryId,
        asset_id: AssetId,
        at: Option<BlockHash>,
    ) -> Result<AssetResponse<AssetId, Moment>>;
    #[rpc(name = "get_leases")]
    fn get_leases(
        &self,
        lessor: Did,
        at: Option<BlockHash>,
    ) -> Result<Vec<LeaseAgreementResponse<LeaseId, ProposalId, RegistryId, AssetId, Moment>>>;
    #[rpc(name = "get_lease")]
    fn get_lease(
        &self,
        lessor: Did,
        lease_id: LeaseId,
        at: Option<BlockHash>,
    ) -> Result<LeaseAgreementResponse<LeaseId, ProposalId, RegistryId, AssetId, Moment>>;
    #[rpc(name = "get_lease_allocations")]
    fn get_lease_allocations(
        &self,
        registry_id: RegistryId,
        asset_id: AssetId,
        at: Option<BlockHash>,
    ) -> Result<Vec<LeaseAllocationResponse<LeaseId, Moment>>>;
}

#[derive(Serialize, Deserialize)]
pub struct RegistryResponse<RegistryId> {
    pub registry_id: RegistryId,
    pub name: String,
}

impl<RegistryId, BoundedStringName> From<(RegistryId, Registry<BoundedStringName>)>
    for RegistryResponse<RegistryId>
where
    BoundedStringName: Into<Vec<u8>>,
{
    fn from((registry_id, registry): (RegistryId, Registry<BoundedStringName>)) -> Self {
        RegistryResponse {
            registry_id,
            name: String::from_utf8_lossy(&registry.name.into()).to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssetResponse<AssetId, Moment> {
    pub asset_id: AssetId,
    pub properties: Vec<AssetPropertyResponse>,
    pub name: String,
    pub asset_number: Option<String>,
    pub status: String,
    pub serial_number: Option<String>,
    pub total_shares: u64,
    //u64 instead of Balance due to bug in serde https://github.com/paritytech/substrate/issues/4641
    pub residual_value: Option<u64>,
    //u64 instead of Balance due to bug in serde https://github.com/paritytech/substrate/issues/4641
    pub purchase_value: Option<u64>,
    pub acquired_date: Option<Moment>,
}

impl<AssetId, Moment, Balance, BoundedStringName, BoundedStringFact>
    From<(
        AssetId,
        Asset<Moment, Balance, BoundedStringName, BoundedStringFact>,
    )> for AssetResponse<AssetId, Moment>
where
    BoundedStringName: Into<Vec<u8>>,
    BoundedStringFact: Into<Vec<u8>>,
    Balance: AtLeast32BitUnsigned,
{
    fn from(
        (asset_id, asset): (
            AssetId,
            Asset<Moment, Balance, BoundedStringName, BoundedStringFact>,
        ),
    ) -> Self {
        AssetResponse {
            asset_id,
            properties: asset
                .properties
                .into_iter()
                .map(|property| property.into())
                .collect(),
            name: String::from_utf8_lossy(&asset.name.into()).to_string(),
            asset_number: asset
                .asset_number
                .map(|str| String::from_utf8_lossy(&str.into()).to_string()),
            status: match asset.status {
                AssetStatus::Draft => "Draft".to_string(),
                AssetStatus::Active => "Active".to_string(),
                AssetStatus::InActive => "InActive".to_string(),
            },
            serial_number: asset
                .serial_number
                .map(|str| String::from_utf8_lossy(&str.into()).to_string()),
            total_shares: asset.total_shares,
            residual_value: asset.residual_value.map(|v| v.unique_saturated_into()),
            purchase_value: asset.purchase_value.map(|v| v.unique_saturated_into()),
            acquired_date: asset.acquired_date,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssetPropertyResponse {
    pub name: String,
    pub fact: FactResponse,
}
impl<BoundedStringName, BoundedStringFact> From<AssetProperty<BoundedStringName, BoundedStringFact>>
    for AssetPropertyResponse
where
    BoundedStringName: Into<Vec<u8>>,
    BoundedStringFact: Into<Vec<u8>>,
{
    fn from(property: AssetProperty<BoundedStringName, BoundedStringFact>) -> Self {
        AssetPropertyResponse {
            name: String::from_utf8_lossy(&property.name.into()).to_string(),
            fact: property.fact.into(),
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct LeaseAgreementResponse<LeaseId, ProposalId, RegistryId, AssetId, Moment> {
    pub lease_id: LeaseId,
    pub proposal_id: Option<ProposalId>,
    pub contract_number: String,
    pub lessor: Did,
    pub lessee: Did,
    pub effective_ts: Moment,
    pub expiry_ts: Moment,
    pub allocations: Vec<AssetAllocationResponse<RegistryId, AssetId>>,
}

impl<LeaseId, ProposalId, RegistryId, AssetId, Moment, BoundedStringName>
    From<(
        LeaseId,
        LeaseAgreement<ProposalId, RegistryId, AssetId, Moment, BoundedStringName>,
    )> for LeaseAgreementResponse<LeaseId, ProposalId, RegistryId, AssetId, Moment>
where
    BoundedStringName: Into<Vec<u8>>,
{
    fn from(
        (lease_id, lease): (
            LeaseId,
            LeaseAgreement<ProposalId, RegistryId, AssetId, Moment, BoundedStringName>,
        ),
    ) -> Self {
        LeaseAgreementResponse {
            lease_id,
            proposal_id: lease.proposal_id,
            contract_number: String::from_utf8_lossy(&lease.contract_number.into()).to_string(),
            lessor: lease.lessor.into(),
            lessee: lease.lessee.into(),
            effective_ts: lease.effective_ts,
            expiry_ts: lease.expiry_ts,
            allocations: lease
                .allocations
                .into_iter()
                .map(|allocation| allocation.into())
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssetAllocationResponse<RegistryId, AssetId> {
    pub registry_id: RegistryId,
    pub asset_id: AssetId,
    pub allocated_shares: u64,
}
impl<RegistryId, AssetId> From<AssetAllocation<RegistryId, AssetId>>
    for AssetAllocationResponse<RegistryId, AssetId>
{
    fn from(allocation: AssetAllocation<RegistryId, AssetId>) -> Self {
        AssetAllocationResponse {
            registry_id: allocation.registry_id,
            asset_id: allocation.asset_id,
            allocated_shares: allocation.allocated_shares,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LeaseAllocationResponse<LeaseId, Moment> {
    pub lease_id: LeaseId,
    pub allocation: u64,
    pub expiry: Moment,
}

impl<LeaseId, Moment> From<(LeaseId, u64, Moment)> for LeaseAllocationResponse<LeaseId, Moment> {
    fn from((lease_id, allocation, expiry): (LeaseId, u64, Moment)) -> Self {
        LeaseAllocationResponse {
            lease_id,
            allocation,
            expiry,
        }
    }
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

impl<
        C,
        Block,
        AccountId,
        ProposalId,
        RegistryId,
        AssetId,
        LeaseId,
        Moment,
        Balance,
        BoundedStringName,
        BoundedStringFact,
    >
    AssetRegistryApi<
        <Block as BlockT>::Hash,
        AccountId,
        ProposalId,
        RegistryId,
        AssetId,
        LeaseId,
        Moment,
        Balance,
        BoundedStringName,
        BoundedStringFact,
    >
    for AssetRegistry<
        C,
        (
            Block,
            AccountId,
            ProposalId,
            RegistryId,
            AssetId,
            LeaseId,
            Moment,
            Balance,
            BoundedStringName,
            BoundedStringFact,
        ),
    >
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: AssetRegistryRuntimeApi<
        Block,
        AccountId,
        ProposalId,
        RegistryId,
        AssetId,
        LeaseId,
        Moment,
        Balance,
        BoundedStringName,
        BoundedStringFact,
    >,
    AccountId: Codec + Send + Sync + 'static,
    ProposalId: Codec + Send + Sync + 'static,
    RegistryId: Codec + Copy + Send + Sync + 'static,
    AssetId: Codec + Copy + Send + Sync + 'static,
    LeaseId: Codec + Copy + Send + Sync + 'static,
    Moment: Codec + Copy + Send + Sync + 'static,
    Balance: Codec + Copy + Send + Sync + AtLeast32BitUnsigned + 'static,
    BoundedStringName: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
    BoundedStringFact: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
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
    fn get_registry(
        &self,
        did: Did,
        registry_id: RegistryId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<RegistryResponse<RegistryId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let registry = api
            .get_registry(&at, did.into(), registry_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok((registry_id, registry).into())
    }

    fn get_assets(
        &self,
        registry_id: RegistryId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<AssetResponse<AssetId, Moment>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let assets = api.get_assets(&at, registry_id).map_err(convert_error!())?;
        Ok(assets
            .into_iter()
            .map(|(asset_id, asset)| (asset_id, asset).into())
            .collect())
    }
    fn get_asset(
        &self,
        registry_id: RegistryId,
        asset_id: AssetId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<AssetResponse<AssetId, Moment>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let asset = api
            .get_asset(&at, registry_id, asset_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok((asset_id, asset).into())
    }

    fn get_leases(
        &self,
        lessor: Did,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<LeaseAgreementResponse<LeaseId, ProposalId, RegistryId, AssetId, Moment>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let leases = api
            .get_leases(&at, lessor.into())
            .map_err(convert_error!())?;
        Ok(leases
            .into_iter()
            .map(|(asset_id, asset)| (asset_id, asset).into())
            .collect())
    }

    fn get_lease(
        &self,
        lessor: Did,
        lease_id: LeaseId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<LeaseAgreementResponse<LeaseId, ProposalId, RegistryId, AssetId, Moment>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let lease = api
            .get_lease(&at, lessor.into(), lease_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok((lease_id, lease).into())
    }

    fn get_lease_allocations(
        &self,
        registry_id: RegistryId,
        asset_id: AssetId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<LeaseAllocationResponse<LeaseId, Moment>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let lease_allocations = api
            .get_lease_allocations(&at, registry_id, asset_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok(lease_allocations
            .into_iter()
            .map(|lease_allocation| lease_allocation.into())
            .collect())
    }
}
