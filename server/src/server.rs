use std::io;

use actix_web::web::Data;
use actix_web::{App, HttpServer};
use clap::Parser;
use color_eyre::eyre::Context as _;

use crate::routes::{add, list};
use crate::state::ServerState;

/// Server app that runs with the given parameters
#[derive(Parser)]
pub struct Server {
    /// Path to the file where to store the state of the server to make it persistant
    #[arg(short = 'D', long, default_value = "data")]
    data_path: String,
    /// Host to use for serving the app, defaults to localhost
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,
    /// Path to the file where to store the state of the server to make it persistant
    #[arg(short = 'L', long, default_value = "log")]
    log_path: String,
    /// Port to use for serving the app, defaults to 3000
    #[arg(short = 'P', long, default_value_t = 3000)]
    port: u16,
}

impl Server {
    /// Runs the app
    pub fn run(self) -> color_eyre::Result<()> {
        let state = Data::new(ServerState::load(self.data_path, self.log_path)?);

        state.log(&format!(
            "Erudition-server running on http://{}:{}",
            self.host, self.port
        ));

        Self::serve(state, self.host, self.port).context("Failed to run server")
    }

    #[actix_web::main]
    async fn serve(state: Data<ServerState>, host: String, port: u16) -> io::Result<()> {
        HttpServer::new(move || {
            App::new()
                .app_data(state.clone())
                .service(list)
                .service(add)
        })
        .bind((host, port))?
        .run()
        .await
    }
}
