#![allow(clippy::uninlined_format_args)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

mod cli;
mod core;
mod outcome;

use std::path::Path;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use nix::unistd::getuid;

use cli::{AppArgs, Commands};
use core::deploy::deploy_snapshots;
use core::flags::toggle_rdonly_flag;
use core::fstab::{brew_fstab, burn_fstab};
use core::recipe::Recipe;
use core::rollback::run_rollback;
use outcome::{AppError, AppMessage, AppResult};

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
        Commands::BrewFstab { prefix } => {
            let recipe = Recipe::load(Path::new(&args.recipe))?;

            let fstab_content = match prefix {
                Some(prefix_value) => brew_fstab(&recipe, Some(&prefix_value)),
                None => brew_fstab(&recipe, None),
            };

            emit_fstab(&fstab_content);
        }
        Commands::BurnFstab {
            source,    // Fstab content path.
            target,     // A bootable subvolume.
            set_rdonly, // To leave the subvolume read-only after burning.
        } => {
            ensure_root()?;

            let content = std::fs::read_to_string(&source)
                .map_err(|e| AppError::FstabSourceReadError {
                    what: e.to_string(),
                })?;
            
            let fstab_path = target.join(&args.fstab_path);

            // NOTE: Ensure target is read-write before burning fstab.
            toggle_rdonly_flag(&target, false)?;

            burn_fstab(&fstab_path, &content)?;

            if set_rdonly {
                toggle_rdonly_flag(&target, true)?;
            }
        }
        Commands::Deploy { prefix } => {
            ensure_root()?;

            let prefix = match prefix {
                Some(prefix_value) => prefix_value,
                None => {
                    let nanos = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos();

                    format!("{:08x}", crc32fast::hash(&nanos.to_le_bytes()))
                }
            };

            AppMessage::UsingPrefix {
                prefix: prefix.clone(),
            }
            .emit();

            let recipe = Recipe::load(Path::new(&args.recipe))?;

            // NOTE: Pass Some() for prefix as we are brewing a snapshot fstab.
            let fstab_content = brew_fstab(&recipe, Some(&prefix));

            // INTENTIONAL: For non-btrfs recipes just brew fstab and emit it.
            let layout = match &recipe.btrfs_layout {
                Some(layout_value) => layout_value,
                None => {
                    emit_fstab(&fstab_content);
                    return Ok(());
                }
            };

            deploy_snapshots(&recipe, &prefix)?;

            let bootable = match &layout.bootable {
                Some(bootable_value) => bootable_value,
                None => {
                    emit_fstab(&fstab_content);
                    return Ok(());
                }
            };

            let bootable_subvolume = format!("{}.{}", prefix, bootable);

            let bottom_path = &layout.bottom;
            let snapshots_path = bottom_path.join(&layout.snapshots);
            let bootable_path = snapshots_path.join(&bootable_subvolume);
            let fstab_path = bootable_path.join(&args.fstab_path);

            if args.fstab_stdout {
                emit_fstab(&fstab_content);
            } else {
                toggle_rdonly_flag(&bootable_path, false)?;
                burn_fstab(fstab_path, &fstab_content)?;
                toggle_rdonly_flag(&bootable_path, true)?;
            }
        }
        Commands::Rollback { prefix } => {
            ensure_root()?;

            AppMessage::UsingPrefix {
                prefix: prefix.clone(),
            }
            .emit();

            let recipe = Recipe::load(Path::new(&args.recipe))?;

            run_rollback(&recipe, &prefix)?;

            // NOTE: Pass None for prefix, because we are brewing fstab for a main system.
            let fstab_content = brew_fstab(&recipe, None);

            #[allow(clippy::expect_used, reason = "Btrfs layout is already verified.")]
            let layout = recipe.btrfs_layout.as_ref().expect("btrfs_layout_missing");

            let bootable = match &layout.bootable {
                Some(bootable_value) => bootable_value,
                None => {
                    emit_fstab(&fstab_content);
                    return Ok(());
                }
            };

            let bottom_path = &layout.bottom;
            let bootable_path = bottom_path.join(bootable);
            let fstab_path = bootable_path.join(&args.fstab_path);

            if args.fstab_stdout {
                emit_fstab(&fstab_content);
            } else {
                toggle_rdonly_flag(&bootable_path, false)?;
                burn_fstab(fstab_path, &fstab_content)?;
            }
        }
    }
    Ok(())
}

fn ensure_root() -> AppResult<()> {
    if getuid().is_root() {
        Ok(())
    } else {
        Err(AppError::RootRequired)
    }
}

fn emit_fstab(fstab_content: &str) {
    println!("{}", fstab_content);
    AppMessage::FstabEmitted {
        len: fstab_content.len(),
    }
    .emit();
}
