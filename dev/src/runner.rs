use std::io::{BufRead as _, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use color_eyre::eyre::{Context as _, ContextCompat as _};

use crate::cli::{Action, Platform};
use crate::tmux::Tmux;
use crate::{Result, cmd};

/// Runner for tmux
pub struct Runner {
    /// Action to be run
    pub action: Action,
    /// Platform on which to run the app
    pub platform: Platform,
    /// Path to the current working directory
    pub root_path: PathBuf,
    /// Name of the tmux session
    pub session: String,
    /// Full path to the current binary being run
    pub this: PathBuf,
}

impl Runner {
    /// Runner entry point to execute what was intended by the user
    pub fn run(self) -> Result {
        match self.action {
            Action::All => self.run_all(),
            Action::App => cmd(
                "dx",
                &[
                    "serve",
                    "-p",
                    "erudition-app",
                    &format!("--{}", self.platform),
                ],
                &self.root_path,
            ),
            Action::Kill => Tmux::new(self.session, self.root_path).kill(),
            Action::Logs => self.run_logs(),
            Action::Open => cmd(
                "tmux",
                &["attach-session", "-t", &self.session],
                &self.root_path,
            ),
            Action::Server => cmd(
                "cargo",
                &["run", "-p", "erudition-server"],
                &self.root_path,
            ),
        }
    }

    /// Runs the CLI
    fn run_all(self) -> Result {
        let this = self.this.display();
        let dev = |what| format!("{this} --{what} -p {}", self.platform);

        let mut tmux = Tmux::new(self.session, self.root_path);

        tmux.create()?;
        tmux.run(&dev("app"))?;

        tmux.split()?;
        tmux.run(&dev("server"))?;

        if matches!(self.platform, Platform::Android) {
            tmux.vsplit()?;
            tmux.run(&dev("logs"))?;
        }

        tmux.vsplit()?;
        tmux.run(
            "tailwind -i ./app/assets/input.css -o ./app/assets/tailwind.css \
             --watch",
        )?;

        tmux.attach()?;

        Ok(())
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
            .current_dir(&self.root_path)
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
}
