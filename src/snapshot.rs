use crate::outcome::AppError::{
    CStringConvertError, HashCollision, HashDirCreateFailed, SnapsDirOpenFailed,
    SourceDirOpenFailed,
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

    let mut source_dirs: Vec<(String, File)> = Vec::with_capacity(targets.len());
    for (mountpoint, name) in &targets {
        let file = File::open(mountpoint).map_err(|_| SourceDirOpenFailed(name.clone()))?;
        let c_name = CString::new(name.clone()).map_err(|_| CStringConvertError(name.clone()))?;
        source_dirs.push((name.clone(), file));
    }

    let snap_dir_path = Path::new(snaps_root).join(hash_str);
    std::fs::create_dir(&snap_dir_path).map_err(|_| HashDirCreateFailed(hash_str.to_string()))?;

    Ok(targets)
}
