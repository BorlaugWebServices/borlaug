//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use runtime::{self, opaque::Block, RuntimeApi};
use polkadot_sdk::sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use polkadot_sdk::sc_telemetry::{Telemetry, TelemetryWorker};
use polkadot_sdk::sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use std::{sync::Arc};
use polkadot_sdk::sp_runtime::traits::Block as BlockT;
use polkadot_sdk::sc_network::{NetworkBackend};
use polkadot_sdk::sc_network_sync::{SyncingService, WarpSyncConfig};
use polkadot_sdk::sc_client_api::BlockBackend;

// Our native executor instance.
pub type HostFunctions = (
    polkadot_sdk::sp_io::SubstrateHostFunctions,
    polkadot_sdk::sp_statement_store::runtime_api::HostFunctions,
);

pub type RuntimeExecutor = polkadot_sdk::sc_executor::WasmExecutor<HostFunctions>;
pub type FullClient = polkadot_sdk::sc_service::TFullClient<Block, RuntimeApi, RuntimeExecutor>;
type FullBackend = polkadot_sdk::sc_service::TFullBackend<Block>;
type FullSelectChain = polkadot_sdk::sc_consensus::LongestChain<FullBackend, Block>;

const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

#[allow(dead_code)]
pub struct NewFullBase {
    pub task_manager: TaskManager,
    pub client: Arc<FullClient>,
    pub network: Arc<dyn polkadot_sdk::sc_network::service::traits::NetworkService>,
    pub sync: Arc<SyncingService<Block>>,
    pub transaction_pool: Arc<polkadot_sdk::sc_transaction_pool::TransactionPoolHandle<Block, FullClient>>,
    pub rpc_handlers: polkadot_sdk::sc_service::RpcHandlers,
}

pub fn new_partial(
    config: &Configuration,
) -> Result<
    polkadot_sdk::sc_service::PartialComponents<
        FullClient,
        FullBackend,
        FullSelectChain,
        polkadot_sdk::sc_consensus::DefaultImportQueue<Block>,
        polkadot_sdk::sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
        (
            polkadot_sdk::sc_consensus_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
            polkadot_sdk::sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
            Option<Telemetry>,
        ),
    >,
    ServiceError,
> {
    let telemetry = config.telemetry_endpoints.clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, polkadot_sdk::sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    // Use the new WasmExecutor instead of NativeElseWasmExecutor
    let executor = polkadot_sdk::sc_service::new_wasm_executor(&config.executor);

    let (client, backend, keystore_container, task_manager) =
        polkadot_sdk::sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = polkadot_sdk::sc_consensus::LongestChain::new(backend.clone());

    // Updated Transaction Pool Builder
    let transaction_pool = Arc::from(
        polkadot_sdk::sc_transaction_pool::Builder::new(
            task_manager.spawn_essential_handle(),
            client.clone(),
            config.role.is_authority().into(),
        )
            .with_options(config.transaction_pool.clone())
            .with_prometheus(config.prometheus_registry())
            .build(),
    );

    let (grandpa_block_import, grandpa_link) = polkadot_sdk::sc_consensus_grandpa::block_import(
        client.clone(),
        GRANDPA_JUSTIFICATION_PERIOD,
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let slot_duration = polkadot_sdk::sc_consensus_aura::slot_duration(&*client)?;

    let import_queue = polkadot_sdk::sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _>(
        polkadot_sdk::sc_consensus_aura::ImportQueueParams {
            block_import: grandpa_block_import.clone(),
            justification_import: Some(Box::new(grandpa_block_import.clone())),
            client: client.clone(),
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = polkadot_sdk::sp_timestamp::InherentDataProvider::from_system_time();
                let slot = polkadot_sdk::sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                    *timestamp,
                    slot_duration,
                );
                Ok((slot, timestamp))
            },
            spawner: &task_manager.spawn_essential_handle(),
            registry: config.prometheus_registry(),
            check_for_equivocation: Default::default(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            compatibility_mode: Default::default(),
        },
    )?;

    Ok(polkadot_sdk::sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (grandpa_block_import, grandpa_link, telemetry),
    })
}

/// Builds a new service for a full client.
pub fn new_full_base<N: NetworkBackend<Block, <Block as BlockT>::Hash>>(
    config: Configuration,
) -> Result<NewFullBase, ServiceError> {
    let polkadot_sdk::sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain:_,
        transaction_pool,
        other: (_block_import, grandpa_link, mut telemetry),
    } = new_partial(&config)?;

    // Initialize the new modular network configuration
    let mut net_config = polkadot_sdk::sc_network::config::FullNetworkConfiguration::<_, _, N>::new(
        &config.network,
        config.prometheus_config.as_ref().map(|cfg| cfg.registry.clone()),
    );

    let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
    let peer_store_handle = net_config.peer_store_handle();

    // Explicitly register the GRANDPA protocol
    let (grandpa_protocol_config, _grandpa_notification_service) =
        polkadot_sdk::sc_consensus_grandpa::grandpa_peers_set_config::<_, N>(
            polkadot_sdk::sc_consensus_grandpa::protocol_standard_name(&genesis_hash, &config.chain_spec),
            N::register_notification_metrics(config.prometheus_config.as_ref().map(|cfg| &cfg.registry)),
            Arc::clone(&peer_store_handle),
        );
    net_config.add_notification_protocol(grandpa_protocol_config);


    let rpc_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();

        Box::new(move | _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                deny_unsafe: polkadot_sdk::sc_rpc::DenyUnsafe::No,
            };
            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    let (network, system_rpc_tx, tx_handler_controller, sync_service) =
        polkadot_sdk::sc_service::build_network(polkadot_sdk::sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_config: Some(WarpSyncConfig::WithProvider(Arc::new(
                polkadot_sdk::sc_consensus_grandpa::warp_proof::NetworkProvider::new(
                    backend.clone(),
                    grandpa_link.shared_authority_set().clone(),
                    Vec::default(),
                ),
            ))),
            block_relay: None,
            metrics: N::register_notification_metrics(config.prometheus_config.as_ref().map(|cfg| &cfg.registry)),
        })?;

    // 1. Call spawn_tasks to start RPC, Telemetry, and other background services
    let rpc_handlers = polkadot_sdk::sc_service::spawn_tasks(polkadot_sdk::sc_service::SpawnTasksParams {
        config,
        backend: backend.clone(),
        client: client.clone(),
        keystore: keystore_container.keystore(),
        network: network.clone(),
        rpc_builder: Box::new(rpc_builder),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        system_rpc_tx,
        tx_handler_controller,
        sync_service: sync_service.clone(),
        telemetry: telemetry.as_mut(),
        tracing_execute_block: None,
    })?;

    Ok(NewFullBase {
        task_manager,
        client,
        network,
        sync: sync_service,
        transaction_pool,
        rpc_handlers,
    })
}

pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
    match config.network.network_backend {
        polkadot_sdk::sc_network::config::NetworkBackendType::Libp2p => {
            new_full_base::<polkadot_sdk::sc_network::NetworkWorker<_, _>>(config)
                .map(|base| base.task_manager)
        },
        polkadot_sdk::sc_network::config::NetworkBackendType::Litep2p => {
            new_full_base::<polkadot_sdk::sc_network::Litep2pNetworkBackend>(config)
                .map(|base| base.task_manager)
        },
    }
}