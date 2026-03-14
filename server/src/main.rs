use std::sync::Mutex;

use actix_web::web::Data;
use actix_web::{App, HttpResponse, HttpServer, Responder, get};
use serde::Serialize;

#[derive(Serialize)]
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let items = Data::new(Items::default());

    HttpServer::new(move || App::new().app_data(items.clone()).service(list))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
