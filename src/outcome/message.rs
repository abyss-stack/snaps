use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppMessage {
    /* flags */
    RdonlyToggled {
        path: String,
        value: bool,
    },

    /* fstab */
    FstabBurned { path: String },
}

impl AppMessage {
    pub fn to_json(&self) -> String {
        // EXPECT: infallible serialization.
        // Standard types only, guaranteed to always be serializable.
        serde_json::to_string(self).expect("serialize_fail")
    }
    
    pub fn emit(&self) {
        eprintln!("{}", self.to_json());
    }
}
