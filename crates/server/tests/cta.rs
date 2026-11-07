//! The call to action after posts: a site default and a choice for each post.
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

/// Stores `site` and the published post `sqlite-backups`, and returns the page of the post.
async fn page_with(site: Value, post_show_cta: Value, kind: &str) -> String {
    let app = TestApp::new().await;
    let res = app.put_site(site).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let post = json!({
        "title": "How we back up SQLite", "body": "We back up one file.",
        "status": "published", "kind": kind, "show_cta": post_show_cta,
    });
    let res = app.put_post("sqlite-backups", post).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(res.json()["post"]["show_cta"], post_show_cta);
    let page = app.get("/sqlite-backups/").await;
    assert_eq!(page.status, StatusCode::OK);
    page.text()
}

fn site_with(show_cta: Option<bool>) -> Value {
    let mut site = blog_site();
    if let Some(show_cta) = show_cta {
        site["show_cta"] = json!(show_cta);
    }
    site
}

fn has_pitch(page: &str) -> bool {
    page.contains(r#"class="pitch""#)
}

#[tokio::test]
async fn post_choice_wins_over_the_site_default() {
    let cases = [
        (None, Value::Null, true),
        (Some(true), Value::Null, true),
        (Some(false), Value::Null, false),
        (Some(false), json!(true), true),
        (Some(true), json!(false), false),
        (None, json!(false), false),
    ];
    for (site, post, expected) in cases {
        let page = page_with(site_with(site), post.clone(), "post").await;
        assert_eq!(has_pitch(&page), expected, "site {site:?}, post {post}");
        if expected {
            assert!(page.contains("Harbor runs your app on a server that you own."));
        }
    }
}

#[tokio::test]
async fn pages_follow_the_same_rule() {
    let page = page_with(site_with(Some(false)), Value::Null, "page").await;
    assert!(!has_pitch(&page));
    let page = page_with(site_with(Some(false)), json!(true), "page").await;
    assert!(has_pitch(&page));
}

#[tokio::test]
async fn no_pitch_without_a_project() {
    let mut site = site_with(Some(true));
    site["project_url"] = Value::Null;
    let page = page_with(site.clone(), Value::Null, "post").await;
    assert!(!has_pitch(&page));
    let page = page_with(site, json!(true), "post").await;
    assert!(!has_pitch(&page));
}

#[tokio::test]
async fn site_returns_show_cta() {
    let app = TestApp::new().await;
    let res = app.put_site(blog_site()).await;
    assert_eq!(res.json()["show_cta"], json!(true));
    let res = app.put_site(site_with(Some(false))).await;
    assert_eq!(res.json()["show_cta"], json!(false));
}
