use std::ffi::CString;
use std::fs::File;
use std::os::fd::AsFd;

use crate::outcome::{AppError, AppMessage, AppResult};
use crate::core::recipe::Recipe;

pub fn deploy_snapshots(recipe: &Recipe, prefix: &str) -> AppResult<()> {
    AppMessage::DeployingSnapshots.emit();

    let layout = match &recipe.btrfs_layout {
        Some(layout_value) => layout_value,
        None => return Err(AppError::NoLayoutToDeploy),
    };

    let snapshots_path = layout.bottom.join(&layout.snapshots);
        
    // NOTE: Tradeoff: over-allocating memory to avoid reallocations.
    let mut sources: Vec<(CString, File)> = Vec::with_capacity(recipe.btrfs_entries.len());

    // NOTE: All-or-nothing snapshots creation, checking collisions first.
    recipe.btrfs_entries.iter()
        .filter(|entry| layout.tracked_set.contains(&entry.subvol))
        .try_for_each(|entry| -> AppResult<()> {
            let name = format!("{}.{}", prefix, entry.subvol);
            let target_path = snapshots_path.join(&name);

            if target_path.exists() {
                return Err(AppError::PrefixCollision {
                    prefix: prefix.to_string(),
                });
            }

            let source_path = layout.bottom.join(&entry.subvol);
            let source_file = File::open(&source_path)
                .map_err(|e| AppError::OpenSubvolError {
                    path: source_path,
                    what: e.to_string(),
                })?;

            let c_name = CString::new(name).map_err(|e| AppError::CreateCStringError {
                what: e.to_string(),
            })?;

            sources.push((c_name, source_file)); 
            
            Ok(())
        })?;

    let snapshots_file = File::open(&snapshots_path)
        .map_err(|e| AppError::SnapshotsDirOpenError {
            path: snapshots_path.clone(),
            what: e.to_string(),
    })?;

    for (c_name, file) in sources {
        btrfs_uapi::subvolume::snapshot_create(snapshots_file.as_fd(), file.as_fd(), &c_name, true, &[])
            .map_err(|e| AppError::CreateSnapshotError {
                what: e.to_string(),
            })?;
    }

    AppMessage::DeploymentDone.emit();

    Ok(())
} 
