use std::fmt::Write;
use std::path::Path;

use crate::core::recipe::Recipe;
use crate::outcome::{AppError, AppMessage, AppResult};

pub fn burn_fstab<P>(path: P, content: &str) -> AppResult<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    std::fs::write(path, content).map_err(|e| AppError::FstabWriteError {
        path: path.to_path_buf(),
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

    let estimated_lines = recipe.nonbtrfs_entries.len() + recipe.btrfs_entries.len() + 1;
    let capacity = (estimated_lines * 128) + 256;

    let mut buffer = String::with_capacity(capacity);

    for entry in &recipe.nonbtrfs_entries {
        let _ = write!(
            &mut buffer,
            "{}\t{}\t{}\t",
            entry.device, entry.mountpoint, entry.fs
        );

        let mut opt_iter = entry.options.iter().peekable();
        while let Some(opt) = opt_iter.next() {
            let _ = write!(&mut buffer, "{}", opt);
            if opt_iter.peek().is_some() {
                let _ = write!(&mut buffer, ",");
            }
        }

        let _ = writeln!(&mut buffer, "\t{}\t{}", entry.dump, entry.pass);
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
        let is_tracked = layout.tracked_set.contains(&entry.subvol);

        let _ = write!(
            &mut buffer,
            "{}\t{}\tbtrfs\t",
            layout.device, entry.mountpoint
        );

        let mut options_iter = layout.options.iter().peekable();
        while let Some(opt) = options_iter.next() {
            let _ = write!(&mut buffer, "{},", opt);
        }

        match (is_tracked, prefix) {
            (true, Some(prefix_val)) => {
                let _ = write!(
                    &mut buffer,
                    "subvol={}/{}.{}",
                    layout.snapshots, prefix_val, entry.subvol
                );
            }
            _ => {
                let _ = write!(&mut buffer, "subvol={}", entry.subvol);
            }
        }

        let _ = writeln!(&mut buffer, "\t0\t0");
    }
    AppMessage::BtrfsBrewed {
        count: recipe.btrfs_entries.len(),
    }
    .emit();

    let _ = write!(&mut buffer, "{}\t{}\tbtrfs\t", layout.device, layout.bottom.display());

    let mut bottom_opt_iter = layout.bottom_options.iter().peekable();
    while let Some(opt) = bottom_opt_iter.next() {
        let _ = write!(&mut buffer, "{}", opt);
        if bottom_opt_iter.peek().is_some() {
            let _ = write!(&mut buffer, ",");
        }
    }

    let _ = writeln!(&mut buffer, "\t0\t0");

    AppMessage::FstabBrewed.emit();
    buffer
}
