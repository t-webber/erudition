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

use core::time::Duration;
use std::env::current_dir;
use std::path::PathBuf;
use std::process::Command;
use std::thread::sleep;

use clap::Parser;
use color_eyre::eyre::{Context as _, eyre};

/// Result type for this file.
type Result<T = ()> = color_eyre::Result<T>;

/// A runner that handles all different aspects (server, app, logs) with one
/// command.
#[derive(Parser)]
struct Dev {
    /// Kill the running tmux session
    #[arg(short, long, default_value_t = false)]
    kill: bool,
    /// Change the delay between launching the app and opening the logs
    #[arg(short, long, default_value_t = 2000)]
    logs_delay: u64,
    /// Open the running tmux session
    #[arg(short, long, default_value_t = false)]
    open: bool,
    /// Name of the tmux session
    #[arg(short, long, default_value = "erudition")]
    session: String,
}

impl Dev {
    /// Attaches the tmux session to the current terminal
    fn attach(&self) -> Result {
        Command::new("tmux")
            .args(["attach-session", "-t", &self.session])
            .spawn()?
            .wait()?;
        Ok(())
    }

    /// Returns the runner with more settings that the CLI
    fn into_runner(self) -> Result<Option<Runner>> {
        if self.kill {
            tmux(&["kill-session", "-t", &self.session]).map(|()| None)
        } else if self.open {
            self.attach()
                .with_context(|| {
                    format!(
                        "Failed to attach session {} to the current terminal",
                        self.session
                    )
                })
                .map(|()| None)
        } else {
            Ok(Some(Runner {
                logs_delay: self.logs_delay,
                pwd: current_dir().context("Failed to read PWD")?,
                session: self.session,
            }))
        }
    }
}

/// Runner for tmux
struct Runner {
    /// Change the delay between launching the app and opening the logs
    logs_delay: u64,
    /// Path to the current working directory
    pwd: PathBuf,
    /// Name of the tmux session
    session: String,
}

impl Runner {
    /// Runs the CLI
    fn run(self) -> Result {
        tmux(&["new-session", "-d", "-s", &self.session, "-n", "app"])?;
        self.send_keys("app", "builtin cd app && dx serve --android")?;

        tmux(&["new-window", "-t", &self.session, "-n", "server"])?;
        self.send_keys(
            "server",
            "builtin cd server && $(/bin/which cargo) run",
        )?;

        sleep(Duration::from_millis(self.logs_delay));

        tmux(&["new-window", "-t", &self.session, "-n", "log"])?;
        self.send_keys(
            "log",
            r"adb logcat | /bin/grep RustStdoutStderr | /bin/grep -v s_glBindAttribLocation | sed 's/\(.* .*\) [0-9]* [0-9]* I RustStdoutStderr:/\x1b[33m\1\x1b[0m/' | sed 's/\[\([^][]*\)\]/\x1b[37m[\1]\x1b[0m/g'",
        )?;

        tmux(&["attach-session", "-t", &self.session])?;

        Ok(())
    }

    /// Runs a tmux 'send-keys' command
    fn send_keys(&self, window: &str, keys: &str) -> Result {
        tmux(&[
            "send-keys",
            "-t",
            &format!("{}:{window}", self.session),
            &format!("builtin cd {} && {keys}", self.pwd.display()),
            "C-m",
        ])
    }
}

/// Runs a tmux command
fn tmux(args: &[&str]) -> Result {
    let cmd = || format!("Failed to run `tmux {}`", args.to_vec().join(" "));
    let out = Command::new("tmux").args(args).output().wrap_err_with(cmd)?;
    if out.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stderr.starts_with("duplicate session: ") {
        Err(eyre!("{stderr}")).wrap_err(cmd()).wrap_err(
            "A session is already running with that name.\nOpen it with \
             `--open`, kill it with `--kill` or use a different name with \
             `--session`",
        )
    } else {
        Err(eyre!("{stderr}").wrap_err(cmd()))
    }
}

fn main() -> Result {
    color_eyre::install()?;
    if let Some(runner) = Dev::parse().into_runner()? {
        runner.run()?;
    }
    Ok(())
}
