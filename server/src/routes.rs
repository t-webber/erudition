use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::web::{Data, Json, Path};
use actix_web::{App, Error, HttpResponse, get, post, put};

use crate::item::Item;
use crate::state::{ItemError, ServerState};

#[get("/list")]
async fn list(state: Data<ServerState>) -> HttpResponse {
    state.log("GET: /list");
    HttpResponse::Ok().json(state.items())
}

#[put("/edit/{index}")]
async fn edit(state: Data<ServerState>, index: Path<usize>, item: Json<Item>) -> HttpResponse {
    state.log(&format!("POST: /edit/{index}: {item:?}"));
    if state.edit_item(index.into_inner(), item.into_inner()) {
        HttpResponse::Ok().into()
    } else {
        HttpResponse::BadRequest().body("Index is not valid")
    }
}

#[post("/add")]
async fn add(item: Json<Item>, state: Data<ServerState>) -> HttpResponse {
    state.log(&format!("POST: /add: {item:?}"));
    state.add_item(item.into_inner());
    match state.store() {
        Ok(()) => HttpResponse::Ok().into(),
        Err(ItemError::PostCard(ser)) => {
            state.log(&format!(
                "Failed to serialise items to disk:\nItems:\n{:?}\n\nError:\n{ser}",
                state.items()
            ));
            HttpResponse::UnprocessableEntity().body(format!("Failed to serialise data: {ser}"))
        }
        Err(ItemError::Io(io)) => {
            state.log(&format!("Failed to save items to disk: {io}"));
            HttpResponse::InternalServerError().into()
        }
    }
}

/// Registers all the routes of the app from the given file.
///
/// Works better here to not forget to register them.
pub fn register_routes<
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
>(
    app: App<T>,
) -> App<T> {
    app.service(add).service(list).service(edit)
}
