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

    FstabWriteError { what: String },
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // EXPECT: infallible serialization.
        let json = serde_json::to_string(self).expect("serialize_fail");
        write!(f, "{}", json)
    }
}

impl std::error::Error for AppError {}

