# Erudition

![Clippy](https://github.com/t-webber/erudition/actions/workflows/clippy.yml/badge.svg?branch=main)
![Build](https://github.com/t-webber/erudition/actions/workflows/build.yml/badge.svg?branch=main)
![Tests](https://github.com/t-webber/erudition/actions/workflows/tests.yml/badge.svg?branch=main)
![Rustdoc](https://github.com/t-webber/erudition/actions/workflows/rustdoc.yml/badge.svg?branch=main)
![Rusfmt](https://github.com/t-webber/erudition/actions/workflows/rustfmt.yml/badge.svg?branch=main)
![Taplo](https://github.com/t-webber/erudition/actions/workflows/taplo.yml/badge.svg?branch=main)
![Spelling](https://github.com/t-webber/erudition/actions/workflows/spelling.yml/badge.svg?branch=main)

[![github](https://img.shields.io/badge/GitHub-t--webber/erudition-blue?logo=GitHub)](https://github.com/t-webber/erudition)
[![license](https://img.shields.io/badge/Licence-MIT%20or%20Apache%202.0-darkgreen)](https://github.com/t-webber/erudition?tab=MIT-2-ov-file)
[![rust-edition](https://img.shields.io/badge/Rust--edition-2024-darkred?logo=Rust)](https://doc.rust-lang.org/stable/edition-guide/rust-2024/)

## Development

To proceed, you will need to instal:

- `tmux` (should be in your package manager)
- [`cargo`](https://rust-lang.org/tools/install/)
- `dx` that can be installed with `cargo install dioxus-cli`
- `android-studio` with a working emulator, more information on how to set it up [here](https://dioxuslabs.com/learn/0.6/guides/mobile/#android).

Then, the only thing you need to run is:

```bash
cargo dev
```

This will open a `tmux` with the app, the server, and the logs all in one place. You can also pass in arguments:

```bash
cargo dev --help
```

```txt
A runner that handles all different aspects (server, app, logs) with one command

Usage: dev [OPTIONS]

Options:
  -k, --kill                     Kill the running tmux session
  -l, --logs-delay <LOGS_DELAY>  Change the delay between launching the app and opening the logs [default: 2000]
  -o, --open                     Open the running tmux session
  -s, --session <SESSION>        Name of the tmux session [default: erudition]
  -h, --help                     Print help
```

## Structure

```txt
.
├── app     <- front-end android app
├── dev     <- dev script to ease development
├── lib     <- shared code between app and server
                   to ensure correctness of serde
├── server  <- server
└── target  <- builds folder, not committed
```
