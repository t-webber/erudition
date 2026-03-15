use std::fs;
use std::io;
use std::sync::Mutex;

use actix_web::web::{Data, Json};
use actix_web::{App, HttpResponse, HttpServer, get, post};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum Item {
    MultipleChoice {
        question: String,
        answers: Vec<String>,
    },
}

#[derive(Default, Serialize, Debug)]
struct Items(Mutex<Vec<Item>>);

enum SaveError {
    Io(io::Error),
    Serialisation(postcard::Error),
}

impl Items {
    fn save(&self) -> Result<(), SaveError> {
        let data = postcard::to_allocvec(&self).map_err(SaveError::Serialisation)?;
        fs::write("data", data).map_err(SaveError::Io)?;
        Ok(())
    }
}

#[get("/list")]
async fn list(items: Data<Items>) -> HttpResponse {
    HttpResponse::Ok().json(items)
}

#[post("/add")]
async fn add(item: Json<Item>, items: Data<Items>) -> HttpResponse {
    items.0.lock().unwrap().push(item.into_inner());
    match items.save() {
        Ok(()) => HttpResponse::Ok().into(),
        Err(SaveError::Serialisation(ser)) => {
            eprintln!("Failed to serialise items to disk:\nItems:\n{items:?}\n\nError:\n{ser}");
            HttpResponse::UnprocessableEntity().body(format!("Failed to serialise data: {ser}"))
        }
        Err(SaveError::Io(io)) => {
            eprintln!("Failed to save items to disk: {io}");
            HttpResponse::InternalServerError().into()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let items = Data::new(Items::default());

    HttpServer::new(move || {
        App::new()
            .app_data(items.clone())
            .service(list)
            .service(add)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
