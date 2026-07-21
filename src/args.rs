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
        #[arg(long)]
        set_rdonly: bool,
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

    #[arg(long, default_value = "etc/fstab")]
    pub fstab_rel: String,

    #[arg(long, help = "Emit fstab to stdout.")]
    pub emit_fstab: bool,

    #[command(subcommand)]
    pub command: Commands,
}
