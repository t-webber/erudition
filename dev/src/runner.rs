use core::time::Duration;
use std::io::{BufRead as _, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread::sleep;

use color_eyre::eyre::{Context as _, ContextCompat as _, bail, eyre};

use crate::cli::Action;

/// Result type for this file
type Result<T = ()> = color_eyre::Result<T>;

/// Runner for tmux
pub struct Runner {
    /// Action to be run
    pub action: Action,
    /// Change the delay between launching the app and opening the logs
    pub logs_delay: u64,
    /// Path to the current working directory
    pub pwd: PathBuf,
    /// Name of the tmux session
    pub session: String,
    /// Full path to the current binary being run
    pub this: PathBuf,
}

impl Runner {
    /// Runs a command without capturing the output
    fn cmd(&self, prog: &str, args: &[&str]) -> Result {
        let cmd = || format!("`{prog} {}`", args.join(" "));
        if !Command::new(prog)
            .args(args)
            .current_dir(&self.pwd)
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

    /// Runner entry point to execute what was intended by the user
    pub fn run(self) -> Result {
        match self.action {
            Action::All => self.run_all(),
            Action::App =>
                self.cmd("dx", &["serve", "-p", "erudition-app", "--android"]),
            Action::Kill => self.tmux(&["kill-session", "-t", &self.session]),
            Action::Logs => self.run_logs(),
            Action::Open =>
                self.cmd("tmux", &["attach-session", "-t", &self.session]),
            Action::Server =>
                self.cmd("cargo", &["run", "-p", "erudition-server"]),
        }
    }

    /// Runs the CLI
    fn run_all(self) -> Result {
        self.tmux(&["new-session", "-d", "-s", &self.session, "-n", "app"])?;
        self.send_keys("app")?;

        self.tmux(&["new-window", "-t", &self.session, "-n", "server"])?;
        self.send_keys("server")?;

        sleep(Duration::from_millis(self.logs_delay));

        self.tmux(&["new-window", "-t", &self.session, "-n", "logs"])?;
        self.send_keys("logs")?;

        self.cmd("tmux", &["attach-session", "-t", &self.session])
    }

    /// Listens to the logs and prettify them
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::string_slice,
        clippy::print_stdout,
        reason = "i don't care"
    )]
    fn run_logs(&self) -> Result {
        let mut child = Command::new("adb")
            .arg("logcat")
            .current_dir(&self.pwd)
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to run `adb logcat`")?;
        let reader = BufReader::new(
            child.stdout.take().context("Failed to reach stdout of adb")?,
        );
        for next in reader.lines() {
            let line = next.context("Failed to read output of adb")?;

            if line.contains("RustStdoutStderr")
                && !line.contains("s_glBindAttribLocation")
                && let Some(datetime_end) =
                    line.find(' ').and_then(|date_end| {
                        line[date_end + 1..]
                            .find(' ')
                            .map(|relative| relative + date_end + 1)
                    })
                && let Some(begin) = line[datetime_end + 1..]
                    .find("I RustStdoutStderr: ")
                    .map(|begin| {
                        datetime_end + 1 + begin + "I RustStdoutStderr: ".len()
                    })
            {
                print!("\x1b[37m{}\x1b[0m ", &line[0..datetime_end]);
                if line[begin..].chars().next().is_some_and(|ch| ch == '[')
                    && let Some(end) = line[begin..].find(']')
                {
                    let (scope, text) = line[begin..].split_at(end + 1);
                    println!("\x1b[33m{scope}\x1b[0m{text}");
                } else {
                    println!("{}", &line[begin..]);
                }
            }
        }
        Ok(())
    }

    /// Runs a tmux 'send-keys' command with nice error handling
    fn send_keys(&self, window: &str) -> Result {
        self.tmux(&[
            "send-keys",
            "-t",
            &format!("{}:{window}", self.session),
            &format!(
                "builtin cd {} && {} --{window}",
                self.pwd.display(),
                self.this.display()
            ),
            "C-m",
        ])
    }

    /// Runs a command with nice error handling
    fn tmux(&self, args: &[&str]) -> Result {
        let cmd = || format!("Failed to run `tmux {}`", args.join(" "));

        let out = Command::new("tmux")
            .args(args)
            .current_dir(&self.pwd)
            .output()
            .wrap_err_with(cmd)?;

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
}
