use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

use mpy_rs::generate::generate;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(short = 'C')]
    dir: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[cfg(feature = "install")]
    Install,
    Generate,
}

#[derive(Debug, Subcommand)]
enum Generate {
    Libs,
    Headers,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let dir = cli
        .dir
        .as_ref()
        .map(|d| d.canonicalize())
        .transpose()
        .with_context(|| {
            format!(
                "couldn't canonicalize directory `{}`",
                cli.dir.unwrap().display()
            )
        })?;

    match cli.command {
        #[cfg(feature = "install")]
        Command::Install => mpy_rs::install::install(dir),
        Command::Generate => generate(dir),
    }
}
