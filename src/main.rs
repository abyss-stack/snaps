mod cli;
mod config;
mod fstab;
mod status;
use crate::cli::{Cli, Commands};
use json_color::Colorizer;
use status::{AppError, AppMessage, AppResult};
use std::fs;
use std::process::ExitCode;

pub struct AppContext {
    pub pretty: bool,
}

impl AppContext {
    pub fn new(cli: &Cli) -> Self {
        Self { pretty: cli.pretty }
    }

    pub fn emit_message(&self, msg: &AppMessage) {
        let output = if self.pretty {
            let pretty_json = serde_json::to_string_pretty(msg).unwrap(); // UNWRAP: won`t fail
            let colorizer = Colorizer::arbitrary();
            colorizer.colorize_json_str(&pretty_json).unwrap() // UNWRAP: won`t fail
        } else {
            serde_json::to_string(msg).unwrap() // UNWRAP: won`t fail
        };
        println!("{}", output);
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse_args();
    let ctx = AppContext::new(&cli);

    match run(&cli, &ctx) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", serde_json::to_string(&err).unwrap()); // UNWRAP: won`t fail
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli, ctx: &AppContext) -> AppResult<()> {
    match &cli.command {
        Some(Commands::Run) => {
            println!("Running snapshot process...");
            Ok(())
        }
        None => {
            greet_user(ctx)?;
            Ok(())
        }
    }
}

fn greet_user(ctx: &AppContext) -> AppResult<()> {
    let config_dir = dirs::config_dir().ok_or(AppError::ConfigDirNotFound)?;
    let path = config_dir.join("abyss/snaps/greet.txt");
    let path_string = path.display().to_string();

    let content = fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::GreetFileNotFound(path_string.clone())
        } else {
            AppError::ReadGreetError(format!("{}: {}", path_string, e))
        }
    })?;

    if ctx.pretty {
        println!("{content}");
    }

    ctx.emit_message(&AppMessage::GreetShown(path_string));

    Ok(())
}
