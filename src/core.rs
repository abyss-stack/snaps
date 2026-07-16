use crate::outcome::{
    AppMessage,
    AppError,
    AppResult,
};
use crate::recipe::Recipe;

use std::ffi::CString;
use std::fs::File;
use std::os::fd::{
    AsFd,
    AsRawFd,
    RawFd
};
use std::path::{Path, PathBuf};

pub fn get_subvol_flags(fd: RawFd) -> AppResult<u64> {
    let mut flags: u64 = 0;

    nix::ioctl_read!(get_flags, btrfs_uapi::raw::BTRFS_IOCTL_MAGIC, 25, u64);

    // UNSAFE: btrfs ioctl.
    unsafe {
        get_flags(fd, &mut flags)
            .map_err(|_| AppError::BtrfsGetFlagsError)?;
    }

    Ok(flags)
}


pub fn set_subvol_flags(fd: RawFd, flags: u64) -> AppResult<u64> {
    nix::ioctl_write_ptr!(set_flags, btrfs_uapi::raw::BTRFS_IOCTL_MAGIC, 26, u64);

    // UNSAFE: btrfs ioctl.
    unsafe {
        set_flags(fd, &flags)
            .map_err(|_| AppError::BtrfsSetFlagsError)?;
    }

    Ok(flags)
}

pub fn set_readonly_flag(path: &Path, readonly: bool) -> AppResult<()> {
    let file = File::open(path)
        .map_err(|_| AppError::SetReadonlyError {
             path: path.to_string_lossy().into_owned()
    })?;

    let mut flags = get_subvol_flags(file.as_raw_fd())?;

    match readonly {
        true => flags |= btrfs_uapi::raw::BTRFS_SUBVOL_RDONLY as u64,
        false => flags &= !(btrfs_uapi::raw::BTRFS_SUBVOL_RDONLY as u64),
    }

    set_subvol_flags(file.as_raw_fd(), flags)?;
    
    AppMessage::ReadOnlyToggled {
        path: path.to_string_lossy().into_owned(),
        value: readonly
    }.emit();

    Ok(())
}

pub fn burn_fstab(path: &Path, content: &str) -> AppResult<()> {
    std::fs::write(path, content)
        .map_err(|err| AppError::FstabWriteError{
            what: err.to_string()
    })?;

    AppMessage::FstabBurned {
        path: path.to_string_lossy().into_owned()
    }.emit();

    Ok(())
}

pub fn create_snapshots(recipe: &Recipe, prefix: &str) -> AppResult<Option<PathBuf>> {
    AppMessage::CreatingSnapshots.emit();

    let layout = match &recipe.btrfs_layout {
        Some(layout_value) => layout_value,
        None => return Err(AppError::BtrfsLayoutRequired),
    };

    let mut bootable_path: Option<PathBuf> = None;

    let bottom_path = Path::new(&layout.bottom);
    let snapshots_path = bottom_path.join(&layout.snapshots);

    let snapshots_file = File::open(&snapshots_path)
        .map_err(|_| AppError::SnapshotsDirOpenError {
            path: snapshots_path.to_string_lossy().into_owned()
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

        let c_name = CString::new(name)
            .map_err(|_| AppError::CreateCStringError)?;
        let file = File::open(&source_path)
            .map_err(|_| AppError::OpenSubvolError {
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
        ).map_err(|_| AppError::CreateSnapshotError)?;
    }

    AppMessage::SnapshotsCreated.emit();

    Ok(bootable_path)
}
