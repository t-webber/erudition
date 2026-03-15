use actix_web::web::{Data, Json};
use actix_web::{HttpResponse, get, post};

use crate::item::Item;
use crate::state::{ItemError, ServerState};

#[get("/list")]
pub async fn list(items: Data<ServerState>, state: Data<ServerState>) -> HttpResponse {
    state.log("GET: /list");
    HttpResponse::Ok().json(items)
}

#[post("/add")]
pub async fn add(item: Json<Item>, state: Data<ServerState>) -> HttpResponse {
    state.log(&format!("POST: /add: {item:?}"));
    match state.items.lock() {
        Ok(ref mut items) => items.push(item.into_inner()),
        Err(ref mut err) => {
            err.get_mut().push(item.into_inner());
        }
    }
    match state.store() {
        Ok(()) => HttpResponse::Ok().into(),
        Err(ItemError::PostCard(ser)) => {
            state.log(&format!(
                "Failed to serialise items to disk:\nItems:\n{:?}\n\nError:\n{ser}",
                state.items
            ));
            HttpResponse::UnprocessableEntity().body(format!("Failed to serialise data: {ser}"))
        }
        Err(ItemError::Io(io)) => {
            state.log(&format!("Failed to save items to disk: {io}"));
            HttpResponse::InternalServerError().into()
        }
    }
}
