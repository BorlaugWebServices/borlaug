//! Borlaug CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod groups_rpc;
mod rpc;

fn main() -> sc_cli::Result<()> {
    command::run()
}
