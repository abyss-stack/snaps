use crate::outcome::AppError::{
    CStringConvertError, GeneralError, HashCollision, HashDirCreateFailed, KernelIoctlFailure,
    SnapsDirOpenFailed, SourceDirOpenFailed,
};
use crate::{config::FstabConfig, outcome::AppResult};
use std::ffi::CString;
use std::fs::File;
use std::os::fd::AsFd;
use std::path::Path;

pub fn create_snap(
    config: &FstabConfig,
    hash_str: &str,
    snaps_root: &str,
) -> AppResult<Vec<(String, String)>> {
    let parent_dir =
        File::open(snaps_root).map_err(|_| SnapsDirOpenFailed(String::from(snaps_root)))?;

    // mountpoint: "/", "/home", etc.
    // name: "@", "@home", etc.
    let targets: Vec<(String, String)> = config
        .iter()
        .filter(|mount| mount.is_state)
        .filter_map(|mount| {
            mount.subvolume.as_ref().map(|sv| {
                let name_str = sv.clone();
                (mount.mountpoint.clone(), name_str)
            })
        })
        .collect();

    if targets
        .iter()
        .any(|(_, name)| Path::new(snaps_root).join(name).exists())
    {
        return Err(HashCollision(String::from(hash_str)));
    }

    let mut source_dirs: Vec<(CString, File)> = Vec::with_capacity(targets.len());
    for (mountpoint, name) in &targets {
        let file = File::open(mountpoint).map_err(|_| SourceDirOpenFailed(name.clone()))?;
        let c_name = CString::new(name.clone()).map_err(|_| CStringConvertError(name.clone()))?;
        source_dirs.push((c_name, file));
    }

    let snap_dir_path = Path::new(snaps_root).join(hash_str);
    std::fs::create_dir(&snap_dir_path).map_err(|_| HashDirCreateFailed(hash_str.to_string()))?;

    for (c_name, file) in source_dirs {
        btrfs_uapi::subvolume::snapshot_create(
            parent_dir.as_fd(),
            file.as_fd(),
            &c_name,
            true, // INTENTIONAL: checking is_root here will turn the code to a mess.
            &[],
        )
        .map_err(|_| KernelIoctlFailure)?;
    }

    Ok(targets)
}

pub fn get_subvolume_flags(fd: std::os::fd::RawFd) -> AppResult<u64> {
    let mut flags: u64 = 0;
    // UNSAFE: low-level ioctl to get subvolume flags.
    // The kernel API is stable; this is the only way to access this functionality.
    unsafe {
        nix::ioctl_read!(btrfs_get_flags, btrfs_uapi::raw::BTRFS_IOCTL_MAGIC, 25, u64);
        btrfs_get_flags(fd, &mut flags)
            .map_err(|e| GeneralError(format!("Failed to get subvolume flags: {}", e)))?;
    }
    Ok(flags)
}
