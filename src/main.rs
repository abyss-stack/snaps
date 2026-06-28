mod cli;
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
        println!(
            "{}",
            fs::read_to_string(valid_path).unwrap_or_else(|_| String::from("error reading file"))
        )
    }
}
