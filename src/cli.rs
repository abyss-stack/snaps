use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    pub fn parse_args() -> Self {
        <Self as clap::Parser>::parse()
    }
}

#[derive(Subcommand)]
pub enum Commands {
    Run,
}
