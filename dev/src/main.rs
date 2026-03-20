use core::time::Duration;
use std::process::Command;
use std::thread;

fn tmux(args: &[&str]) {
    Command::new("tmux").args(args).status().unwrap();
}

fn send_keys(session: &str, window: &str, keys: &str) {
    tmux(&["send-keys", "-t", &format!("{session}:{window}"), keys, "C-m"])
}

fn main() {
    let session = "erudition";

    tmux(&["new-session", "-d", "-s", session, "-n", "server"]);
    send_keys(
        session,
        "server",
        "builtin cd server && $(/bin/which cargo) run",
    );
    tmux(&["new-window", "-t", session, "-n", "app"]);
    send_keys(session, "app", "builtin cd app && dx serve --android");

    thread::sleep(Duration::from_secs(2));

    tmux(&["new-window", "-t", session, "-n", "log"]);
    send_keys(
        session,
        "log",
        r"adb logcat | /bin/grep RustStdoutStderr | /bin/grep -v s_glBindAttribLocation | sed 's/\(.* .*\) [0-9]* [0-9]* I RustStdoutStderr:/\x1b[33m\1\x1b[0m/' | sed 's/\[\([^][]*\)\]/\x1b[37m[\1]\x1b[0m/g'",
    );

    tmux(&["attach-session", "-t", session]);
}
