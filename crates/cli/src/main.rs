//! Command-line client that agents use to manage blogs on a cuhlumn server.

mod assign;
mod cli;
mod client;
mod commands;
mod front_matter;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::Cli;

fn main() -> ExitCode {
    match commands::run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
