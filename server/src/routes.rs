use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::web::{Data, Json, Path};
use actix_web::{App, Error, HttpResponse, get, post, put};

use crate::item::Item;
use crate::state::ServerState;

#[get("/list")]
async fn list(state: Data<ServerState>) -> HttpResponse {
    state.log("GET: /list");
    HttpResponse::Ok().json(state.items())
}

#[put("/edit/{index}")]
async fn edit(
    state: Data<ServerState>,
    index: Path<usize>,
    item: Json<Item>,
) -> HttpResponse {
    state.log(&format!("POST: /edit/{index}: {item:?}"));
    state.edit_item(index.into_inner(), item.into_inner()).map_or_else(
        || HttpResponse::BadRequest().body("Index is not valid"),
        handle_internal_error,
    )
}

#[post("/add")]
async fn add(item: Json<Item>, state: Data<ServerState>) -> HttpResponse {
    state.log(&format!("POST: /add: {item:?}"));
    handle_internal_error(state.add_item(item.into_inner()))
}

/// Registers all the routes of the app from the given file.
///
/// Works better here to not forget to register them.
pub fn register_routes<T>(app: App<T>) -> App<T>
where T: ServiceFactory<
            ServiceRequest,
            Config = (),
            Error = Error,
            InitError = (),
        > {
    app.service(add).service(list).service(edit)
}

/// From a boolean indicating if an internal error occurence, create an
/// [`HttpResponse`] that is either `200 OK` or `500 Internal Server Error`.
fn handle_internal_error(is_internal_error: bool) -> HttpResponse {
    if is_internal_error {
            HttpResponse::Ok()
        } else {
            HttpResponse::InternalServerError()
        }
        .into()
}
