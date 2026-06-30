mod args;
mod boot;
mod config;
mod context;
mod outcome;
mod snapshot;
use args::Args;
use clap::Parser;
use context::AppContext;
use outcome::AppError::InternalHashError;

use crate::outcome::AppMessage::HashGenerated;
use crate::{args::Commands, outcome::AppResult};
use std::{
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() -> ExitCode {
    let arguments = Args::parse();
    let ctx = AppContext::from_args(arguments.raw);

    match arguments.command {
        Some(Commands::Run) => match run_inner(&ctx) {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        },
        None => ExitCode::SUCCESS,
    }
}

fn run_inner(ctx: &AppContext) -> AppResult<()> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| InternalHashError)?
        .as_nanos();
    let hash_string = format!("{:08x}", crc32fast::hash(&nanos.to_le_bytes()));

    ctx.emit_message(&HashGenerated(hash_string.clone()))?;

    Ok(())
}
