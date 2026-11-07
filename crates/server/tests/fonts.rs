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

use axum::{body::Body, http::StatusCode};
use serde_json::json;

use crate::common::{ADMIN_HOST, PNG, Res, TOKEN, TestApp, request};

const FAMILY: &str = "Harbor Sans";

async fn upload(app: &TestApp, body: Vec<u8>) -> Res {
    let builder = request("POST", ADMIN_HOST, "/api/fonts")
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "font/ttf");
    app.send(builder.body(Body::from(body)).unwrap()).await
}

/// Builds a minimal TrueType file with the tables `head`, `hhea`, `maxp`, and `name`.
fn minimal_ttf(family: &str) -> Vec<u8> {
    let tables: [(&[u8; 4], Vec<u8>); 4] = [
        (b"head", head()),
        (b"hhea", hhea()),
        (b"maxp", [0, 0, 0x50, 0, 0, 1].to_vec()),
        (b"name", name(family)),
    ];
    let mut font = Vec::new();
    font.extend_from_slice(&[0, 1, 0, 0]);
    font.extend_from_slice(&4_u16.to_be_bytes());
    font.extend_from_slice(&[0, 64, 0, 2, 0, 0]);
    let mut offset = 12 + 16 * tables.len();
    let mut data = Vec::new();
    for (tag, table) in &tables {
        font.extend_from_slice(*tag);
        font.extend_from_slice(&0_u32.to_be_bytes());
        font.extend_from_slice(&u32::try_from(offset).unwrap().to_be_bytes());
        font.extend_from_slice(&u32::try_from(table.len()).unwrap().to_be_bytes());
        let mut padded = table.clone();
        padded.resize(table.len().div_ceil(4) * 4, 0);
        offset += padded.len();
        data.extend(padded);
    }
    font.extend(data);
    font
}

fn head() -> Vec<u8> {
    let mut table = Vec::new();
    table.extend_from_slice(&[0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
    table.extend_from_slice(&0x5F0F_3CF5_u32.to_be_bytes());
    table.extend_from_slice(&0_u16.to_be_bytes());
    table.extend_from_slice(&1000_u16.to_be_bytes());
    table.extend_from_slice(&[0; 16]);
    table.extend_from_slice(&[0, 0, 0, 0, 0x03, 0xE8, 0x03, 0xE8]);
    table.extend_from_slice(&[0, 0, 0, 8, 0, 2, 0, 0, 0, 0]);
    assert_eq!(table.len(), 54);
    table
}

fn hhea() -> Vec<u8> {
    let mut table = vec![0, 1, 0, 0];
    table.extend_from_slice(&800_i16.to_be_bytes());
    table.extend_from_slice(&(-200_i16).to_be_bytes());
    table.extend_from_slice(&[0; 26]);
    table.extend_from_slice(&1_u16.to_be_bytes());
    assert_eq!(table.len(), 36);
    table
}

fn name(family: &str) -> Vec<u8> {
    let postscript = family.replace(' ', "");
    let strings: Vec<(u16, Vec<u8>)> = [(1, family), (2, "Regular"), (4, family), (6, &postscript)]
        .into_iter()
        .map(|(id, text)| (id, text.encode_utf16().flat_map(u16::to_be_bytes).collect()))
        .collect();
    let count = u16::try_from(strings.len()).unwrap();
    let mut table = Vec::new();
    table.extend_from_slice(&0_u16.to_be_bytes());
    table.extend_from_slice(&count.to_be_bytes());
    table.extend_from_slice(&(6 + 12 * count).to_be_bytes());
    let mut storage = Vec::new();
    for (id, text) in &strings {
        for value in [3_u16, 1, 0x0409, *id] {
            table.extend_from_slice(&value.to_be_bytes());
        }
        table.extend_from_slice(&u16::try_from(text.len()).unwrap().to_be_bytes());
        table.extend_from_slice(&u16::try_from(storage.len()).unwrap().to_be_bytes());
        storage.extend_from_slice(text);
    }
    table.extend(storage);
    table
}

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
    let font = minimal_ttf(FAMILY);
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
