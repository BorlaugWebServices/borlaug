use polkadot_sdk::sc_cli::RunCmd;

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub run: RunCmd,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Key management cli utilities
    #[command(subcommand)]
    Key(polkadot_sdk::sc_cli::KeySubcommand),

    /// Build a chain specification.
    BuildSpec(polkadot_sdk::sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(polkadot_sdk::sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(polkadot_sdk::sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(polkadot_sdk::sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(polkadot_sdk::sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(polkadot_sdk::sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(polkadot_sdk::sc_cli::RevertCmd),

    /// Sub-commands concerned with benchmarking.
    #[command(subcommand)]
    Benchmark(polkadot_sdk::frame_benchmarking_cli::BenchmarkCmd),

    /// Try some command against runtime state.
    #[cfg(feature = "try-runtime")]
    TryRuntime(polkadot_sdk::try_runtime_cli::TryRuntimeCmd),

    /// Try some command against runtime state. Note: `try-runtime` feature must be enabled.
    #[cfg(not(feature = "try-runtime"))]
    TryRuntime,

    /// Db meta columns information.
    ChainInfo(polkadot_sdk::sc_cli::ChainInfoCmd),
}
