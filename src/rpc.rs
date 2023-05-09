#![warn(missing_docs)]

use std::sync::Arc;

use futures::channel::mpsc::Sender;
use jsonrpsee::RpcModule;
use runtime::primitives::{
    AccountId, AssetId, AuditId, Balance, Block, BoundedStringFact, BoundedStringName, CatalogId,
    ClaimId, ControlPointId, DefinitionId, DefinitionStepIndex, EvidenceId, ExtrinsicIndex,
    GroupId, Hash, Index, LeaseId, MemberCount, ModuleIndex, Moment, ObservationId, ProcessId,
    ProposalId, RegistryId,
};
use runtime::BoundedStringUrl;
use sc_consensus_manual_seal::{
    rpc::{ManualSeal, ManualSealApiServer},
    EngineCommand,
};
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// Full client dependencies.
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// A command stream to send authoring commands to manual seal consensus engine
    pub command_sink: Sender<EngineCommand<Hash>>,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: BlockBuilder<Block>,
    C::Api: groups_runtime_api::GroupsApi<
        Block,
        AccountId,
        GroupId,
        MemberCount,
        ProposalId,
        Hash,
        BoundedStringName,
        Balance,
    >,
    C::Api: provenance_runtime_api::ProvenanceApi<
        Block,
        AccountId,
        RegistryId,
        DefinitionId,
        ProcessId,
        ProposalId,
        Moment,
        MemberCount,
        DefinitionStepIndex,
        BoundedStringName,
        BoundedStringFact,
    >,
    C::Api: identity_runtime_api::IdentityApi<
        Block,
        AccountId,
        CatalogId,
        ClaimId,
        MemberCount,
        Moment,
        BoundedStringName,
        BoundedStringFact,
    >,
    C::Api: audits_runtime_api::AuditsApi<
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
    C::Api: asset_registry_runtime_api::AssetRegistryApi<
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
    C::Api: settings_runtime_api::SettingsApi<Block, ModuleIndex, ExtrinsicIndex, Balance>,
    P: TransactionPool + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    use crate::asset_registry_rpc::{AssetRegistry, AssetRegistryApiServer};
    use crate::audits_rpc::{Audits, AuditsApiServer};
    use crate::groups_rpc::{Groups, GroupsApiServer};
    use crate::identity_rpc::{Identity, IdentityApiServer};
    use crate::provenance_rpc::{Provenance, ProvenanceApiServer};
    use crate::settings_rpc::{Settings, SettingsApiServer};

    let mut module = RpcModule::new(());
    let FullDeps {
        command_sink,
        client,
        pool,
        deny_unsafe,
    } = deps;

    module.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

    module.merge(Groups::new(client.clone()).into_rpc())?;
    module.merge(Provenance::new(client.clone()).into_rpc())?;
    module.merge(Identity::new(client.clone()).into_rpc())?;
    module.merge(Audits::new(client.clone()).into_rpc())?;
    module.merge(AssetRegistry::new(client.clone()).into_rpc())?;
    module.merge(Settings::new(client.clone()).into_rpc())?;

    // The final RPC extension receives commands for the manual seal consensus engine.
    module.merge(
        // We provide the rpc handler with the sending end of the channel to allow the rpc
        // send EngineCommands to the background block authorship task.
        ManualSealApiServer::to_delegate(ManualSeal::new(command_sink)).into_rpc(),
    )?;

    Ok(module)
}
