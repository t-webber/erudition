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

To proceed, you will need to install:

- `tmux` (should be in your package manager)
- [`cargo`](https://rust-lang.org/tools/install/)
- `dx` that can be installed with `cargo install dioxus-cli`
- `android-studio` with a working emulator, more information on how to set it up [here](https://dioxuslabs.com/learn/0.6/guides/mobile/#android).
- [`tailwindcss`](https://github.com/tailwindlabs/tailwindcss)

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
  -a, --app
          Run only the app

  -k, --kill
          Kill the running tmux session

  -l, --logs
          Run only the logs

  -n, --name <NAME>
          Name of the tmux session

          [default: erudition]

  -o, --open
          Open the running tmux session

  -p, --platform <PLATFORM>
          Platform on which to run the app

          Possible values:
          - android: Serve the app in an android emulator
          - desktop: Serve the app natively
          - web:     Serve the app in the browser

          [default: desktop]

  -s, --server
          Run only the server

  -h, --help
          Print help (see a summary with '-h')
```

## Structure

```txt
.
├── app         <- front-end android app
│   └── assets  <- front-end assets (styles, images, etc.)
├── dev         <- dev script to ease development
├── lib         <- shared code between app and server
│                       to ensure correctness of serialisation
├── server      <- server, supposed to run on a different machine
└── target      <- builds folder, not committed
```
