use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Run,
    Rollback {
        #[arg(short, long)]
        hash: String,
    },
}

#[derive(Parser)]
#[command(
    version,
    about = "A config-driven subvolumes orchestrator and state machine."
)]
pub struct Args {
    #[arg(long, help = "Machine-parsable format")]
    pub raw: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}
