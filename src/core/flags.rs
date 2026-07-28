use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::path::Path;
use std::fs::OpenOptions;

use crate::outcome::{AppError, AppMessage, AppResult};

nix::ioctl_read!(get_flags_ioctl, btrfs_uapi::raw::BTRFS_IOCTL_MAGIC, 25, u64);
nix::ioctl_write_ptr!(set_flags_ioctl, btrfs_uapi::raw::BTRFS_IOCTL_MAGIC, 26, u64);

pub fn get_flags(fd: BorrowedFd<'_>) -> Result<u64, nix::Error> {
    let mut flags: u64 = 0;
    unsafe { get_flags_ioctl(fd.as_raw_fd(), &mut flags) }?;
    Ok(flags)
}

pub fn set_flags(fd: BorrowedFd<'_>, flags: u64) -> Result<(), nix::Error> {
    unsafe { set_flags_ioctl(fd.as_raw_fd(), &flags) }?;
    Ok(())
}

pub fn toggle_rdonly_flag<P>(path: P, readonly: bool) -> AppResult<()>
where
    P: AsRef<Path>,
{
    const RDONLY_FLAG: u64 = btrfs_uapi::raw::BTRFS_SUBVOL_RDONLY as u64;

    let path = path.as_ref();
    
    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| AppError::SetRdonlyError {
            path: path.to_path_buf(),
            what: e.to_string(),
        })?;

    let fd = file.as_fd();
    
    let mut flags = get_flags(fd).map_err(|e| AppError::GetFlagsError {
        path: path.to_path_buf(),
        what: e.to_string(),
    })?;

    if readonly {
        flags |= RDONLY_FLAG;
    } else {
        flags &= !RDONLY_FLAG;
    }

    set_flags(fd, flags).map_err(|e| AppError::SetFlagsError {
        path: path.to_path_buf(),
        what: e.to_string(),
    })?;

    AppMessage::RdonlyToggled {
        subvol: path.to_path_buf(),
        value: readonly,
    }
    .emit();

    Ok(())
}
