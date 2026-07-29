use std::fmt::Write;
use std::path::Path;

use crate::outcome::{AppError, AppMessage, AppResult};
use crate::core::recipe::Recipe;

pub fn burn_fstab<P: AsRef<Path>>(path: P, content: &str) -> AppResult<()> {
    let path = path.as_ref();

    std::fs::write(path, content).map_err(|e| AppError::BurnFstabError {
        path: path.to_path_buf(),
        what: e.to_string(),
    })?;

    AppMessage::FstabBurned {
        path: path.to_path_buf(),
    }.emit();

    Ok(())
}

pub fn brew_fstab(recipe: &Recipe, prefix: Option<&str>) -> String {
    AppMessage::BrewingFstab.emit();

    // NOTE: Preallocate memory to avoid unnecessary reallocations.
    let capacity = (recipe.btrfs_entries.len() + recipe.non_btrfs_entries.len() + 1) * 128 + 256;
    let mut buffer = String::with_capacity(capacity);

    recipe.non_btrfs_entries.iter().for_each(|entry| {
        let _ = write!(&mut buffer, "{}\t{}\t{}\t", entry.device, entry.mountpoint, entry.fstype);

        // NOTE: Always append a trailing comma, then remove it at the end.
        entry.options.iter().for_each(|opt| {
            let _ = write!(&mut buffer, "{},", opt);
        });

        if !entry.options.is_empty() {
            buffer.pop();
        }

        let _ = writeln!(&mut buffer, "\t{}\t{}", entry.dump, entry.pass);
    });

    AppMessage::NonBtrfsBrewed {
        count: recipe.non_btrfs_entries.len(),
    }.emit();

    let layout = match &recipe.btrfs_layout {
        Some(layout_value) => layout_value,
        None => {
            AppMessage::FstabBrewed.emit();
            return buffer;
        }
    };
    
    recipe.btrfs_entries.iter().for_each(|entry| {
        let is_tracked = layout.tracked_set.contains(&entry.subvol);

        let _ = write!(&mut buffer, "{}\t{}\tbtrfs\t", layout.device, entry.mountpoint);

        layout.options.iter().for_each(|opt| {
            let _ = write!(&mut buffer, "{},", opt);
        });

        // NOTE: Route to the snapshot if the subvolume is tracked and a prefix is provided.
        match (prefix, is_tracked) {
            (Some(prefix_value), true) => {
                let _ = write!(&mut buffer, "subvol={}/{}.{}", layout.snapshots, prefix_value, entry.subvol);
            },
            _ => {
                let _ = write!(&mut buffer, "subvol={}", entry.subvol);
            }
        }

        let _ = writeln!(&mut buffer, "\t0\t0");
    });

    AppMessage::BtrfsBrewed {
        count: recipe.btrfs_entries.len()
    }.emit();

    // NOTE: The Btrfs root subvolume must always be mounted.
    let _ = write!(&mut buffer, "{}\t{}\tbtrfs\t", layout.device, layout.bottom.display());

    layout.bottom_options.iter().for_each(|opt| {
        let _ = write!(&mut buffer, "{},", opt);
    });

    if !layout.bottom_options.is_empty() {
        buffer.pop();
    }

    let _ = writeln!(&mut buffer, "\t0\t0");

    AppMessage::FstabBrewed.emit();

    buffer
}
