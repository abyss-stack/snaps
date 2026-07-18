
use std::time::{SystemTime, UNIX_EPOCH};
mod args;
mod outcome;
mod fstab;
mod recipe;
mod core;

use crate::core::{
    burn_fstab,
    set_readonly_flag,
    create_snapshots,
    rollback,
};
use crate::fstab::brew_fstab;
use crate::outcome::{
    AppMessage,
    AppError,
    AppResult,
};
use crate::recipe::Recipe;
use crate::args::{
    AppArgs,
    Commands,
};

use clap::Parser;
use nix::unistd::getuid;
use std::process::ExitCode;
use std::path::Path;

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        },
    }
}

fn run() -> AppResult<()> {
    let args = AppArgs::parse();

    match args.command {
        Commands::RecipeTemplate => {
            println!("{}", Recipe::TEMPLATE);
        },
        Commands::BurnFstab { source, target } => {
            if !getuid().is_root() {
                return Err(AppError::RootRequired);
            }
            let content = std::fs::read_to_string(&source)
                .map_err(|err| AppError::FstabReadError {
                    path: source.to_string_lossy().into_owned(),
                    what: err.to_string()
                })?;
            let fstab_path = target.join("etc/fstab");
            set_readonly_flag(&target, false)?;
            burn_fstab(&fstab_path, &content)?;
            set_readonly_flag(&target, true)?;
        },
        Commands::Run { prefix } => {
            if !getuid().is_root() {
                return Err(AppError::RootRequired);
            }
            let prefix_value = match prefix {
                Some(p) => p,
                None => {
                    // EXPECT: 1970-01-01 is always in the past.
                    let nanos = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("timestamp_fail")
                        .as_nanos();
                    format!("{:08x}", crc32fast::hash(&nanos.to_le_bytes()))  
                }
            };

            AppMessage::UsingPrefix {
                prefix: prefix_value.clone()
            }.emit();

            let recipe_path = Path::new(&args.recipe);
            let recipe = Recipe::load(recipe_path)?;

            let fstab_content = brew_fstab(&recipe, Some(&prefix_value));

            if recipe.btrfs_layout.is_none() {
                println!("{}", fstab_content);
                AppMessage::FstabEmitted {
                    length: fstab_content.len(),
                }.emit();

                return Ok(())
            }

            let bootable = create_snapshots(&recipe, &prefix_value)?;

            match (bootable, args.emit_fstab) {
                (Some(bootable_path), false) => {
                    let fstab_path = bootable_path.join("etc/fstab");
                    set_readonly_flag(&bootable_path, false)?;
                    burn_fstab(&fstab_path, &fstab_content)?;
                    set_readonly_flag(&bootable_path, true)?;
                },
                _ => {
                    println!("{}", fstab_content);
                    AppMessage::FstabEmitted {
                        length: fstab_content.len(),
                    }.emit();
                }
            }
        },
        Commands::Rollback { prefix } => {
            if !getuid().is_root() {
                return Err(AppError::RootRequired);
            }
            AppMessage::UsingPrefix {
                prefix: prefix.clone()
            }.emit();
            
            let recipe_path = Path::new(&args.recipe);
            let recipe = Recipe::load(recipe_path)?;

            let bootable = rollback(&recipe, &prefix)?;
            
            let fstab_content = brew_fstab(&recipe, None);

            match (bootable, args.emit_fstab) {
                (Some(bootable_path), false) => {
                    let fstab_path = bootable_path.join("etc/fstab");
                    burn_fstab(&fstab_path, &fstab_content)?;
                },
                _ => {
                    println!("{}", fstab_content);
                    AppMessage::FstabEmitted {
                        length: fstab_content.len(),
                    }.emit();
                }
            }
        }
    }    
    
    Ok(())
}
