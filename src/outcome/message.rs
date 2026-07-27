use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppMessage {
    RdonlyToggled { subvol: PathBuf, value: bool },

    FstabBurned { path: PathBuf },
    BrewingFstab,
    NonBtrfsBrewed { count: usize },
    BtrfsBrewed { count: usize },
    FstabBrewed,

    LoadingRecipe { path: PathBuf },
    RecipeLoaded,

    CreatingSnapshots,
    SnapshotsCreated,

    UsingPrefix { prefix: String },
    FstabEmitted { length: usize },

    RollingBack { prefix: String },
    RollbackDone,
}

impl AppMessage {
    #[allow(clippy::expect_used, reason = "Infallible serialization.")]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialize_fail")
    }

    pub fn emit(&self) {
        eprintln!("{}", self.to_json());
    }
}
