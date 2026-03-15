use actix_web::web::Data;
use actix_web::{App, HttpServer};
use clap::Parser;
use color_eyre::eyre::Context as _;

use crate::routes::{add, list};
use crate::state::ServerState;

#[derive(Parser)]
pub struct Server {
    #[arg(short = 'F', long, default_value = "data")]
    file: String,
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,
    #[arg(short = 'P', long, default_value_t = 3000)]
    port: u16,
}

impl Server {
    pub async fn run(self) -> color_eyre::Result<()> {
        let state = Data::new(ServerState::load(self.file)?);

        println!(
            "Erudition-server running on http://{}:{}",
            self.host, self.port
        );

        HttpServer::new(move || {
            App::new()
                .app_data(state.clone())
                .service(list)
                .service(add)
        })
        .bind((self.host, self.port))
        .context("Failed to bind at localhost:8080")?
        .run()
        .await
        .context("Failed to run server")
    }
}
