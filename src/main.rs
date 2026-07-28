#![allow(clippy::uninlined_format_args)]
//#![deny(clippy::unwrap_used)]
//#![deny(clippy::expect_used)]

mod args;
mod core;
mod outcome;

use crate::args::{AppArgs, Commands};
use crate::core::deploy::create_snapshots;
use crate::core::flags::toggle_rdonly_flag;
use crate::core::fstab::{brew_fstab, burn_fstab};
use crate::core::recipe::Recipe;
use crate::core::rollback::rollback;
use crate::outcome::{AppError, AppMessage, AppResult};
use clap::Parser;
use nix::unistd::getuid;
use std::path::Path;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

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
        Commands::RecipeTemplate => {
            println!("{}", Recipe::TEMPLATE);
        }
        Commands::BurnFstab {
            source,
            target,
            set_rdonly,
        } => {
            if !getuid().is_root() {
                return Err(AppError::RootRequired);
            }
            let content =
                std::fs::read_to_string(&source).map_err(|err| AppError::FstabReadError {
                    path: source.to_path_buf(),
                    what: err.to_string(),
                })?;
            let fstab_path = target.join(args.fstab_rel);

            toggle_rdonly_flag(&target, false)?;
            burn_fstab(&fstab_path, &content)?;

            if set_rdonly {
                toggle_rdonly_flag(&target, true)?;
            }
        }
        Commands::Run { prefix } => {
            if !getuid().is_root() {
                return Err(AppError::RootRequired);
            }

            let prefix_value: String = match prefix {
                Some(p) => p.to_string(),
                None => {
                    #[allow(clippy::expect_used, reason = "1970-01-01 is always in the past.")]
                    let nanos = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("timestamp_fail")
                        .as_nanos();

                    format!("{:08x}", crc32fast::hash(&nanos.to_le_bytes()))
                }
            };
            
            AppMessage::UsingPrefix {
                prefix: prefix_value.clone(),
            }
            .emit();
            let recipe_path = Path::new(&args.recipe);
            let recipe = Recipe::load(recipe_path)?;
            let fstab_content = brew_fstab(&recipe, Some(&prefix_value));
            if recipe.btrfs_layout.is_none() {
                println!("{}", fstab_content);
                AppMessage::FstabEmitted {
                    length: fstab_content.len(),
                }
                .emit();

                return Ok(());
            }

            let bootable = create_snapshots(&recipe, &prefix_value)?;

            match (bootable, args.emit_fstab) {
                (Some(bootable_path), false) => {
                    let fstab_path = bootable_path.join(args.fstab_rel);
                    toggle_rdonly_flag(&bootable_path, false)?;
                    burn_fstab(&fstab_path, &fstab_content)?;
                    toggle_rdonly_flag(&bootable_path, true)?;
                }
                _ => {
                    println!("{}", fstab_content);
                    AppMessage::FstabEmitted {
                        length: fstab_content.len(),
                    }
                    .emit();
                }
            }
        }
        Commands::Rollback { prefix } => {
            if !getuid().is_root() {
                return Err(AppError::RootRequired);
            }
            AppMessage::UsingPrefix {
                prefix: prefix.clone(),
            }
            .emit();

            let recipe_path = Path::new(&args.recipe);
            let recipe = Recipe::load(recipe_path)?;

            let bootable = rollback(&recipe, &prefix)?;

            let fstab_content = brew_fstab(&recipe, None);

            match (bootable, args.emit_fstab) {
                (Some(bootable_path), false) => {
                    let fstab_path = bootable_path.join(args.fstab_rel);
                    burn_fstab(&fstab_path, &fstab_content)?;
                }
                _ => {
                    println!("{}", fstab_content);
                    AppMessage::FstabEmitted {
                        length: fstab_content.len(),
                    }
                    .emit();
                }
            }
        }
    }

    Ok(())
}
