//! A nice helper to run everything
#![deny(
    missing_docs,
    warnings,
    deprecated_safe,
    future_incompatible,
    keyword_idents,
    let_underscore,
    nonstandard_style,
    refining_impl_trait,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    rust_2024_compatibility,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::style,
    clippy::perf,
    clippy::complexity,
    clippy::correctness,
    clippy::restriction,
    clippy::nursery,
//     clippy::cargo
)]
#![allow(
    clippy::single_call_fn,
    clippy::implicit_return,
    clippy::pattern_type_mismatch,
    clippy::blanket_clippy_restriction_lints,
    clippy::missing_trait_methods,
    clippy::missing_inline_in_public_items,
    clippy::question_mark_used,
    clippy::mod_module_files,
    clippy::module_name_repetitions,
    clippy::pub_with_shorthand,
    clippy::unseparated_literal_suffix,
    clippy::else_if_without_else,
    clippy::doc_paragraphs_missing_punctuation,
    reason = "bad lints"
)]
#![allow(
    clippy::missing_errors_doc,
    reason = "this crate is all about running commands"
)]

/// Cli arguments parsing
mod cli;
/// Runs the actions and commands
mod runner;
/// Handles tmux commands
mod tmux;

use std::path::Path;
use std::process::Command;

use clap::Parser as _;
use color_eyre::eyre::{Context as _, bail};

use crate::cli::Cli;

/// Result type for this binary
type Result<T = ()> = color_eyre::Result<T>;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Cli::parse().into_runner()?.run()?;
    Ok(())
}

/// Runs a command without capturing the output
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
