//! Server for the erudition app.

/// Server cli and runner.
mod cli;
/// Stores the location at which the data should be stored to and loaded from.
mod dir;
/// Initialises data from some `csv` files.
mod initialise;
/// Server routes and their handler.
mod routes;
/// Server state, shared across route handlers.
mod state;
/// State that is stored to the file system to be persistent after the server is
/// restarted.
mod storage;
#[cfg(test)]
mod tests;

use clap::Parser as _;

use crate::cli::Server;

/// # Errors
///
/// Returns an error if the app failed to initialise with the given parameters.
/// Once running, it shouldn't return.
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Server::parse().run()
}
