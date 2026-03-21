use std::env::{current_dir, current_exe};

use clap::{ArgGroup, Parser};
use color_eyre::eyre::Context as _;

use crate::runner::Runner;

/// Macro to ease create of the [`Action`] enum
macro_rules! first_true {
    ($( $flag:ident => $variant:ident ,)*) => {
        pub enum Action {
            All,
            $($variant,)*
        }
        impl Action {
            const VALUES: &[&str] = &[$(stringify!($flag)),*];

            const fn from_cli(cli: &Cli) -> Self {
                $( if cli.$flag { return Self::$variant; } )*
                Self::All
            }
        }
    };
}

first_true! {
    app => App,
    kill => Kill,
    logs => Logs,
    open => Open,
    server => Server,
}

/// A runner that handles all different aspects (server, app, logs) with one
/// command.
#[derive(Parser, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "cli flags")]
#[command(group(
    ArgGroup::new("scope")
        .args(Action::VALUES)
        .multiple(false)
))]
#[command(group(
    ArgGroup::new("tmux")
        .args(["app", "logs", "server", "name"])
        .multiple(false)
))]
pub struct Cli {
    /// Run only the app
    #[arg(short, long, default_value_t = false)]
    app: bool,
    /// Kill the running tmux session
    #[arg(short, long, default_value_t = false)]
    kill: bool,
    /// Run only the logs
    #[arg(short, long, default_value_t = false)]
    logs: bool,
    /// Name of the tmux session
    #[arg(short, long, default_value = "erudition")]
    name: String,
    /// Open the running tmux session
    #[arg(short, long, default_value_t = false)]
    open: bool,
    /// Run only the server
    #[arg(short, long, default_value_t = false)]
    server: bool,
}

impl Cli {
    /// Returns the [`Runner`] with the current settings to execute what was
    /// intended by the user.
    pub fn into_runner(self) -> color_eyre::Result<Runner> {
        Ok(Runner {
            action: Action::from_cli(&self),
            session: self.name,
            this: current_exe()
                .context("failed to get current executable path")?,
            pwd: current_dir()
                .context("Failed to get current working directory")?,
        })
    }
}
