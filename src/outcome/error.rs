use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "fail", rename_all = "snake_case")]
pub enum AppError {
    /* Deploy */
    BottomDirOpenError { path: PathBuf, what: String },
    BtrfsLayoutRequired,
    CreateCStringError,
    CreateSnapshotError { what: String },
    DeleteSnapshotError { subvol: String, what: String },
    OpenSubvolError { subvol: String, what: String },
    PrefixCollision { prefix: String },
    RenameSubvolError { subvol: String, what: String },
    SnapshotNotFound { subvol: String },
    SnapshotsDirOpenError { path: PathBuf, what: String },

    /* Flags */
    GetFlagsError { path: PathBuf, what: String },
    SetFlagsError { path: PathBuf, what: String },
    SetRdonlyError { path: PathBuf, what: String },

    /* Fstab */
    FstabReadError { path: PathBuf, what: String },
    FstabWriteError { path: PathBuf, what: String },

    /* Main */
    RootRequired,

    /* Recipe */
    RecipeLoadError { path: PathBuf, what: String },
    RecipeParseError { what: String },
}

impl AppError {
    #[allow(clippy::expect_used, reason = "Infallible serialization.")]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialize_fail")
    }

    pub fn emit(&self) {
        eprintln!("{}", self.to_json());
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_json())
    }
}

impl std::error::Error for AppError {}
