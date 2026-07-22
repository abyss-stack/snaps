use serde::{
    Serialize,
    Deserialize,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "msg", rename_all = "snake_case")]
pub enum AppMessage {
    /* Recipe */
    LoadingRecipe { path: String },
    RecipeLoaded,

    /* Fstab */
    BrewingFstab,
    NonBtrfsBrewed { count: usize },
    BtrfsBrewed { count: usize },
    FstabBrewed,

    /* Core */
    ReadOnlyToggled {
        path: String,
        value: bool,
    },
    FstabBurned { path: String },
    CreatingSnapshots,
    SnapshotsCreated,

    RollingBack { prefix: String },
    RollbackDone,

    /* Main */
    UsingPrefix { prefix: String },
    FstabEmitted { length: usize },
}

impl AppMessage {
    pub fn to_json(&self) -> String {
        // EXPECT: infallible serialization.
        serde_json::to_string(self).expect("serialize_fail")
    }
    pub fn emit(&self) {
        eprintln!("{}", self.to_json());
    }
}
