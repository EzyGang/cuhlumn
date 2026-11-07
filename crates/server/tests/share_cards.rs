//! Share cards: PNG output, caching, drafts, the logo, the card styles, and the meta switch.
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
use resvg::tiny_skia::Pixmap;
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

const CARD_SIZE: (u32, u32) = (1200, 630);

fn decode(body: &[u8]) -> Pixmap {
    let pixmap = Pixmap::decode_png(body).unwrap();
    assert_eq!((pixmap.width(), pixmap.height()), CARD_SIZE);
    pixmap
}

fn color(pixmap: &Pixmap, x: u32, y: u32) -> [u8; 3] {
    let pixel = pixmap.pixel(x, y).unwrap();
    [pixel.red(), pixel.green(), pixel.blue()]
}

/// Checks a pixel colour with room for one step of rounding in each channel.
fn assert_close(actual: [u8; 3], expected: [u8; 3], style: &str) {
    let near = actual
        .iter()
        .zip(expected)
        .all(|(actual, expected)| actual.abs_diff(expected) <= 1);
    assert!(near, "{style}: {actual:?} is not {expected:?}");
}

/// Counts the pixels right of the host in the bottom row that differ from the ground.
fn meta_pixels(pixmap: &Pixmap) -> usize {
    let ground = color(pixmap, 60, 610);
    (548..584)
        .flat_map(|y| (760..1104).map(move |x| (x, y)))
        .filter(|&(x, y)| color(pixmap, x, y) != ground)
        .count()
}

#[tokio::test]
async fn each_card_style_paints_its_ground() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let accent = [0x0e, 0x74, 0x90];
    for (style, ground, band) in [
        ("dark", [17, 19, 24], accent),
        ("light", [248, 248, 245], accent),
        ("accent", accent, [57, 141, 164]),
    ] {
        let mut site = blog_site();
        site["brand"]["card_style"] = json!(style);
        assert_eq!(app.put_site(site).await.status, StatusCode::OK);
        let card = app.get("/og/sqlite-backups.png").await;
        assert!(
            is_card(&card.body),
            "{style} card is {} bytes",
            card.body.len()
        );
        let pixmap = decode(&card.body);
        assert_close(color(&pixmap, 60, 610), ground, style);
        assert_close(color(&pixmap, 10, 300), band, style);
        assert!(meta_pixels(&pixmap) > 100, "{style} shows the meta line");
    }
}

#[tokio::test]
async fn the_meta_switch_removes_the_meta_line() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let with_meta = app.get("/og/sqlite-backups.png").await.body;
    let mut site = blog_site();
    site["brand"]["card_meta"] = json!(false);
    assert_eq!(app.put_site(site).await.status, StatusCode::OK);
    let without_meta = app.get("/og/sqlite-backups.png").await.body;
    assert!(is_card(&without_meta));
    assert_ne!(without_meta, with_meta);
    assert!(meta_pixels(&decode(&with_meta)) > 100);
    assert_eq!(meta_pixels(&decode(&without_meta)), 0);
}
