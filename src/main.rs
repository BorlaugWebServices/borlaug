//! Borlaug CLI library.
#![warn(missing_docs)]

#[cfg_attr(feature = "grandpa_babe", path = "chain_spec_grandpa_babe.rs")]
mod chain_spec;
#[macro_use]
#[cfg_attr(feature = "grandpa_babe", path = "service_grandpa_babe.rs")]
mod service;
mod cli;
#[cfg_attr(feature = "grandpa_babe", path = "command_grandpa_babe.rs")]
mod command;
mod groups_rpc;
#[cfg_attr(feature = "grandpa_babe", path = "rpc_grandpa_babe.rs")]
mod rpc;

fn main() -> sc_cli::Result<()> {
    command::run()
}
