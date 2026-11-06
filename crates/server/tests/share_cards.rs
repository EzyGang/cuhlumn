//! Share cards: PNG output, caching, drafts, and the logo.
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

use crate::common::{ADMIN_HOST, HOST, TOKEN, TestApp, blog_site, request};

fn is_card(body: &[u8]) -> bool {
    body.starts_with(b"\x89PNG\r\n\x1a\n") && body.len() > 5_000
}

#[tokio::test]
async fn post_and_site_cards_are_png() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let card = app.get("/og/sqlite-backups.png").await;
    assert_eq!(card.status, StatusCode::OK);
    assert_eq!(card.header("content-type"), "image/png");
    assert_eq!(card.header("cache-control"), "public, max-age=86400");
    assert!(card.header("content-encoding").is_empty());
    assert!(is_card(&card.body), "card is {} bytes", card.body.len());
    let site = app.get("/og.png?v=1").await;
    assert!(
        is_card(&site.body),
        "site card is {} bytes",
        site.body.len()
    );
    assert_ne!(site.body, card.body);
    assert_eq!(
        app.get("/og/unreleased.png").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.get("/og/missing.png").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn logo_changes_the_card() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let plain = app.get("/og/sqlite-backups.png").await.body;
    let request = request("POST", ADMIN_HOST, &format!("/api/sites/{HOST}/media"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "image/png")
        .body(Body::from(plain.clone()))
        .unwrap();
    let saved = app.send(request).await.json();
    let mut site = blog_site();
    site["brand"]["logo"] = saved["url"].clone();
    assert_eq!(app.put_site(site).await.status, StatusCode::OK);
    let branded = app.get("/og/sqlite-backups.png").await.body;
    assert!(is_card(&branded));
    assert_ne!(branded, plain);
}

#[tokio::test]
async fn foreign_logo_draws_the_initial() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let plain = app.get("/og/sqlite-backups.png").await.body;
    let mut site = blog_site();
    site["brand"]["logo"] =
        json!("https://cdn.example.com/media/0123456789abcdef0123456789abcdef.png");
    assert_eq!(app.put_site(site).await.status, StatusCode::OK);
    assert_eq!(app.get("/og/sqlite-backups.png").await.body, plain);
}
