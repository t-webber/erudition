use std::fs;
use std::io;
use std::sync::Mutex;

use actix_web::web::{Data, Json};
use actix_web::{App, HttpResponse, HttpServer, get, post};
use clap::Parser;
use color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum Item {
    MultipleChoice {
        question: String,
        answers: Vec<String>,
    },
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct ServerState {
    items: Mutex<Vec<Item>>,
    file: String,
}

enum ItemError {
    Io(io::Error),
    PostCard(postcard::Error),
}

impl ServerState {
    fn store(&self) -> Result<(), ItemError> {
        let data = postcard::to_allocvec(&self).map_err(ItemError::PostCard)?;
        fs::write(&self.file, data).map_err(ItemError::Io)?;
        Ok(())
    }

    fn load(file: String) -> color_eyre::Result<Self> {
        Ok(Self {
            items: postcard::from_bytes(
                fs::read_to_string(&file)
                    .with_context(|| format!("Failed to read {file}"))?
                    .as_bytes(),
            )
            .with_context(|| format!("File {file} has invalid data"))?,
            file,
        })
    }
}

#[get("/list")]
async fn list(items: Data<ServerState>) -> HttpResponse {
    HttpResponse::Ok().json(items)
}

#[post("/add")]
async fn add(item: Json<Item>, state: Data<ServerState>) -> HttpResponse {
    state.items.lock().unwrap().push(item.into_inner());
    match state.store() {
        Ok(()) => HttpResponse::Ok().into(),
        Err(ItemError::PostCard(ser)) => {
            eprintln!(
                "Failed to serialise items to disk:\nItems:\n{:?}\n\nError:\n{ser}",
                state.items
            );
            HttpResponse::UnprocessableEntity().body(format!("Failed to serialise data: {ser}"))
        }
        Err(ItemError::Io(io)) => {
            eprintln!("Failed to save items to disk: {io}");
            HttpResponse::InternalServerError().into()
        }
    }
}

#[derive(Parser)]
struct Server {
    #[arg(short = 'F', long, default_value = "data")]
    file: String,
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,
    #[arg(short = 'P', long, default_value_t = 3000)]
    port: u16,
}

impl Server {
    async fn run(self) -> color_eyre::Result<()> {
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

#[actix_web::main]
async fn main() -> color_eyre::Result<()> {
    Server::parse().run().await
}
