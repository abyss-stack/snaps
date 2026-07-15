use crate::outcome::{
    AppMessage,
    AppError,
    AppResult,
};
use serde::{
    Serialize,
    Deserialize,
};
use std::path::{
    Path,
    PathBuf
};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct BtrfsLayout {
    pub device: String,
    pub bottom: PathBuf,
    pub snapshots: String,
    pub bootable: Option<String>,
    pub options: Vec<String>,
    pub bottom_options: Vec<String>,
    pub tracked: Vec<String>,
    #[serde(skip)]
    pub tracked_set: HashSet<String>,
}

impl BtrfsLayout {
    fn init_tracked_set(&mut self) {
        self.tracked_set = self.tracked.iter().cloned().collect();
    }
}

#[derive(Serialize, Deserialize)]
pub struct BtrfsEntry {
    pub mountpoint: String,
    pub subvol: String,
}

#[derive(Serialize, Deserialize)]
pub struct NonBtrfsEntry {
    pub device: String,
    pub mountpoint: String,
    pub fs: String,
    pub options: Vec<String>,
    pub dump: u8,
    pub pass: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Recipe {
    pub btrfs_layout: Option<BtrfsLayout>,
    pub btrfs_entries: Vec<BtrfsEntry>,
    pub nonbtrfs_entries: Vec<NonBtrfsEntry>,
}

impl Recipe {
    pub const TEMPLATE: &str = include_str!("../template.json");

    pub fn load (path: &Path) -> AppResult<Self> {
        AppMessage::LoadingRecipe {
            path: path.to_string_lossy().into_owned()
        }.emit();
    
        let data = std::fs::read_to_string(path)
            .map_err(|err| AppError::RecipeLoadError {
                what: err.to_string(),
            })?;

        let mut recipe: Self = serde_json::from_str(&data)
            .map_err(|err| AppError::RecipeParseError {
                what: err.to_string(),
            })?;

        if let Some(layout) = &mut recipe.btrfs_layout {
            layout.init_tracked_set();
        }

        AppMessage::RecipeLoaded.emit();

        Ok(recipe)
    }
}
