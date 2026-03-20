use core::time::Duration;
use std::process::Command;
use std::thread::sleep;

use clap::Parser;

#[derive(Parser)]
struct Dev {
    #[arg(short, long, default_value = "erudition")]
    session: String,
    #[arg(short, long, default_value_t = false)]
    kill: bool,
    #[arg(short, long, default_value_t = 2000)]
    logs_delay: u32,
}

impl Dev {
    fn tmux(args: &[&str]) {
        Command::new("tmux").args(args).status().unwrap();
    }

    fn send_keys(&self, window: &str, keys: &str) {
        Self::tmux(&[
            "send-keys",
            "-t",
            &format!("{}:{window}", self.session),
            keys,
            "C-m",
        ])
    }

    fn run(self) {
        if self.kill {
            Self::tmux(&["kill-session", "-t", &self.session]);
            return;
        }

        Self::tmux(&["new-session", "-d", "-s", &self.session, "-n", "server"]);
        self.send_keys(
            "server",
            "builtin cd server && $(/bin/which cargo) run",
        );
        Self::tmux(&["new-window", "-t", &self.session, "-n", "app"]);
        self.send_keys("app", "builtin cd app && dx serve --android");

        sleep(Duration::from_millis(2000));

        Self::tmux(&["new-window", "-t", &self.session, "-n", "log"]);
        self.send_keys(
            "log",
            r"adb logcat | /bin/grep RustStdoutStderr | /bin/grep -v s_glBindAttribLocation | sed 's/\(.* .*\) [0-9]* [0-9]* I RustStdoutStderr:/\x1b[33m\1\x1b[0m/' | sed 's/\[\([^][]*\)\]/\x1b[37m[\1]\x1b[0m/g'",
        );

        Self::tmux(&["attach-session", "-t", &self.session]);
    }
}

fn main() {
    Dev::parse().run()
}
