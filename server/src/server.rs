use std::path::PathBuf;
use std::{fs, io};

use actix_web::web::Data;
use actix_web::{App, HttpServer};
use clap::Parser;
use color_eyre::eyre::{Context as _, ContextCompat as _, ensure};

use crate::routes::register_routes;
use crate::state::ServerState;

/// Server app that runs with the given parameters
#[derive(Parser)]
pub struct Server {
    /// Path to the file where to store the state of the
    /// server to make it persistant
    ///
    /// Defaults to a file erudition/data in the 'data dir'
    /// folder
    #[arg(short = 'D', long)]
    data_path: Option<PathBuf>,
    /// Host to use for serving the app, defaults to
    /// localhost
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,
    /// Path to the file where to store the state of the
    /// server to make it persistant
    #[arg(short = 'L', long)]
    log_path: Option<PathBuf>,
    /// Port to use for serving the app, defaults to 3000
    ///
    /// Defaults to a file erudition/log in the 'data dir'
    /// folder
    #[arg(short = 'P', long, default_value_t = 3000)]
    port: u16,
}

impl Server {
    /// Resolves a path in case it is not provided as a CLI
    /// argument.
    fn resolve_path(
        cli_path: Option<PathBuf>,
        default_name: &str,
    ) -> color_eyre::Result<PathBuf> {
        if let Some(path) = cli_path {
            return Ok(path);
        }

        let parent = dirs::data_dir()
            .context(if cfg!(target_os = "windows") {
                "Your environment seems to be broken: FOLDERID_RoamingAppData \
                 variable does not exist"
            } else {
                "Your environment seems to be broken: HOME variable does not \
                 exist"
            })
            .map(|path| path.join("erudition"))?;

        fs::create_dir_all(&parent)
            .with_context(|| format!("Failed to mkdir {}", parent.display()))?;

        Ok(parent.join(default_name))
    }

    /// Runs the app
    pub fn run(self) -> color_eyre::Result<()> {
        let data_path = Self::resolve_path(self.data_path, "data")?;
        let log_path = Self::resolve_path(self.log_path, "log")?;

        ensure!(
            data_path != log_path,
            "Log and data path shouldn't be the same."
        );

        let state = Data::new(ServerState::load(data_path, log_path)?);

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
            App::new().app_data(state.clone()).configure(register_routes)
        })
        .bind((host, port))?
        .run()
        .await
    }
}
