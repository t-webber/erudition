//! Server for the erudition app.

/// Server routes and their handler.
mod routes;
/// Server cli and runner.
mod server;
/// Server state, shared across route handlers.
mod state;
/// State that is stored to the file system to be persistent after the server is
/// restarted.
mod storage;
#[cfg(test)]
mod tests;

use clap::Parser as _;

use crate::server::Server;

/// # Errors
///
/// Returns an error if the app failed to initialise with the given parameters.
/// Once running, it shouldn't return.
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Server::parse().run()
}
