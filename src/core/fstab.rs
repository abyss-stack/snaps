use std::path::Path;

use crate::outcome::{AppError, AppMessage, AppResult};

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
