use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::outcome::{AppError, AppMessage, AppResult};

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
    pub const TEMPLATE: &str = include_str!("../../template.json");
    
    pub fn load<P>(path: P) -> AppResult<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        
        AppMessage::LoadingRecipe {
            path: path.to_path_buf(),
        }
        .emit();
        
        let data = std::fs::read_to_string(path).map_err(|e| AppError::RecipeLoadError {
            path: path.to_path_buf(),
            what: e.to_string(),
        })?;
        
        let mut recipe: Self = serde_json::from_str(&data).map_err(|e| AppError::RecipeParseError {
                what: e.to_string(),
            })?;
            
        if let Some(layout) = &mut recipe.btrfs_layout {
            layout.init_tracked_set();
        }
        
        AppMessage::RecipeLoaded.emit();
        Ok(recipe)
    }
}
