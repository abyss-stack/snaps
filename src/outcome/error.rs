use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "fail", rename_all = "snake_case")]
pub enum AppError {
    /* flags */
    GetFlagsError { path: PathBuf, what: String },
    SetFlagsError { path: PathBuf, what: String },
    SetRdonlyError { path: PathBuf, what: String },

    /* fstab */
    FstabWriteError { path: PathBuf, what: String },

    /* recipe */
    RecipeLoadError { path: PathBuf, what: String },
    RecipeParseError { what: String },

    /* deploy */
    BtrfsLayoutRequired,
    SnapshotsDirOpenError { path: PathBuf },
    PrefixCollision { prefix: String },
    CreateCStringError,
    OpenSubvolError { subvol: String },
    CreateSnapshotError,
    BottomDirOpenError { path: String },
    SnapshotNotFound { subvol: String },
    RenameSubvolError,
    DeleteSnapshotError,

    FstabReadError { path: PathBuf, what: String },

    /* main */
    RootRequired,
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
