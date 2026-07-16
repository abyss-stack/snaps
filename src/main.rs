
use std::time::{SystemTime, UNIX_EPOCH};
mod args;
mod outcome;
mod fstab;
mod recipe;
mod core;

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
        }
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
