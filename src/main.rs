/*
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_hash() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:08x}", crc32fast::hash(&nanos.to_le_bytes()))
}

*/

mod args;
mod outcome;
mod fstab;
mod recipe;

use crate::outcome::{
    AppMessage,
    AppError,
    AppResult,
};
use std::process::ExitCode;

fn main() -> ExitCode {
    AppMessage::LoadingRecipe {path:"123".to_string()}.emit();
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
    
    
}

fn run() -> AppResult<()> {
    Ok(())
}
