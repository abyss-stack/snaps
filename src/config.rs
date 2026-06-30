use crate::outcome::AppError::{ConfigReadError, JsonParseError};
use crate::outcome::AppResult;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct FstabEntry {
    pub device: String,
    pub mountpoint: String,
    pub fstype: String,
    pub options: Vec<String>,
    pub subvolume: Option<String>,
    pub dump: u8,
    pub pass: u8,
    pub is_state: bool,
}

pub type FstabConfig = Vec<FstabEntry>;

pub fn load_config(path: &str) -> AppResult<FstabConfig> {
    let data = std::fs::read_to_string(path).map_err(|_| ConfigReadError(path.to_string()))?;

    let config = serde_json::from_str(&data).map_err(|_| JsonParseError)?;

    Ok(config)
}
