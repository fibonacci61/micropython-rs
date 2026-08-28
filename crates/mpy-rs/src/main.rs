use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

use mpy_rs::generate::generate;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(short = 'C')]
    chdir: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[cfg(feature = "install")]
    Install,
    Generate,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if let Some(chdir) = cli.chdir {
        std::env::set_current_dir(&chdir)
            .with_context(|| format!("couldn't change working directory to {}", chdir.display()))?
    }

    match cli.command {
        #[cfg(feature = "install")]
        Command::Install => mpy_rs::install::install(),
        Command::Generate => generate(),
    }
}
