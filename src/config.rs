//use anyhow::Result;
use serde::Deserialize;
//use std::path::Path;

#[derive(Deserialize, Clone)]
pub struct FstabEntry {
    pub device: String,
    pub mountpoint: String,
    pub fstype: String,
    pub options: Vec<String>,
    pub subvol: Option<String>,
    pub dump: u8,
    pub pass: u8,
    pub is_dynamic: bool,
}

/*
pub fn load_config(path: AsRef<Path>) -> Result<Vec<FstabEntry>> {
    let config_data = std::fs::read_to_string(path.as_ref())
    .map_err(|_| );
}
*/
