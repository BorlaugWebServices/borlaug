// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A collection of node-specific RPC methods.
//!
//! Since `substrate` core functionality makes no assumptions
//! about the modules used inside the runtime, so do
//! RPC methods defined in `sc-rpc` crate.
//! It means that `client/rpc` can't have any methods that
//! need some strong assumptions about the particular runtime.
//!
//! The RPCs available in this crate however can make some assumptions
//! about how the runtime is constructed and what FRAME pallets
//! are part of it. Therefore all node-runtime-specific RPCs can
//! be placed here or imported from corresponding FRAME RPC definitions.

#![warn(missing_docs)]

use std::sync::Arc;

use jsonrpsee::RpcModule;
use runtime::primitives::{
    AccountId, AssetId, AuditId, Balance, Block, BoundedStringFact, BoundedStringName, CatalogId,
    ClaimId, ControlPointId, DefinitionId, DefinitionStepIndex, EvidenceId, ExtrinsicIndex,
    GroupId, Hash, Index, LeaseId, MemberCount, ModuleIndex, Moment, ObservationId, ProcessId,
    ProposalId, RegistryId,
};
use runtime::BoundedStringUrl;
use sc_client_api::AuxStore;
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
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + AuxStore
        + HeaderMetadata<Block, Error = BlockChainError>
        + Sync
        + Send
        + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
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
        client,
        pool,
        deny_unsafe,
    } = deps;

    module.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

    module.merge(Groups::new(client.clone()).into_rpc())?;
    module.merge(Groups::new(client.clone()).into_rpc())?;
    module.merge(Provenance::new(client.clone()).into_rpc())?;
    module.merge(Identity::new(client.clone()).into_rpc())?;
    module.merge(Audits::new(client.clone()).into_rpc())?;
    module.merge(AssetRegistry::new(client.clone()).into_rpc())?;
    module.merge(Settings::new(client.clone()).into_rpc())?;

    Ok(module)
}
