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
    }
}

impl AppMessage {
    pub fn emit(&self) {
        // EXPECT: infallible serialization.
        let json = serde_json::to_string(self).expect("serialize_fail");
        eprintln!("{}", json);
    }
}
