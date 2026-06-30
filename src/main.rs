mod args;
mod boot;
mod config;
mod context;
mod outcome;
mod snapshot;
use args::Args;

use clap::Parser;
use outcome::AppError::InternalHashError;

use std::{
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{args::Commands, outcome::AppResult};

fn main() -> ExitCode {
    let arguments = Args::parse();

    match arguments.command {
        Some(Commands::Run) => match run_inner() {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        },
        None => ExitCode::SUCCESS,
    }
}

fn run_inner() -> AppResult<()> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| InternalHashError)?
        .as_nanos();
    let hash_string = format!("{:08x}", crc32fast::hash(&nanos.to_le_bytes()));
    Ok(())
}
