use std::path::PathBuf;
use std::{fs, io};

use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use clap::Parser;
use color_eyre::eyre::{Context as _, ContextCompat as _};

use crate::routes::register_routes;
use crate::state::ServerState;

/// Server app that runs with the given parameters.
#[derive(Parser)]
pub struct Server {
    /// Path to the folder where to store the state and logs of the server to
    /// make them persistent.
    ///
    /// Defaults to a folder `erudition/` in the 'data dir'
    /// folder.
    #[arg(short = 'F', long)]
    folder_path: Option<PathBuf>,
    /// Host to use for serving the app, defaults to
    /// localhost.
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,
    /// Port to use for serving the app, defaults to 3000.
    ///
    /// Defaults to a file erudition/log in the 'data dir'
    /// folder.
    #[arg(short = 'P', long, default_value_t = 3000)]
    port: u16,
}

impl Server {
    /// Resolves a path in case it is not provided as a CLI
    /// argument.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment is missing a crucial variable, like
    /// `HOME`.
    fn data_dir(cli_path: Option<PathBuf>) -> color_eyre::Result<PathBuf> {
        let path = if let Some(path) = cli_path {
            path
        } else {
            dirs::data_dir()
                .context(if cfg!(target_os = "windows") {
                    "Your environment seems to be broken: \
                     FOLDERID_RoamingAppData variable does not exist"
                } else {
                    "Your environment seems to be broken: HOME variable does \
                     not exist"
                })
                .map(|path| path.join("erudition"))?
        };

        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to mkdir {}", path.display()))?;

        Ok(path)
    }

    /// Runs the app.
    ///
    /// # Errors
    ///
    /// Returns an error if failed to initialise the app's state or to start it.
    /// Once running, it should handle errors and never return one.
    pub fn run(self) -> color_eyre::Result<()> {
        let state =
            Data::new(ServerState::load(Self::data_dir(self.folder_path)?)?);

        state.log(&format!(
            "Erudition-server running on http://{}:{}",
            self.host, self.port
        ));

        Self::serve(state, self.host, self.port).context("Failed to run server")
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
