//! The font library: upload validation, listing, deletion, and the setup gate.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "allow-unwrap-in-tests does not cover helper functions of integration test crates"
)]

#[allow(
    dead_code,
    reason = "each test file uses a different subset of the shared helpers"
)]
mod common;

use axum::http::StatusCode;
use serde_json::json;

use crate::common::{
    PNG, TestApp,
    font::{REGULAR, minimal_ttf, upload},
};

const FAMILY: &str = "Harbor Sans";

#[tokio::test]
async fn uploads_must_be_fonts() {
    let app = TestApp::new().await;
    let png = upload(&app, PNG.to_vec()).await;
    png.assert_error(StatusCode::BAD_REQUEST, "validation");
    assert_eq!(png.field(), "file");
    let broken = upload(&app, vec![0, 1, 0, 0, 0, 0, 0, 0]).await;
    broken.assert_error(StatusCode::BAD_REQUEST, "validation");
    upload(&app, Vec::new())
        .await
        .assert_error(StatusCode::BAD_REQUEST, "validation");
    assert_eq!(app.api("GET", "/api/fonts", None).await.json(), json!([]));
    let files = std::fs::read_dir(app.data().join("fonts")).unwrap().count();
    assert_eq!(files, 0);
}

#[tokio::test]
async fn a_font_file_is_stored_listed_and_deleted() {
    let app = TestApp::new().await;
    app.seed_cache();
    let font = minimal_ttf(FAMILY, REGULAR);
    let res = upload(&app, font.clone()).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let item = res.json();
    let name = item["name"].as_str().unwrap().to_owned();
    assert!(
        name.starts_with("HarborSans-") && name.ends_with(".ttf"),
        "{name}"
    );
    assert_eq!(item["families"], json!([FAMILY]));
    assert_eq!(item["source"], "upload");
    assert_eq!(item["size"], font.len());
    assert!(app.cache_is_cleared());
    assert_eq!(
        std::fs::read(app.data().join("fonts").join(&name)).unwrap(),
        font
    );
    let list = app.api("GET", "/api/fonts", None).await.json();
    assert_eq!(list, json!([item]));
    let path = format!("/api/fonts/{name}");
    assert_eq!(
        app.api("DELETE", &path, None).await.status,
        StatusCode::NO_CONTENT
    );
    assert!(!app.data().join("fonts").join(&name).exists());
    app.api("DELETE", &path, None)
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    app.api("DELETE", "/api/fonts/..%2Fcuhlumn.db", None)
        .await
        .assert_error(StatusCode::BAD_REQUEST, "validation");
}

#[tokio::test]
async fn google_import_accepts_only_google_addresses() {
    let app = TestApp::new().await;
    for url in [
        "https://harbor.example.com/specimen/Inter",
        "http://fonts.googleapis.com/css2?family=Inter",
        "https://fonts.googleapis.com/icon?family=Material+Icons",
    ] {
        let res = app
            .api("POST", "/api/fonts/google", Some(json!({"url": url})))
            .await;
        res.assert_error(StatusCode::BAD_REQUEST, "validation");
        assert_eq!(res.field(), "url");
    }
}
