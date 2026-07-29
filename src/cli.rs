use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum Commands {
    RecipeTemplate,
    BurnFstab {
        #[arg(long)]
        content: String,
        
        #[arg(long)]
        set_rdonly: bool,
    },
    Deploy {
        #[arg(long)]
        prefix: Option<String>,
    },
    Rollback {
        #[arg(long)]
        prefix: String,
    },
}

#[derive(Parser)]
#[command(version = env!("PROJECT_VERSION"))]
pub struct AppArgs {
    #[arg(long, default_value = "/etc/abyss-snaps/recipe.json")]
    pub recipe: PathBuf,

    #[arg(long, help = "Print fstab to stdout.")]
    pub fstab_stdout: bool,

    #[command(subcommand)]
    pub command: Commands,
}
