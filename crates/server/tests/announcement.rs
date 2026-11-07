//! The announcement bar: shown on HTML pages while it is live, never in feeds, indexes, or Markdown copies.
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
use serde_json::{Value, json};

use crate::common::{TestApp, blog_site};

const TEXT: &str = "Harbor moves its backups on November 20.";
/// The first 8 hex digits of SHA-256 of [`TEXT`], a line feed, and `/sqlite-backups/`.
const KEY: &str = "912d6b7b";

/// Seeds the blog with `announcement`.
async fn seed_with(announcement: Value) -> TestApp {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let mut site = blog_site();
    site["announcement"] = announcement;
    let res = app.put_site(site).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    app
}

fn linked() -> Value {
    json!({"text": TEXT, "link": "/sqlite-backups/", "link_label": "Read the plan", "tone": "warning"})
}

fn has_bar(page: &str) -> bool {
    page.contains(r#"<div class="announcement"#)
}

#[tokio::test]
async fn live_bar_is_first_in_every_html_page() {
    let app = seed_with(linked()).await;
    for path in [
        "/",
        "/sqlite-backups/",
        "/about/",
        "/tags/databases/",
        "/nothing-here/",
    ] {
        let res = app.get(path).await;
        let html = res.text();
        let body = &html[html.find("<body>").unwrap() + "<body>".len()..];
        let bar = format!(
            r#"<div class="announcement announcement-warning" role="region" aria-label="Announcement" data-announcement="{KEY}"><p class="announcement-text">{TEXT} <a href="/sqlite-backups/">Read the plan<span class="announcement-context"> about this announcement</span></a></p><button class="announcement-close" type="button" aria-label="Dismiss the announcement"><svg"#
        );
        assert!(body.starts_with(&bar), "{path}:\n{body}");
        let head = &html[..html.find("</head>").unwrap()];
        let hide = format!(r#"localStorage.getItem("announcement:{KEY}")"#);
        assert!(head.contains(&hide), "{path} lacks the pre-paint script");
        assert!(
            body.contains(r#"localStorage.setItem("announcement:"+"#),
            "{path}"
        );
    }
}

#[tokio::test]
async fn fixed_bar_without_link_has_no_button_or_scripts() {
    let app = seed_with(json!({"text": TEXT, "dismissible": false})).await;
    let html = app.get("/").await.text();
    let bar = &html[html.find(r#"<div class="announcement"#).unwrap()..];
    let bar = &bar[..bar.find("</div>").unwrap()];
    assert!(
        bar.starts_with(r#"<div class="announcement announcement-info" "#),
        "{bar}"
    );
    assert!(
        bar.contains(&format!(r#"<p class="announcement-text">{TEXT}</p>"#)),
        "{bar}"
    );
    assert!(!bar.contains("<a ") && !bar.contains("<button"), "{bar}");
    assert!(!html.contains("localStorage.getItem(\"announcement:"));
    assert!(!html.contains("localStorage.setItem(\"announcement:"));
}

#[tokio::test]
async fn scheduled_and_ended_bars_are_hidden() {
    for window in [
        json!({"starts_at": 4_102_444_800_i64}),
        json!({"starts_at": 1_000, "ends_at": 2_000}),
    ] {
        let mut announcement = linked();
        for (key, value) in window.as_object().unwrap() {
            announcement[key] = value.clone();
        }
        let app = seed_with(announcement).await;
        for path in ["/", "/sqlite-backups/"] {
            let html = app.get(path).await.text();
            assert!(!has_bar(&html), "{window} {path}");
            assert!(!html.contains(TEXT), "{window} {path}");
            assert!(!html.contains("announcement:"), "{window} {path}");
        }
    }
}

#[tokio::test]
async fn bar_stays_out_of_feeds_indexes_and_copies() {
    let app = seed_with(linked()).await;
    assert!(has_bar(&app.get("/").await.text()));
    for path in [
        "/feed.xml",
        "/sitemap.xml",
        "/llms.txt",
        "/llms-full.txt",
        "/sqlite-backups.md",
        "/robots.txt",
    ] {
        let res = app.get(path).await;
        assert_eq!(res.status, StatusCode::OK, "{path}");
        let text = res.text();
        assert!(
            !text.contains(TEXT) && !text.contains("announcement"),
            "{path}:\n{text}"
        );
    }
}
