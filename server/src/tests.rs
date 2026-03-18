#![allow(clippy::restriction, reason = "tests are reproductable")]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use actix_web::test::{TestRequest, call_service, init_service};
use actix_web::web::Data;
use actix_web::{App, test};

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
    Data::new(
        ServerState::load(folder.join("data"), folder.join("logs")).unwrap(),
    )
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
async fn test_feedback() {
    let folder = target_dir().join("test").join("test_feedback");
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
    for app in [app, app!(state(&folder))] {
        assert_eq!(get!(&app, "/feedback"), ser);
    }

    fs::remove_dir_all(&folder).unwrap();
    assert_eq!(get!(&app!(state(&folder)), "/feedback"), "[]");
}
