use crate::core::recipe::Recipe;
use crate::outcome::{AppError, AppMessage, AppResult};
use std::ffi::CString;
use std::fs::File;
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::path::{Path, PathBuf};

pub fn rollback(recipe: &Recipe, prefix: &str) -> AppResult<Option<PathBuf>> {
    AppMessage::RollingBack {
        prefix: prefix.to_string(),
    }
    .emit();

    let layout = match &recipe.btrfs_layout {
        Some(layout_value) => layout_value,
        None => return Err(AppError::BtrfsLayoutRequired),
    };

    let mut bootable_path: Option<PathBuf> = None;

    let bottom_path = Path::new(&layout.bottom);
    let snapshots_path = bottom_path.join(&layout.snapshots);

    let bottom_file = File::open(&bottom_path).map_err(|_| AppError::BottomDirOpenError {
        path: snapshots_path.to_string_lossy().into_owned(),
    })?;

    let mut sources: Vec<(CString, String)> = Vec::new();

    for entry in &recipe.btrfs_entries {
        if !layout.tracked_set.contains(&entry.subvol) {
            continue;
        }

        let snapshot_name = format!("{}.{}", prefix, entry.subvol);
        let source_path = snapshots_path.join(&snapshot_name);
        let target_path = bottom_path.join(&entry.subvol);

        if !source_path.exists() {
            return Err(AppError::SnapshotNotFound {
                subvol: entry.subvol.clone(),
            });
        }

        if layout.bootable.as_ref() == Some(&entry.subvol) {
            bootable_path = Some(target_path.clone());
        }

        let c_name =
            CString::new(entry.subvol.clone()).map_err(|_| AppError::CreateCStringError)?;

        sources.push((c_name, entry.subvol.clone()));
    }

    // TODO: try going without .old

    for (c_name, subvol) in sources {
        let target = bottom_path.join(&subvol);
        let old = bottom_path.join(format!("{}.old", subvol));

        let snapshot_name = format!("{}.{}", prefix, subvol);
        let source_path = snapshots_path.join(&snapshot_name);

        let snapshot_file = File::open(&source_path).map_err(|_| AppError::OpenSubvolError {
            subvol: subvol.clone(),
        })?;

        if target.exists() {
            std::fs::rename(&target, &old).map_err(|_| AppError::RenameSubvolError)?;
        }

        btrfs_uapi::subvolume::snapshot_create(
            bottom_file.as_fd(),
            snapshot_file.as_fd(),
            &c_name,
            false,
            &[],
        )
        .map_err(|_| AppError::CreateSnapshotError)?;

        if old.exists() {
            let old_c_name = CString::new(format!("{}.old", subvol))
                .map_err(|_| AppError::CreateCStringError)?;

            btrfs_uapi::subvolume::subvolume_delete(bottom_file.as_fd(), &old_c_name)
                .map_err(|_| AppError::DeleteSnapshotError)?;
        }
    }

    AppMessage::RollbackDone.emit();

    Ok(bootable_path)
}
