use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::path::Path;
use std::fs::OpenOptions;

use crate::{AppError, AppMessage, AppResult};

const BTRFS_GETFLAGS_SEQ: u8 = 25;
const BTRFS_SETFLAGS_SEQ: u8 = 26;

nix::ioctl_read!(get_flags_ioctl, btrfs_uapi::raw::BTRFS_IOCTL_MAGIC, BTRFS_GETFLAGS_SEQ, u64);
nix::ioctl_write_ptr!(set_flags_ioctl, btrfs_uapi::raw::BTRFS_IOCTL_MAGIC, BTRFS_SETFLAGS_SEQ, u64);

fn get_flags(fd: BorrowedFd<'_>) -> Result<u64, nix::Error> {
    let mut flags: u64 = 0;
    unsafe { get_flags_ioctl(fd.as_raw_fd(), &mut flags) }?;
    Ok(flags)
}

fn set_flags(fd: BorrowedFd<'_>, flags: u64) -> Result<(), nix::Error> {
    unsafe { set_flags_ioctl(fd.as_raw_fd(), &flags) }?;
    Ok(())
}

pub fn toggle_rdonly_flag<P: AsRef<Path>>(path: P, rdonly: bool) -> AppResult<()> {
    const RDONLY_FLAG: u64 = btrfs_uapi::raw::BTRFS_SUBVOL_RDONLY as u64;

    let path = path.as_ref();

    // NOTE: Do not set '.write(true)' to avoid failures on read-only subvolumes.
    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| AppError::OpenSubvolError {
             path: path.to_path_buf(),
             what: e.to_string(),
         })?;

     let fd = file.as_fd();

     let mut flags = get_flags(fd).map_err(|e| AppError::GetFlagsError {
         path: path.to_path_buf(),
         what: e.to_string(),
     })?;

     if rdonly {
         flags |= RDONLY_FLAG;
     }
     else {
         flags &= !RDONLY_FLAG;
     }

     set_flags(fd, flags).map_err(|e| AppError::SetFlagsError {
         path: path.to_path_buf(),
         what: e.to_string(),
     })?;

     AppMessage::RdonlyToggled {
         path: path.to_path_buf(),
         rdonly
     }.emit();

    Ok(())
}
