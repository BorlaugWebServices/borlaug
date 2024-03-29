#![warn(missing_docs)]

use std::sync::Arc;

use futures::channel::mpsc::Sender;
use runtime::primitives::{
    AccountId, AssetId, AuditId, Balance, Block, BoundedStringFact, BoundedStringName, CatalogId,
    ClaimId, ControlPointId, DefinitionId, DefinitionStepIndex, EvidenceId, ExtrinsicIndex,
    GroupId, Hash, Index, LeaseId, MemberCount, ModuleIndex, Moment, ObservationId, ProcessId,
    ProposalId, RegistryId,
};
use runtime::BoundedStringUrl;
use sc_consensus_manual_seal::{
    rpc::{ManualSeal, ManualSealApi},
    EngineCommand,
};
pub use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_transaction_pool::TransactionPool;

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

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(deps: FullDeps<C, P>) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
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
    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        command_sink,
        client,
        ..
    } = deps;

    // Add the groups api
    io.extend_with(crate::groups_rpc::GroupsApi::to_delegate(
        crate::groups_rpc::Groups::new(client.clone()),
    ));
    // Add the provenance api
    io.extend_with(crate::provenance_rpc::ProvenanceApi::to_delegate(
        crate::provenance_rpc::Provenance::new(client.clone()),
    ));
    // Add the identity api
    io.extend_with(crate::identity_rpc::IdentityApi::to_delegate(
        crate::identity_rpc::Identity::new(client.clone()),
    ));
    // Add the audits api
    io.extend_with(crate::audits_rpc::AuditsApi::to_delegate(
        crate::audits_rpc::Audits::new(client.clone()),
    ));
    // Add the asset_registry api
    io.extend_with(crate::asset_registry_rpc::AssetRegistryApi::to_delegate(
        crate::asset_registry_rpc::AssetRegistry::new(client.clone()),
    ));
    // Add the settings api
    io.extend_with(crate::settings_rpc::SettingsApi::to_delegate(
        crate::settings_rpc::Settings::new(client),
    ));

    // The final RPC extension receives commands for the manual seal consensus engine.
    io.extend_with(
        // We provide the rpc handler with the sending end of the channel to allow the rpc
        // send EngineCommands to the background block authorship task.
        ManualSealApi::to_delegate(ManualSeal::new(command_sink)),
    );

    io
}
