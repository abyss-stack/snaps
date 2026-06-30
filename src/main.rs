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
        None => {
            println!("{}", greet_user());
            ExitCode::SUCCESS
        }
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

fn greet_user() -> &'static str {
    r#"
  ____  ____   __ __  _____ _____        _____ ____    ____  ____    _____
 /    ||    \ |  |  |/ ___// ___/       / ___/|    \  /    ||    \  / ___/
|  o  ||  o  )|  |  (   \_(   \_  _____(   \_ |  _  ||  o  ||  o  )(   \_
|     ||     ||  ~  |\__  |\__  ||     |\__  ||  |  ||     ||   _/  \__  |
|  _  ||  O  ||___, |/  \ |/  \ ||_____|/  \ ||  |  ||  _  ||  |    /  \ |
|  |  ||     ||     |\    |\    |       \    ||  |  ||  |  ||  |    \    |
|__|__||_____||____/  \___| \___|        \___||__|__||__|__||__|     \___|

Part of abyss-stack tools: a state machine for Btrfs subvolumes (Static/Dynamic).
It implements a unique way to orchestrate your data, eliminating such common problems
as "My /home is from the future!" or "I've rolled back my database!".

You need to write your own rules in a config file, which is the single source of truth.
Abyss-snaps has a very simple UX, thanks to its config-driven architecture.
"#
    .trim_start_matches('\n')
}
