use std::io;
use std::path::PathBuf;

use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use clap::Parser;
use color_eyre::eyre::Context as _;

use crate::dir::DataDir;
use crate::routes::register_routes;
use crate::state::ServerState;

/// Server app that runs with the given parameters.
#[derive(Parser)]
pub struct Server {
    /// Host to use for serving the app, defaults to
    /// localhost.
    #[arg(short = 'a', long, default_value = "localhost")]
    address: String,
    /// Path to the folder where to store the state and logs of the server to
    /// make them persistent.
    ///
    /// Defaults to a folder `erudition/` in the 'data dir'
    /// folder.
    #[arg(short = 'f', long)]
    folder_path: Option<PathBuf>,
    /// Option to initialise the server data from the `csv` files in the data
    /// folder.
    #[arg(short, long, conflicts_with = "address", conflicts_with = "port")]
    initialise: bool,
    /// Port to use for serving the app, defaults to 3000.
    ///
    /// Defaults to a file erudition/log in the 'data dir'
    /// folder.
    #[arg(short = 'p', long, default_value_t = 3000)]
    port: u16,
}

impl Server {
    /// Runs the app.
    ///
    /// # Errors
    ///
    /// Returns an error if failed to initialise the app's state or to start it.
    /// Once running, it should handle errors and never return one.
    pub fn run(self) -> color_eyre::Result<()> {
        let data_dir = DataDir::new(self.folder_path)?;

        let state = Data::new(ServerState::load(data_dir)?);

        state.log(&format!(
            "Erudition-server running on http://{}:{}",
            self.address, self.port
        ));

        Self::serve(state, self.address, self.port)
            .context("Failed to run server")
    }

    #[actix_web::main]
    async fn serve(
        state: Data<ServerState>,
        host: String,
        port: u16,
    ) -> io::Result<()> {
        HttpServer::new(move || {
            let cors = Cors::permissive();

            App::new()
                .wrap(cors)
                .app_data(state.clone())
                .configure(register_routes)
        })
        .bind((host, port))?
        .run()
        .await
    }
}
