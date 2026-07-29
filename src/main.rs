#![allow(clippy::uninlined_format_args)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

mod cli;
mod core;
mod outcome;

use std::process::ExitCode;
use std::path::Path;

use clap::Parser;
use nix::unistd::getuid;

use crate::core::fstab::burn_fstab;
use crate::outcome::{AppError, AppMessage, AppResult};
use crate::cli::{AppArgs, Commands};
use crate::core::recipe::Recipe;
use crate::core::flags::toggle_rdonly_flag;

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

    match args.command {
        Commands::RecipeTemplate => println!("{}", Recipe::TEMPLATE),
        Commands::BurnFstab {
            content, // Fstab content.
            target, // A bootable subvolume.
            fstab_path, // Relative, default to "etc/fstab".
            set_rdonly // To leave the subvolume read-only after burning.
        } => {
            if !getuid().is_root() {
                return Err(AppError::RootRequired);
            }

            // INTENTIONAL: Relative path moved here and can`t be used anymore.
            let fstab_full_path = target.join(fstab_path);

            // NOTE: Ensure target is read-write before burning fstab.
            toggle_rdonly_flag(&target, false)?;

            burn_fstab(&fstab_full_path, &content)?;

            if set_rdonly {
                toggle_rdonly_flag(&target, true)?;
            }
        }
        
        
        _=>{}/////
    }
    Ok(())
}

