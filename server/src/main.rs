mod item;
mod routes;
mod server;
mod state;

use clap::Parser;

use crate::server::Server;

#[actix_web::main]
async fn main() -> color_eyre::Result<()> {
    Server::parse().run().await
}
