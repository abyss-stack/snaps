use std::path::Path;

use crate::outcome::{AppError, AppMessage, AppResult};
use crate::core::recipe::Recipe;

pub fn burn_fstab(path: &Path, content: &str) -> AppResult<()> {
    std::fs::write(path, content)
        .map_err(|e| AppError::FstabWriteError {
        what: e.to_string(),
    })?;
    AppMessage::FstabBurned {
        path: path.to_path_buf(),
    }
    .emit();
    Ok(())
}

pub fn brew_fstab(recipe: &Recipe, prefix: Option<&str>) -> String {
    AppMessage::BrewingFstab.emit();
    let mut buffer = String::new();
    for entry in &recipe.nonbtrfs_entries {
        buffer.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            entry.device,
            entry.mountpoint,
            entry.fs,
            entry.options.join(","),
            entry.dump,
            entry.pass
        ));
    }
    AppMessage::NonBtrfsBrewed {
        count: recipe.nonbtrfs_entries.len(),
    }
    .emit();
    let layout = match &recipe.btrfs_layout {
        Some(layout_value) => layout_value,
        None => {
            AppMessage::FstabBrewed.emit();
            return buffer;
        }
    };
    for entry in &recipe.btrfs_entries {
        let mut options = layout.options.clone();
        match (layout.tracked_set.contains(&entry.subvol), prefix) {
            (true, Some(prefix_value)) => options.push(format!(
                "subvol={}/{}.{}",
                layout.snapshots, prefix_value, entry.subvol
            )),
            _ => options.push(format!("subvol={}", entry.subvol)),
        }
        buffer.push_str(&format!(
            "{}\t{}\tbtrfs\t{}\t0\t0\n",
            layout.device,
            entry.mountpoint,
            options.join(",")
        ));
    }
    AppMessage::BtrfsBrewed {
        count: recipe.btrfs_entries.len(),
    }
    .emit();
    buffer.push_str(&format!(
        "{}\t{}\tbtrfs\t{}\t0\t0\n",
        layout.device,
        layout.bottom.to_string_lossy().into_owned(),
        layout.bottom_options.join(",")
    ));
    AppMessage::FstabBrewed.emit();
    buffer
}
