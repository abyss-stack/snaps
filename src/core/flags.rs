use std::fs::File;
use std::os::fd::{AsRawFd, RawFd};
use std::path::Path;

use crate::outcome::{AppError, AppMessage, AppResult};

nix::ioctl_read!(get_flags_ioctl, btrfs_uapi::raw::BTRFS_IOCTL_MAGIC, 25, u64);
nix::ioctl_write_ptr!(set_flags_ioctl, btrfs_uapi::raw::BTRFS_IOCTL_MAGIC, 26, u64);

pub fn get_flags(fd: RawFd) -> AppResult<u64> {
    let mut flags: u64 = 0;
    unsafe { get_flags_ioctl(fd.as_raw_fd(), &mut flags) }.map_err(|_| AppError::GetFlagsError)?;
    Ok(flags)
}

pub fn set_flags(fd: RawFd, flags: u64) -> AppResult<()> {
    unsafe { set_flags_ioctl(fd.as_raw_fd(), &flags) }.map_err(|_| AppError::SetFlagsError)?;
    Ok(())
}

pub fn toggle_rdonly_flag(path: &Path, readonly: bool) -> AppResult<()> {
    let file = File::open(path).map_err(|_| AppError::SetRdonlyError {
        path: path.to_path_buf(),
    })?;
    let mut flags = get_flags(file.as_raw_fd())?;
    match readonly {
        true => flags |= btrfs_uapi::raw::BTRFS_SUBVOL_RDONLY as u64,
        false => flags &= !(btrfs_uapi::raw::BTRFS_SUBVOL_RDONLY as u64),
    }
    set_flags(file.as_raw_fd(), flags)?;
    AppMessage::RdonlyToggled {
        subvol: path.to_path_buf(),
        value: readonly,
    }
    .emit();
    Ok(())
}
