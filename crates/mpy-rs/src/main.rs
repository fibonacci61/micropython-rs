mod install;
mod manifest;

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

use crate::install::install;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(short = 'C')]
    dir: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Command {
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
        Command::Install => {
            install(dir)?;
        }
        Command::Generate => {}
    }
    Ok(())
}
