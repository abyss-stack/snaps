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
