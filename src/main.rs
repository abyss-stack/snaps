mod args;
mod boot;
mod config;
mod constants;
mod context;
mod outcome;
mod snapshot;
use crate::config::load_config;
use crate::constants::{DEFAULT_CONFIG, GREET};
use crate::outcome::AppMessage::{
    FstabModified, GreetShown, HashGenerated, JsonConfigAlreadyExists, JsonConfigCreated,
    RollbackCompleted, SnapshotCreated,
};
use crate::{args::Commands, outcome::AppResult};
use args::Args;
use boot::FstabAction::*;
use boot::modify_fstab;
use clap::Parser;
use context::AppContext;
use outcome::AppError::{
    ConfigDirNotFound, InternalHashError, InternalPathError, IoError, RootEntryNotFound,
    RootSubvolumeMissing, SnapshotNotFound,
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

    let result = match arguments.command {
        Some(Commands::Init) => init_config(&ctx),
        Some(Commands::Run) => run_inner(&ctx),
        Some(Commands::Rollback { hash }) => rollback(&ctx, hash),
        None => greet_user(&ctx),
    };

    handle_result(result, &ctx)
}

fn handle_result<T>(result: AppResult<T>, ctx: &AppContext) -> ExitCode {
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            let _ = ctx
                .emit_error(&e)
                .expect("Command failed, could not emit error.");
            ExitCode::FAILURE
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

    let config_path = dirs::config_dir()
        .ok_or(ConfigDirNotFound)?
        .join("abyss-snaps")
        .join("config.json");

    let config_path_str = config_path.to_str().ok_or_else(|| InternalPathError)?;
    let cfg = load_config(config_path_str)?;

    let bootable_subvol = &cfg.state.bootable_subvolume;

    let root_entry = cfg
        .mounts
        .iter()
        .find(|mount| mount.subvolume.as_ref() == Some(bootable_subvol))
        .ok_or_else(|| RootEntryNotFound)?;
    let root_name = root_entry
        .subvolume
        .as_ref()
        .ok_or_else(|| RootSubvolumeMissing)?;
    let root_snapshot_path =
        Path::new(&cfg.state.snaps_root).join(format!("{}.{}", hash_string, root_name));

    let created_snapshots = create_snap(&cfg, &hash_string)?;

    for (mountpoint, name) in &created_snapshots {
        ctx.emit_message(&SnapshotCreated {
            hash: hash_string.clone(),
            description: format!("{} -> {}", mountpoint, name),
        })?;
    }

    set_readonly_flag(&root_snapshot_path, false)?;
    modify_fstab(ToSnapshot, &cfg, &root_snapshot_path, Some(&hash_string))?;
    set_readonly_flag(&root_snapshot_path, true)?;

    ctx.emit_message(&FstabModified(root_snapshot_path.display().to_string()))?;

    Ok(())
}

fn rollback(ctx: &AppContext, hash: String) -> AppResult<()> {
    let config_path = dirs::config_dir()
        .ok_or(ConfigDirNotFound)?
        .join("abyss-snaps")
        .join("config.json");

    let config_path_str = config_path.to_str().ok_or_else(|| InternalPathError)?;
    let cfg = load_config(config_path_str)?;

    let bootable_subvol = &cfg.state.bootable_subvolume;

    let snaps_root = &cfg.state.snaps_root;
    let root_snapshot_path = Path::new(snaps_root).join(format!("{}.{}", hash, bootable_subvol));

    if !root_snapshot_path.exists() {
        return Err(SnapshotNotFound(root_snapshot_path.display().to_string()));
    }

    set_readonly_flag(&root_snapshot_path, false)?;
    modify_fstab(ToMain, &cfg, &root_snapshot_path, Some(&hash))?;
    set_readonly_flag(&root_snapshot_path, true)?;
    ctx.emit_message(&RollbackCompleted { hash })?;

    Ok(())
}

fn init_config(ctx: &AppContext) -> AppResult<()> {
    let config: &str = DEFAULT_CONFIG.trim_start_matches('\n');
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
    let greet: &str = GREET.trim_start_matches('\n');
    if ctx.raw {
        ctx.emit_message(&GreetShown(format!("Length is: '{}'.", greet.len())))?;
    } else {
        println!("{}", greet);
    }
    Ok(())
}
