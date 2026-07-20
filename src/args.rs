use clap::{
    Parser,
    Subcommand
};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum Commands {
    RecipeTemplate,
    BurnFstab {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        target: PathBuf,
        #[arg(long, help = "Not handle rw/ro for target.")]
        not_handle_ro: bool,
    },
    Run {
        #[arg(long)]
        prefix: Option<String>,
    },
    Rollback {
        #[arg(long)]
        prefix: String,
    }
}

#[derive(Parser)]
#[command(version)]
pub struct AppArgs {
    #[arg(long, default_value = "/etc/abyss-snaps/recipe.json")]
    pub recipe: PathBuf,

    #[arg(long, help = "Emit fstab to stdout.")]
    pub emit_fstab: bool,

    #[command(subcommand)]
    pub command: Commands,
}
