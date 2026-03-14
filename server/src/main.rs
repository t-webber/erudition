use std::sync::Mutex;

use actix_web::web::{Data, Json};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Item {
    MultipleChoice {
        question: String,
        answers: Vec<String>,
    },
}

#[derive(Default, Serialize)]
struct Items(Mutex<Vec<Item>>);

#[get("/list")]
async fn list(items: Data<Items>) -> impl Responder {
    HttpResponse::Ok().json(items)
}

#[post("/add")]
async fn add(item: Json<Item>, items: Data<Items>) -> impl Responder {
    items.0.lock().unwrap().push(item.into_inner());
    HttpResponse::Ok()
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
