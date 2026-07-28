use crate::core::recipe::Recipe;
use crate::outcome::{AppError, AppMessage, AppResult};
use std::ffi::CString;
use std::fs::File;
use std::os::fd::AsFd;
use std::path::PathBuf;

pub fn create_snapshots(recipe: &Recipe, prefix: &str) -> AppResult<Option<PathBuf>> {
    AppMessage::CreatingSnapshots.emit();
    
    let layout = match &recipe.btrfs_layout {
        Some(layout_value) => layout_value,
        None => return Err(AppError::BtrfsLayoutRequired),
    };
    
    let mut bootable_path: Option<PathBuf> = None;
    let snapshots_path = layout.bottom.join(&layout.snapshots);
    
    let snapshots_file = File::open(&snapshots_path).map_err(|e| AppError::SnapshotsDirOpenError {
        path: snapshots_path.clone(),
        what: e.to_string(),
    })?;
    
    let mut sources: Vec<(CString, File)> = Vec::new();
    
    for entry in &recipe.btrfs_entries {
        if !layout.tracked_set.contains(&entry.subvol) {
            continue;
        }
        
        let name = format!("{}.{}", prefix, entry.subvol);
        let target_path = snapshots_path.join(&name);
        let source_path = layout.bottom.join(&entry.subvol);
        
        if target_path.exists() {
            return Err(AppError::PrefixCollision {
                prefix: prefix.to_string(),
            });
        }
        
        if layout.bootable.as_ref() == Some(&entry.subvol) {
            bootable_path = Some(target_path);
        }
        
        let c_name = CString::new(name).map_err(|_| AppError::CreateCStringError)?;
        
        let file = File::open(&source_path).map_err(|e| AppError::OpenSubvolError {
            subvol: entry.subvol.clone(),
            what: e.to_string(),
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
        .map_err(|e| AppError::CreateSnapshotError {
            what: e.to_string(),
        })?;
    }
    
    AppMessage::SnapshotsCreated.emit();
    Ok(bootable_path)
}
