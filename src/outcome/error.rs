use serde::{
    Serialize,
    Deserialize,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "err", rename_all = "snake_case")]
pub enum AppError {
    /* Recipe */
    RecipeLoadError { what: String },
    RecipeParseError { what: String },

    /* Core */
    BtrfsGetFlagsError,
    BtrfsSetFlagsError,
    SetReadonlyError { path: String },

    FstabReadError {
        path: String,
        what: String,
    },
    FstabWriteError { what: String },
    
    BtrfsLayoutRequired,
    SnapshotsDirOpenError { path: String },
    PrefixCollision { prefix: String },
    CreateCStringError,
    OpenSubvolError { subvol: String },
    CreateSnapshotError,
    BottomDirOpenError { path: String },
    SnapshotNotFound { subvol: String },
    RenameSubvolError,
    DeleteSnapshotError,

    /* Main */
    RootRequired,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // EXPECT: infallible serialization.
        let json = serde_json::to_string(self).expect("serialize_fail");
        write!(f, "{}", json)
    }
}

impl std::error::Error for AppError {}

