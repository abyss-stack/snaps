use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum Commands {
    RecipeTemplate,
    BrewFstab {
        #[arg(long)]
        prefix: Option<String>,
    },
    BurnFstab {
        #[arg(long)]
        content: String,

        // Full path to a bootable subvolume.
        #[arg(long)]
        target: PathBuf,

        // To leave target subvolume read-only.
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
#[command(version = env!("VERSION"))]
pub struct AppArgs {
    #[arg(long, default_value = "/etc/abyss-snaps/recipe.json")]
    pub recipe: PathBuf,

    // Relative fstab path.
    #[arg(long, default_value = "etc/fstab")]
    pub fstab_path: String,

    #[arg(long, help = "Print fstab to stdout.")]
    pub fstab_stdout: bool,

    #[command(subcommand)]
    pub command: Commands,
}
