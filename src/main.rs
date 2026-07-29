#![allow(clippy::uninlined_format_args)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

mod cli;
mod core;
mod outcome;

use std::process::ExitCode;

use clap::Parser;

use crate::outcome::{AppError, AppMessage, AppResult};
use crate::cli::{AppArgs, Commands};
use crate::core::recipe::Recipe;

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            e.emit();
            ExitCode::FAILURE
        }
    }
}

fn run() -> AppResult<()> {
    let args = AppArgs::parse();
    Ok(())
}

