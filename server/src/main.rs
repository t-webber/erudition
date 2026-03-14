use actix_web::{App, HttpResponse, HttpServer, Responder, get};

#[get("/list")]
async fn list() -> impl Responder {
    HttpResponse::Ok().json("Hello world!\n")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(list))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
