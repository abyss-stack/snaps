use crate::core::recipe::Recipe;
use crate::outcome::{AppError, AppMessage, AppResult};
use std::ffi::CString;
use std::fs::File;
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::path::{Path, PathBuf};

pub fn create_snapshots(recipe: &Recipe, prefix: &str) -> AppResult<Option<PathBuf>> {
    AppMessage::CreatingSnapshots.emit();
    let layout = match &recipe.btrfs_layout {
        Some(layout_value) => layout_value,
        None => return Err(AppError::BtrfsLayoutRequired),
    };
    let mut bootable_path: Option<PathBuf> = None;
    let bottom_path = Path::new(&layout.bottom);
    let snapshots_path = bottom_path.join(&layout.snapshots);
    let snapshots_file =
        File::open(&snapshots_path).map_err(|_| AppError::SnapshotsDirOpenError {
            path: snapshots_path.to_path_buf(),
        })?;
    let mut sources: Vec<(CString, File)> = Vec::new();
    for entry in &recipe.btrfs_entries {
        if !layout.tracked_set.contains(&entry.subvol) {
            continue;
        }
        let name = format!("{}.{}", prefix, entry.subvol);
        let target_path = snapshots_path.join(&name);
        let source_path = bottom_path.join(&entry.subvol);
        if target_path.exists() {
            return Err(AppError::PrefixCollision {
                prefix: prefix.to_string(),
            });
        }
        if layout.bootable.as_ref() == Some(&entry.subvol) {
            bootable_path = Some(target_path.clone());
        }
        let c_name = CString::new(name).map_err(|_| AppError::CreateCStringError)?;
        let file = File::open(&source_path).map_err(|_| AppError::OpenSubvolError {
            subvol: entry.subvol.clone(),
        })?;
        sources.push((c_name, file));
    }
    for (c_name, file) in sources {
        btrfs_uapi::subvolume::snapshot_create(
            snapshots_file.as_fd(),
            file.as_fd(),
            &c_name,
            true,
            &[],
        )
        .map_err(|_| AppError::CreateSnapshotError)?;
    }
    AppMessage::SnapshotsCreated.emit();
    Ok(bootable_path)
}
