//! A nice helper to run everything.
#![allow(
    clippy::missing_errors_doc,
    reason = "this crate is all about running commands"
)]

/// Cli arguments parsing.
mod cli;
/// Runs the actions and commands.
mod runner;
/// Handles tmux commands.
mod tmux;

use std::path::Path;
use std::process::Command;

use clap::Parser as _;
use color_eyre::eyre::{Context as _, bail};

use crate::cli::Cli;

/// Result type for this binary.
type Result<T = ()> = color_eyre::Result<T>;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Cli::parse().into_runner()?.run()?;
    Ok(())
}

/// Runs a command without capturing the output.
fn cmd<P: AsRef<Path>>(prog: &str, args: &[&str], cwd: P) -> Result {
    let cmd = || format!("`{prog} {}`", args.join(" "));
    if !Command::new(prog)
        .args(args)
        .current_dir(cwd)
        .spawn()
        .with_context(|| format!("Failed to run {}", cmd()))?
        .wait()
        .with_context(|| format!("{} failed, check its logs above", cmd()))?
        .success()
    {
        bail!("{} failed, check its logs above", cmd())
    }
    Ok(())
}
