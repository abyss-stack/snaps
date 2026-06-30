mod args;
mod boot;
mod config;
mod context;
mod outcome;
mod snapshot;
use crate::config::load_config;
use crate::outcome::AppMessage::{
    FstabModified, GreetShown, HashGenerated, JsonConfigAlreadyExists, JsonConfigCreated,
    SnapshotCreated,
};
use crate::{args::Commands, outcome::AppResult};
use args::Args;
use boot::FstabAction::*;
use boot::modify_fstab;
use clap::Parser;
use context::AppContext;
use outcome::AppError::{
    ConfigDirNotFound, InternalHashError, InternalPathError, IoError, RootEntryNotFound,
    RootSubvolumeMissing,
};
use snapshot::{create_snap, set_readonly_flag};
use std::fs;
use std::path::Path;
use std::{
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() -> ExitCode {
    let arguments = Args::parse();
    let ctx = AppContext::from_args(arguments.raw);

    match arguments.command {
        Some(Commands::Init) => match init_config(&ctx) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                let _ = ctx
                    .emit_error(&e)
                    .expect("Init command failed, could not emit error.");
                ExitCode::FAILURE
            }
        },

        Some(Commands::Run) => match run_inner(&ctx) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                let _ = ctx
                    .emit_error(&e)
                    .expect("Run command failed, could not emit error.");
                ExitCode::FAILURE
            }
        },
        None => match greet_user(&ctx) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                let _ = ctx
                    .emit_error(&e)
                    .expect("Greet command failed, could not emit error.");
                ExitCode::FAILURE
            }
        },
    }
}

fn run_inner(ctx: &AppContext) -> AppResult<()> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| InternalHashError)?
        .as_nanos();
    let hash_string = format!("{:08x}", crc32fast::hash(&nanos.to_le_bytes()));

    ctx.emit_message(&HashGenerated(hash_string.clone()))?;

    let config_path = dirs::config_dir()
        .ok_or(ConfigDirNotFound)?
        .join("abyss-snaps")
        .join("config.json");

    let config_path_str = config_path.to_str().ok_or_else(|| InternalPathError)?;
    let fstab_config = load_config(config_path_str)?;

    let root_entry = fstab_config
        .iter()
        .find(|entry| entry.is_root)
        .ok_or_else(|| RootEntryNotFound)?;
    let root_name = root_entry
        .subvolume
        .as_ref()
        .ok_or_else(|| RootSubvolumeMissing)?;
    let root_snapshot_path = Path::new("/@abyss-snaps")
        .join(&hash_string)
        .join(root_name);

    let created_snapshots = create_snap(&fstab_config, &hash_string, "/@abyss-snaps")?;

    for (mountpoint, name) in &created_snapshots {
        ctx.emit_message(&SnapshotCreated {
            hash: hash_string.clone(),
            description: format!("{} -> {}", mountpoint, name),
        })?;
    }

    set_readonly_flag(&root_snapshot_path, false)?;
    modify_fstab(
        ToSnapshot,
        &fstab_config,
        &root_snapshot_path,
        Some(&hash_string),
    )?;
    set_readonly_flag(&root_snapshot_path, true)?;

    ctx.emit_message(&FstabModified(root_snapshot_path.display().to_string()))?;

    Ok(())
}

fn init_config(ctx: &AppContext) -> AppResult<()> {
    let config: &'static str = r#"
[
  {
    "device": "LABEL=ROOT_PART",
    "mountpoint": "/",
    "fstype": "btrfs",
    "options": ["strictatime"],
    "subvolume": "@",
    "dump": 0,
    "pass": 0,
    "is_state": true,
    "is_root": true,
  },
  {
    "device": "LABEL=ROOT_PART",
    "mountpoint": "/mnt/btrfs-root",
    "fstype": "btrfs",
    "options": ["strictatime", "subvolid=5"],
    "subvolume": null,
    "dump": 0,
    "pass": 0,
    "is_state": false,
    "is_root": false,
  },
  {
    "device": "LABEL=ROOT_PART",
    "mountpoint": "/.abyss-snaps",
    "fstype": "btrfs",
    "options": ["strictatime"],
    "subvolume": "@abyss-snaps",
    "dump": 0,
    "pass": 0,    
    "is_state": false,
    "is_root": false,
  },
  {
    "device": "LABEL=ROOT_PART",
    "mountpoint": "/home",
    "fstype": "btrfs",
    "options": ["strictatime"],
    "subvolume": "@home",
    "dump": 0,
    "pass": 0,    
    "is_state": true,
    "is_root": false,
  },
  {
    "device": "tmpfs",
    "mountpoint": "/tmp",
    "fstype": "tmpfs",
    "options": ["defaults", "nosuid", "nodev"],
    "subvolume": null,
    "dump": 0,
    "pass": 0,
    "is_state": false,
    "is_root": false,
  }
]
"#
    .trim_start_matches('\n');

    let config_dir = dirs::config_dir()
        .ok_or(ConfigDirNotFound)?
        .join("abyss-snaps");
    if config_dir.exists() {
        ctx.emit_message(&JsonConfigAlreadyExists(format!(
            "Json config already exists: '{}'.",
            config_dir.display(),
        )))?;
    } else {
        fs::create_dir_all(&config_dir).map_err(|e| IoError(e.to_string()))?;

        let config_path = config_dir.join("config.json");
        fs::write(&config_path, config).map_err(|e| {
            IoError(format!(
                "Failed to write config to '{}': '{}'.",
                config_path.display(),
                e
            ))
        })?;
        ctx.emit_message(&JsonConfigCreated(format!(
            "JSON config created at: '{}'.",
            config_dir.display()
        )))?;
    }

    Ok(())
}

fn greet_user(ctx: &AppContext) -> AppResult<()> {
    let greet: &'static str = r#"
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
    .trim_start_matches('\n');

    if ctx.raw {
        ctx.emit_message(&GreetShown(format!("Length is: '{}'.", greet.len())))?;
    } else {
        println!("{}", greet);
    }
    Ok(())
}
