use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "fail", rename = "snake_case")]
pub enum AppError {
    /* flags */
    OpenSubvolError {
        path: PathBuf,
        what: String,
    },
    GetFlagsError {
        path: PathBuf,
        what: String,
    },
    SetFlagsError {
        path: PathBuf,
        what: String,
    },

    /* recipe */
    RecipeLoadError {
        path: PathBuf,
        what: String,
    },
    RecipeParseError {
        what: String,
    },

    /* fstab */
    BurnFstabError {
        path: PathBuf,
        what: String,
    },

    /* deploy */
    NoLayoutToDeploy,
    PrefixCollision {
        prefix: String,
    },
    // OpenSubvolError {...},
    CreateCStringError {
        what: String,
    },
    CreateSnapshotError {
        what:String,
    },
    
    SnapshotsDirOpenError {
        path: PathBuf,
        what: String,
    },

}

impl AppError {
    #[allow(clippy::expect_used)]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("to_json_fail")
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
