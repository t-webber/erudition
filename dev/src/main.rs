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
//      clippy::restriction,
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
use std::process::Command;
use std::thread::sleep;

use clap::Parser;

/// Main struct targeted for the CLI, containing options and actions.
#[derive(Parser)]
struct Dev {
    /// Additional option to kill the running tmux session
    #[arg(short, long, default_value_t = false)]
    kill: bool,
    /// Change the delay between launching the app and opening the logs
    #[arg(short, long, default_value_t = 2000)]
    logs_delay: u64,
    /// Name of the tmux session
    #[arg(short, long, default_value = "erudition")]
    session: String,
}

impl Dev {
    /// Runs the CLI
    fn run(self) {
        if self.kill {
            Self::tmux(&["kill-session", "-t", &self.session]);
            return;
        }

        Self::tmux(&["new-session", "-d", "-s", &self.session, "-n", "app"]);
        self.send_keys("app", "builtin cd app && dx serve --android");

        Self::tmux(&["new-window", "-t", &self.session, "-n", "server"]);
        self.send_keys(
            "server",
            "builtin cd server && $(/bin/which cargo) run",
        );

        sleep(Duration::from_millis(self.logs_delay));

        Self::tmux(&["new-window", "-t", &self.session, "-n", "log"]);
        self.send_keys(
            "log",
            r"adb logcat | /bin/grep RustStdoutStderr | /bin/grep -v s_glBindAttribLocation | sed 's/\(.* .*\) [0-9]* [0-9]* I RustStdoutStderr:/\x1b[33m\1\x1b[0m/' | sed 's/\[\([^][]*\)\]/\x1b[37m[\1]\x1b[0m/g'",
        );

        Self::tmux(&["attach-session", "-t", &self.session]);
    }

    /// Runs a tmux 'send-keys' command
    fn send_keys(&self, window: &str, keys: &str) {
        Self::tmux(&[
            "send-keys",
            "-t",
            &format!("{}:{window}", self.session),
            keys,
            "C-m",
        ]);
    }

    /// Runs a tmux command and returns the status
    fn tmux(args: &[&str]) {
        Command::new("tmux").args(args).status().unwrap();
    }
}

fn main() {
    Dev::parse().run();
}
