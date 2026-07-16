
use std::time::{SystemTime, UNIX_EPOCH};
mod args;
mod outcome;
mod fstab;
mod recipe;
mod core;

use crate::core::{burn_fstab, set_readonly_flag};
use crate::outcome::{
    AppMessage,
    AppError,
    AppResult,
};
use crate::recipe::Recipe;
use crate::args::{
    AppArgs,
    Commands,
};

use clap::Parser;

use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        },
    }
}

fn run() -> AppResult<()> {
    let args = AppArgs::parse();

    match args.command {
        Commands::RecipeTemplate => {
            println!("{}", Recipe::TEMPLATE);
        },
        Commands::BurnFstab { source, target } => {
            let content = std::fs::read_to_string(&source)
                .map_err(|err| AppError::FstabReadError {
                    path: source.to_string_lossy().into_owned(),
                    what: err.to_string()
                })?;
            set_readonly_flag(&target, false)?;
            burn_fstab(&target, &content)?;
            set_readonly_flag(&target, true)?;
        },
        Commands::Run { prefix } => {
            let prefix_value = match prefix {
                Some(p) => p,
                None => {
                    // EXPECT: 1970-01-01 is always in the past.
                    let nanos = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("timestamp_fail")
                        .as_nanos();
                    format!("{:08x}", crc32fast::hash(&nanos.to_le_bytes()))  
                }
            };

            println!("{}", prefix_value);
            
        }
        
        _ => {}
    }    
    
    Ok(())
}
