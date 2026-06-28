mod cli;
mod config;
mod fstab;
mod status;
use status::{AppError, AppMessage};
use std::fs;

use crate::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse_args();

    match &cli.command {
        Some(Commands::Run) => {
            println!("Running snapshot process...");
        }
        None => {
            greet_user();
        }
    }
}

fn greet_user() {
    let path = dirs::config_dir()
        .map(|d| d.join("abyss/snaps/greet.txt"))
        .filter(|p| p.exists());

    if let Some(valid_path) = path {
        // 1. Read and print the file content directly to stdout
        let content =
            fs::read_to_string(&valid_path).unwrap_or_else(|_| String::from("error reading file"));
        println!("{}", content);

        // 2. Convert the PathBuf path into a String safely for your enum
        let path_string = valid_path.display().to_string();
        let msg = AppMessage::GreetShown(path_string);

        // 3. Directly serialize the enum and print the JSON line
        let line = serde_json::to_string(&msg).unwrap();
        println!("{line}");
    }
}
