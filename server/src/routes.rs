use actix_web::HttpResponse;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::web::{Data, Json, Path, ServiceConfig};
use erudition_lib::{Auth, Item, SessionId};
use erudition_proc_macro::{get, post, put, routes};

use crate::state::ServerState;

/// Creates the response that matches an authentication request from a session
/// id.
fn auth(res: Option<SessionId>) -> HttpResponse {
    res.map_or_else(
        || HttpResponse::Unauthorized().into(),
        |session_id| {
            let cookie = Cookie::build("session_id", &*session_id.0)
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .path("/")
                .finish();
            HttpResponse::Ok().cookie(cookie).finish()
        },
    )
}

#[post("/signin")]
async fn signin(state: Data<ServerState>, body: Json<Auth>) -> HttpResponse {
    auth(state.signin(body.into_inner()))
}

#[post("/login")]
async fn login(state: Data<ServerState>, body: Json<Auth>) -> HttpResponse {
    auth(state.login(body.into_inner()))
}

#[get("/items")]
async fn get_items(state: Data<ServerState>) -> HttpResponse {
    HttpResponse::Ok().json(state.get_items())
}

#[post("/item")]
async fn post_item(item: Json<Item>, state: Data<ServerState>) -> HttpResponse {
    handle_internal_error(state.add_item(item.into_inner()))
}

#[put("/item/{index}")]
async fn put_item(
    state: Data<ServerState>,
    index: Path<usize>,
    item: Json<Item>,
) -> HttpResponse {
    state.edit_item(index.into_inner(), item.into_inner()).map_or_else(
        || HttpResponse::BadRequest().body("Index is not valid"),
        handle_internal_error,
    )
}

#[get("/feedback")]
async fn get_feedback(state: Data<ServerState>) -> HttpResponse {
    HttpResponse::Ok().json(state.get_feedback())
}

#[post("/feedback")]
async fn post_feedback(data: String, state: Data<ServerState>) -> HttpResponse {
    if data.is_empty() {
        HttpResponse::BadRequest().into()
    } else {
        handle_internal_error(state.add_feedback(data))
    }
}

/// Registers all the routes of the app from the given file.
///
/// Works better here to not forget to register them.
pub fn register_routes(app: &mut ServiceConfig) {
    routes!(app);
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
