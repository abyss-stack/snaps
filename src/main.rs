mod cli;
mod config;
mod fstab;
mod status;

use anyhow::Result;
use status::{AppError, AppMessage, AppResult};
use std::fs;

use crate::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse_args();

    match &cli.command {
        Some(Commands::Run) => {
            println!("Running snapshot process...");
            Ok(())
        }
        None => {
            greet_user()?;
            println!("Ha you don`t read this");

            Ok(())
        }
    }
}

fn greet_user() -> AppResult<()> {
    let config_dir = dirs::config_dir().ok_or(AppError::ConfigDirNotFound)?;

    let path = config_dir.join("abyss/snaps/greet.txt");
    let path_string = path.display().to_string();

    let content = fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            let msg = AppError::GreetFileNotFound(path_string.clone());
            println!("{}", serde_json::to_string(&msg).unwrap());
            msg
        } else {
            let msg = AppError::ReadGreetError(format!("{}: {}", path.display(), e));
            println!("{}", serde_json::to_string(&msg).unwrap());
            msg
        }
    })?;

    let msg = AppMessage::GreetShown(path_string);
    let line = serde_json::to_string(&msg).unwrap(); // UNWRAP: never fails.

    println!("{content}");
    println!("{line}");

    Ok(())
}
