use crate::config::Config;
use crate::outcome::AppError::{FstabWriteError, InternalHashError};
use crate::outcome::AppResult;
use std::path::Path;

pub enum FstabAction {
    ToSnapshot,
    ToMain,
}
/*
Rewrites the fstab to create an immutable snapshot or restore the snapshot.
*/
pub fn modify_fstab(
    action: FstabAction,
    config: &Config,
    root_path: &Path,
    hash: Option<&str>,
) -> AppResult<()> {
    let mut content = String::from("# Generated automatically by abyss-snaps\n");
    for entry in &config.mounts {
        let mut options = entry.options.clone();

        match action {
            FstabAction::ToSnapshot => {
                if let Some(sv) = &entry.subvolume {
                    let hash_val = hash.ok_or_else(|| InternalHashError)?;
                    let payload = if entry.is_state {
                        format!("subvol={}/{}.{}", config.state.snaps_root, hash_val, sv)
                    } else {
                        format!("subvol={sv}")
                    };
                    options.push(payload);
                }
            }
            FstabAction::ToMain => {
                if let Some(sv) = &entry.subvolume {
                    options.push(format!("subvol={sv}"));
                }
            }
        };

        // INTENTIONAL: no checks for duplicates
        let options_string = options.join(",");
        let fstab_string = &format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            entry.device, entry.mountpoint, entry.fstype, options_string, entry.dump, entry.pass
        );
        content.push_str(fstab_string);
    }
    let fstab_path = root_path.join("etc/fstab");
    std::fs::write(&fstab_path, content)
        .map_err(|_| FstabWriteError(fstab_path.display().to_string()))?;
    Ok(())
}
