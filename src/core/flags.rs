use std::os::fd::{RawFd, AsRawFd};
use std::path::Path;
use std::fs::File;

use crate::outcome::{AppError, AppMessage, AppResult};

nix::ioctl_read!(get_flags_ioctl, btrfs_uapi::raw::BTRFS_IOCTL_MAGIC, 25, u64);
nix::ioctl_write_ptr!(set_flags_ioctl, btrfs_uapi::raw::BTRFS_IOCTL_MAGIC, 26, u64);

pub fn get_flags(fd: RawFd) -> AppResult<u64> {
    let mut flags: u64 = 0;
    // UNSAFE: btrfs ioctl.
    unsafe {
        get_flags_ioctl(fd, &mut flags)
            .map_err(|_| AppError::GetFlagsError)?;
    }
    Ok(flags)
}

pub fn set_flags(fd: RawFd, flags: u64) -> AppResult<u64> {
    // UNSAFE: btrfs ioctl.
    unsafe {
        set_flags_ioctl(fd, &flags)
            .map_err(|_| AppError::SetFlagsError)?;
    }
    Ok(flags)
}

pub fn toggle_rdonly_flag(path: &Path, value: bool) -> AppResult<()> {
    let file = File::open(path)
        .map_err(|_| AppError::SetRdonlyError {
             path: path.to_string_lossy().into_owned()
    })?;
    let mut flags = get_flags(file.as_raw_fd())?;
    match value {
        true => flags |= btrfs_uapi::raw::BTRFS_SUBVOL_RDONLY as u64,
        false => flags &= !(btrfs_uapi::raw::BTRFS_SUBVOL_RDONLY as u64),
    }
    let _ = set_flags(file.as_raw_fd(), flags)?;
    AppMessage::RdonlyToggled {
        path: path.to_string_lossy().into_owned(),
        value
    }.emit();
    Ok(())
}
