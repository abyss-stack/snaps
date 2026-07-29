use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppMessage {
    /* flags */
    RdonlyToggled {
        path: PathBuf,
        rdonly: bool,
    },

    /* recipe */
    LoadingRecipe {
        path: PathBuf,
    },
    TrackedSetInited {
        count: usize,
    },
    RecipeLoaded,

    /* fstab */
    FstabBurned {
        path: PathBuf,
    },
    BrewingFstab,
    NonBtrfsBrewed {
        count: usize,
    },
    BtrfsBrewed {
        count: usize,
    },
    FstabBrewed,

    /* deploy */
    DeployingSnapshots,
    DeploymentDone,

    /* rollback */
    RollingBack {
        prefix: String,
    },
    RollbackDone,

    /* main */
    UsingPrefix {
        prefix: String,
    },
    FstabEmitted {
        len: usize,
    },
    
}

impl AppMessage {
    #[allow(clippy::expect_used)]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("to_json_fail")
    }

    pub fn emit(&self) {
        eprintln!("{}", self.to_json());
    }
}

impl std::fmt::Display for AppMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_json())
    }
}
