use std::path::PathBuf;
use std::process::Command;

use color_eyre::eyre::{Context as _, eyre};

use crate::{Result, cmd};

/// Tmux handler, to run commands on a tmux session
pub struct Tmux {
    /// Index of the pane to know where to run the command
    pane_index: usize,
    /// Root path in which to run the commands inside the panes
    root_path: PathBuf,
    /// Name of the tmux session
    session: String,
}
impl Tmux {
    /// Attaches the associated tmux session
    pub fn attach(self) -> Result {
        cmd("tmux", &["attach-session", "-t", &self.session], &self.root_path)
    }

    /// Creates the associated tmux session
    pub fn create(&self) -> Result {
        Self::tmux(&["new-session", "-d", "-s", &self.session, "-n", "main"])
    }

    /// Kills the associated tmux session
    pub fn kill(self) -> Result {
        Self::tmux(&["kill-session", "-t", &self.session])
    }

    /// Creates a new [`Tmux`] with the given name.
    pub const fn new(session: String, root_path: PathBuf) -> Self {
        Self { root_path, session, pane_index: 0 }
    }

    /// Runs a tmux 'send-keys' command with nice error handling
    pub fn run(&self, command: &str) -> Result {
        Self::tmux(&[
            "send-keys",
            "-t",
            &self.pane_index.to_string(),
            &format!("builtin cd {} && {command}", self.root_path.display()),
            "C-m",
        ])?;
        Ok(())
    }

    /// Creates a new pane with a horizontal split
    #[expect(clippy::arithmetic_side_effects, reason = "small")]
    pub fn split(&mut self) -> Result {
        Self::tmux(&["split-window", "-h", "-t", &self.session])?;
        self.pane_index += 1;
        Ok(())
    }

    /// Runs a command with nice error handling
    pub fn tmux(args: &[&str]) -> Result {
        let cmd = || format!("Failed to run `tmux {}`", args.join(" "));

        let out =
            Command::new("tmux").args(args).output().wrap_err_with(cmd)?;

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

    /// Creates a new pane with a vertical split
    #[expect(clippy::arithmetic_side_effects, reason = "small")]
    pub fn vsplit(&mut self) -> Result {
        Self::tmux(&["split-window", "-v", "-t", &self.session])?;
        self.pane_index += 1;
        Ok(())
    }
}
