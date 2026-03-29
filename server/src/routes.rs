use actix_web::cookie::{Cookie, SameSite};
use actix_web::web::{Data, Json, Path, ServiceConfig};
use actix_web::{HttpResponse, get, post, put};
use erudition_lib::{Auth, Item};

use crate::state::ServerState;

#[post("/auth")]
async fn auth(state: Data<ServerState>, body: Json<Auth>) -> HttpResponse {
    state.log("POST: /auth");
    let session_id = state.authenticate(body.into_inner());
    let cookie = Cookie::build("session_id", session_id)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .finish();
    HttpResponse::Ok().cookie(cookie).finish()
}

#[get("/items")]
async fn get_items(state: Data<ServerState>) -> HttpResponse {
    state.log("GET: /items");
    HttpResponse::Ok().json(state.get_items())
}

#[post("/item")]
async fn post_item(item: Json<Item>, state: Data<ServerState>) -> HttpResponse {
    state.log(&format!("POST: /item: {item:?}"));
    handle_internal_error(state.add_item(item.into_inner()))
}

#[put("/item/{index}")]
async fn put_item(
    state: Data<ServerState>,
    index: Path<usize>,
    item: Json<Item>,
) -> HttpResponse {
    state.log(&format!("PUT: /item/{index}: {item:?}"));
    state.edit_item(index.into_inner(), item.into_inner()).map_or_else(
        || HttpResponse::BadRequest().body("Index is not valid"),
        handle_internal_error,
    )
}

#[get("/feedback")]
async fn get_feedback(state: Data<ServerState>) -> HttpResponse {
    state.log("GET: /feedback");
    HttpResponse::Ok().json(state.get_feedback())
}

#[post("/feedback")]
async fn post_feedback(data: String, state: Data<ServerState>) -> HttpResponse {
    state.log(&format!("POST: /feedback: {data:?}"));
    handle_internal_error(state.add_feedback(data))
}

/// Registers all the routes of the app from the given file.
///
/// Works better here to not forget to register them.
pub fn register_routes(app: &mut ServiceConfig) {
    app.service(get_items)
        .service(post_item)
        .service(put_item)
        .service(auth)
        .service(get_feedback)
        .service(post_feedback);
}

/// From a boolean indicating if an internal error occurrence, create an
/// [`HttpResponse`] that is either `200 OK` or `500 Internal Server Error`.
fn handle_internal_error(is_internal_error: bool) -> HttpResponse {
    if is_internal_error {
        HttpResponse::Ok()
    } else {
        HttpResponse::InternalServerError()
    }
    .into()
}
