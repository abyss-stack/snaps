use std::ffi::CString;
use std::fs::File;
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};

use crate::outcome::{AppError, AppMessage, AppResult};
use crate::core::recipe::Recipe;

// NOTE: Holds prepared rollback data.
struct RollbackSource {
    c_name: CString,
    subvolume: String,
    source_path: PathBuf,
    target_path: PathBuf,
}

pub fn run_rollback(recipe: &Recipe, prefix: &str) -> AppResult<()> {
    AppMessage::RollingBack {
        prefix: prefix.to_string(),
    }.emit();

    let layout = match &recipe.btrfs_layout {
        Some(layout_value) => layout_value,
        None => return Err(AppError::NoLayoutForRollback),
    };

    // NOTE: Root subvolume, often the 'subvolid=5' one.
    let bottom_path = Path::new(&layout.bottom);
    let snapshots_path = bottom_path.join(&layout.snapshots);

    let bottom_file = File::open(bottom_path).map_err(|e| AppError::BottomDirOpenError {
        path: bottom_path.to_path_buf(),
        what: e.to_string(),
    })?;
    
    // NOTE: Tradeoff: over-allocating memory to avoid reallocations.
    let mut sources: Vec<RollbackSource> = Vec::with_capacity(recipe.btrfs_entries.len());

    // NOTE: All-or-nothing rollback, checking source availability first.
    recipe.btrfs_entries.iter()
        .filter(|entry| layout.tracked_set.contains(&entry.subvol))
        .try_for_each(|entry| -> AppResult<()> {
            let snapshot_name = format!("{}.{}", prefix, entry.subvol);
            let target_path = bottom_path.join(&entry.subvol);
            let source_path = snapshots_path.join(&snapshot_name);
            
            if !source_path.exists() {
                return Err(AppError::SourceSubvolumeNotFound {
                    subvolume: entry.subvol.clone(),
                });
            }

            let c_name = CString::new(entry.subvol.clone())
                .map_err(|e| AppError::CreateCStringError {
                    what: e.to_string(),
                })?;
            
            sources.push(RollbackSource {
                c_name,
                subvolume: entry.subvol.clone(),
                source_path,
                target_path,
            });
            
            Ok(())
        })?;

    for source in sources {
        let temporary_name = format!("{}.tmp", source.subvolume);
        let temporary_path = bottom_path.join(&temporary_name);
    
        // EXAMPLE: rename '@home' to '@home.tmp'.
        // INTENTIONAL: The '.tmp' extension is appended to the right to avoid collisions with 'tmp' prefix.
        if source.target_path.exists() {
            std::fs::rename(&source.target_path, &temporary_path)
                .map_err(|e| AppError::RenameSubvolumeError {
                    subvolume: source.subvolume.clone(),
                    what: e.to_string(),
                })?;
        }

        let snapshot_file = File::open(&source.source_path).map_err(|e| AppError::OpenSubvolError {
            path: source.source_path.clone(),
            what: e.to_string(),
        })?;

        // EXAMPLE: creating read-write '@home' from 'prefix.@home'.
        btrfs_uapi::subvolume::snapshot_create(
            bottom_file.as_fd(),
            snapshot_file.as_fd(),
            &source.c_name,
            false,
        &[]
        ).map_err(|e| AppError::CreateSnapshotError {
            what: e.to_string(),
        })?;

        // NOTE: Cleaning up the temporary subvolume.
        if temporary_path.exists() {
            let temporary_c_name = CString::new(temporary_name.clone())
                .map_err(|e| AppError::CreateCStringError {
                    what: e.to_string(),
                })?;

            btrfs_uapi::subvolume::subvolume_delete(bottom_file.as_fd(), &temporary_c_name)
                .map_err(|e| AppError::DeleteSnapshotError {
                    subvolume: temporary_name,
                    what: e.to_string(),
                })?;
        }
        
    }

    AppMessage::RollbackDone.emit();

    Ok(())
}
