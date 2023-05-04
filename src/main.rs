//! Borlaug CLI library.
#![warn(missing_docs)]

#[cfg_attr(feature = "grandpa_babe", path = "chain_spec_grandpa_babe.rs")]
#[cfg_attr(feature = "grandpa_aura", path = "chain_spec_grandpa_aura.rs")]
mod chain_spec;
#[macro_use]
#[cfg_attr(feature = "grandpa_babe", path = "service_grandpa_babe.rs")]
#[cfg_attr(feature = "grandpa_aura", path = "service_grandpa_aura.rs")]
mod service;
mod asset_registry_rpc;
mod audits_rpc;
mod benchmarking;
mod cli;
mod command;
mod groups_rpc;
mod identity_rpc;
mod provenance_rpc;
#[cfg_attr(feature = "grandpa_babe", path = "rpc_grandpa_babe.rs")]
#[cfg_attr(feature = "grandpa_aura", path = "rpc_grandpa_aura.rs")]
mod rpc;
mod settings_rpc;

fn main() -> sc_cli::Result<()> {
    command::run()
}
