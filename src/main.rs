//! Borlaug CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;

fn main() -> sc_cli::Result<()> {
    let version = sc_cli::VersionInfo {
        name: "Borlaug",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "borlaug",
        author: "Borlaug Web Services",
        description: "Blockchain for self organizing communities",
        support_url: "https://gitlab.com/Borlaug/blockchain/bg/-/issues",
        copyright_start_year: 2019,
    };

    command::run(version)
}
