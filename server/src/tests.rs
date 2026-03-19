#![allow(clippy::restriction, reason = "tests are reproductable")]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use actix_web::http::StatusCode;
use actix_web::test::{TestRequest, call_service, init_service};
use actix_web::web::Data;
use actix_web::{App, test};

use crate::item::Item;
use crate::routes::register_routes;
use crate::state::ServerState;

fn cargo_metadata() -> String {
    String::from_utf8(
        Command::new("cargo")
            .args(["metadata", "--format-version=1", "--no-deps"])
            .output()
            .expect("failed to run cargo metadata")
            .stdout,
    )
    .expect("invalid cargo metadata output: not utf8")
}

fn default_target_dir() -> PathBuf {
    let metadata = cargo_metadata();
    let key = "\"target_directory\":\"";
    let start = metadata.find(key).expect(
        "invalid cargo metadata output: missing key 'target_directory'",
    ) + key.len();
    let end = metadata[start..]
        .find('"')
        .expect("invalid cargo metadata output: \" not found")
        + start;

    PathBuf::from(&metadata[start..end])
}

fn target_dir() -> PathBuf {
    if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        return dir.into();
    }
    default_target_dir()
}

fn ensure_not_exists(folder: &Path) {
    if folder.exists() {
        fs::remove_dir_all(folder).unwrap();
    }
}

fn state(folder: &Path) -> Data<ServerState> {
    Data::new(ServerState::load(folder.to_path_buf()).unwrap())
}

macro_rules! app {
    ($state:expr) => {
        init_service(App::new().app_data($state).configure(register_routes))
            .await
    };
}

macro_rules! get {
    ($app:expr, $uri:expr) => {{
        let req = TestRequest::get().uri($uri).to_request();
        res!($app, req)
    }};
}

macro_rules! res {
    ($app:expr, $req:expr) => {{
        let app = $app;
        let res = call_service(&app, $req).await;
        assert!(res.status().is_success());
        String::from_utf8(test::read_body(res).await.to_vec()).unwrap()
    }};
}

#[actix_web::test]
async fn feedback() {
    let folder = target_dir().join("test").join("feedback");
    ensure_not_exists(&folder);

    let app = app!(state(&folder));

    assert_eq!(get!(&app, "/feedback"), "[]");

    let contents = vec!["Some content\n\u{2240}Heart", "", "\r", "."];

    for content in &contents {
        let req = TestRequest::post()
            .uri("/feedback")
            .set_payload(content.to_owned())
            .to_request();
        assert_eq!(res!(&app, req), "");
    }

    let ser = serde_json::to_string(&contents).unwrap();
    assert_eq!(get!(&app, "/feedback"), ser);
    assert_eq!(get!(&app!(state(&folder)), "/feedback"), ser);

    fs::remove_dir_all(&folder).unwrap();
    assert_eq!(get!(&app!(state(&folder)), "/feedback"), "[]");
}

#[actix_web::test]
async fn items() {
    let folder = target_dir().join("test").join("items");
    ensure_not_exists(&folder);

    let app = app!(state(&folder));

    assert_eq!(get!(&app, "/items"), "[]");

    macro_rules! item {
        ($question:literal, $($answer:literal),*) => {
            Item::MultipleChoice {
                answers: vec![ $($answer.to_string()),* ],
                question: $question.to_string(),
            }
        };
    }

    let mut items = vec![
        item!("a", "b", "c", "d"),
        item!("question", "answer1", "answer2"),
        item!("only_question",),
    ];

    for item in &items {
        let req = TestRequest::post().uri("/item").set_json(item).to_request();
        assert_eq!(res!(&app, req), "");
    }

    let new_first = item!("e", "f", "g", "h");
    items[0] = new_first.clone();
    let req =
        TestRequest::put().uri("/item/0").set_json(new_first).to_request();
    assert_eq!(res!(&app, req), "");

    let req = TestRequest::put()
        .uri("/item/123")
        .set_json(item!("a", "b"))
        .to_request();
    let res = call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let ser = serde_json::to_string(&items).unwrap();
    assert_eq!(get!(app, "/items"), ser);
    assert_eq!(get!(app!(state(&folder)), "/items"), ser);

    fs::remove_dir_all(&folder).unwrap();
    assert_eq!(get!(app!(state(&folder)), "/items"), "[]");
}

#[actix_web::test]
async fn server_error() {
    let folder = target_dir().join("test").join("server_error");

    let app = app!(state(&folder));
    ensure_not_exists(&folder);

    let req =
        TestRequest::post().uri("/feedback").set_payload("data").to_request();
    let res = call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[actix_web::test]
async fn not_found() {
    let folder = target_dir().join("test").join("not_found");

    let app = app!(state(&folder));
    ensure_not_exists(&folder);

    let req = TestRequest::get().uri("/not-a-valid-route").to_request();
    let res = call_service(&app, req).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[test]
async fn invalid_data() {
    let folder = target_dir().join("test").join("invalid_data");

    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("data"), "invalid data").unwrap();

    let e = ServerState::load(folder).unwrap_err();
    assert!(dbg!(e.to_string()).contains("has invalid data"));
}
