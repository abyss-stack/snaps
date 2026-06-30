use crate::outcome::AppError::{HashCollision, SnapsDirOpenFailed};
use crate::{config::FstabConfig, outcome::AppResult};
use std::fs::File;
use std::path::Path;

pub fn create_snap(
    config: &FstabConfig,
    hash_str: &str,
    snaps_root: &str,
) -> AppResult<Vec<(String, String)>> {
    let parent_dir =
        File::open(snaps_root).map_err(|_| SnapsDirOpenFailed(String::from(snaps_root)))?;

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

    Ok(targets)
}
