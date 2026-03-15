use actix_web::web::{Data, Json};
use actix_web::{HttpResponse, get, post};

use crate::item::Item;
use crate::state::{ItemError, ServerState};

#[get("/list")]
pub async fn list(items: Data<ServerState>) -> HttpResponse {
    HttpResponse::Ok().json(items)
}

#[post("/add")]
pub async fn add(item: Json<Item>, state: Data<ServerState>) -> HttpResponse {
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
