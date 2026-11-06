//! Post routes: validation, publication times, home page rules, and cache invalidation.
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

use std::sync::Arc;

use axum::http::StatusCode;
use cuhlumn_api::site::host::Host;
use cuhlumn_server::web::cache::Cached;
use serde_json::json;

use crate::common::{HOST, TestApp};

fn post_path(slug: &str) -> String {
    format!("/api/sites/{HOST}/posts/{slug}")
}

#[tokio::test]
async fn published_post_gets_url_tags_and_time() {
    let app = TestApp::new().await;
    app.create_site().await;
    let post = json!({"title": "How we back up SQLite", "body": "Text.", "tags": ["Databases", "Operations", "databases"], "status": "published"});
    let res = app
        .api("PUT", &post_path("sqlite-backups"), Some(post))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let saved = res.json();
    assert_eq!(saved["url"], "https://blog.example.com/sqlite-backups/");
    assert_eq!(saved["post"]["url"], saved["url"]);
    assert_eq!(saved["post"]["tags"], json!(["Databases", "Operations"]));
    assert!(saved["post"]["published_at"].is_i64());
    assert_eq!(saved["post"]["kind"], "post");
    assert!(!app.state.indexnow().is_enabled());
}

#[tokio::test]
async fn post_validation_failures() {
    let app = TestApp::new().await;
    app.create_site().await;
    let reserved = app
        .api(
            "PUT",
            &post_path("api"),
            Some(json!({"title": "x", "body": "x"})),
        )
        .await;
    reserved.assert_error(StatusCode::BAD_REQUEST, "validation");
    assert_eq!(reserved.field(), "slug");
    let tags: Vec<String> = (0..21).map(|n| format!("tag{n}")).collect();
    let many = app
        .api(
            "PUT",
            &post_path("many"),
            Some(json!({"title": "x", "body": "x", "tags": tags})),
        )
        .await;
    many.assert_error(StatusCode::BAD_REQUEST, "validation");
    assert_eq!(many.field(), "tags");
    let cases = [
        (json!({"body": "x"}), "title"),
        (json!({"title": "x", "body": " "}), "body"),
        (json!({"title": "x", "body": "x", "kind": "note"}), "kind"),
        (
            json!({"title": "x", "body": "x", "status": "live"}),
            "status",
        ),
        (
            json!({"title": "x", "body": "x", "cover": "//evil.com/x.png"}),
            "cover",
        ),
        (
            json!({"title": "x", "body": "x", "published_at": 253_402_300_800_i64}),
            "published_at",
        ),
        (
            json!({"title": "x", "body": "x", "published_at": -1}),
            "published_at",
        ),
        (json!({"title": "x\u{1b}", "body": "x"}), "title"),
        (json!({"title": "x", "body": "x\u{0}"}), "body"),
        (
            json!({"title": "x", "body": "x", "description": "a\u{7}"}),
            "description",
        ),
    ];
    for (body, field) in cases {
        let res = app.api("PUT", &post_path("valid"), Some(body)).await;
        res.assert_error(StatusCode::BAD_REQUEST, "validation");
        assert_eq!(res.field(), field);
    }
    let no_site = app
        .api(
            "PUT",
            "/api/sites/none.example.com/posts/a",
            Some(json!({"title": "x", "body": "x"})),
        )
        .await;
    no_site.assert_error(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn first_publication_time_is_kept() {
    let app = TestApp::new().await;
    app.create_site().await;
    let path = post_path("hello");
    let draft = app
        .api("PUT", &path, Some(json!({"title": "Hi", "body": "Text."})))
        .await
        .json();
    assert_eq!(draft["post"]["status"], "draft");
    assert!(draft["post"]["published_at"].is_null());
    let published = json!({"title": "Hi", "body": "Text.", "status": "published"});
    let first =
        app.api("PUT", &path, Some(published.clone())).await.json()["post"]["published_at"].clone();
    app.state
        .db()
        .call(|db| {
            db.execute("UPDATE posts SET published_at = 1000", [])
                .map_err(cuhlumn_server::db::DbError::from)
        })
        .await
        .unwrap();
    assert!(first.is_i64());
    let edit = app.api("PUT", &path, Some(published)).await.json();
    assert_eq!(edit["post"]["published_at"], 1000);
    let unpublished = app
        .api(
            "PUT",
            &path,
            Some(json!({"title": "Hi", "body": "Text.", "status": "draft"})),
        )
        .await
        .json();
    assert_eq!(unpublished["post"]["published_at"], 1000);
    let backdated =
        json!({"title": "Hi", "body": "Text.", "status": "published", "published_at": 500});
    assert_eq!(
        app.api("PUT", &path, Some(backdated)).await.json()["post"]["published_at"],
        500
    );
}

#[tokio::test]
async fn read_list_and_delete_posts() {
    let app = TestApp::new().await;
    app.create_site().await;
    app.api(
        "PUT",
        &post_path("draft"),
        Some(json!({"title": "Draft", "body": "x"})),
    )
    .await;
    let published = json!({"title": "Live", "body": "x", "status": "published", "published_at": 1});
    app.api("PUT", &post_path("live"), Some(published)).await;
    let list = app
        .api("GET", &format!("/api/sites/{HOST}/posts"), None)
        .await
        .json();
    let slugs: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|post| post["slug"].as_str().unwrap())
        .collect();
    assert_eq!(slugs, ["draft", "live"]);
    assert_eq!(
        app.api("GET", &post_path("live"), None).await.json()["title"],
        "Live"
    );
    app.api("GET", &post_path("nothing"), None)
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    assert_eq!(
        app.api("DELETE", &post_path("live"), None).await.status,
        StatusCode::NO_CONTENT
    );
    app.api("DELETE", &post_path("live"), None)
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    app.api("GET", "/api/sites/none.example.com/posts", None)
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn home_page_cannot_be_deleted_or_rekinded() {
    let app = TestApp::new().await;
    app.create_site().await;
    let page = json!({"title": "About", "body": "A short note about the team.", "kind": "page", "status": "published"});
    app.api("PUT", &post_path("about"), Some(page.clone()))
        .await;
    let site =
        json!({"title": "Harbor Notes", "aliases": ["www.blog.example.com"], "home": "about"});
    assert_eq!(
        app.api("PUT", &format!("/api/sites/{HOST}"), Some(site))
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        app.api("GET", &post_path("about"), None).await.json()["url"],
        "https://blog.example.com/"
    );
    app.api("DELETE", &post_path("about"), None)
        .await
        .assert_error(StatusCode::CONFLICT, "conflict");
    let rekind = json!({"title": "About", "body": "A short note about the team.", "kind": "post"});
    app.api("PUT", &post_path("about"), Some(rekind))
        .await
        .assert_error(StatusCode::CONFLICT, "conflict");
    assert_eq!(
        app.api("PUT", &post_path("about"), Some(page)).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn writes_invalidate_the_page_cache() {
    let app = TestApp::new().await;
    app.create_site().await;
    let host = Host::new(HOST).unwrap();
    let cache = app.state.cache();
    let generation = cache.generation(&host);
    cache.insert(
        &host,
        "/".to_owned(),
        generation,
        Arc::new(Cached::new("text/plain", "old", 60)),
    );
    assert!(cache.get(&host, "/").is_some());
    app.api(
        "PUT",
        &post_path("new"),
        Some(json!({"title": "New", "body": "x"})),
    )
    .await;
    assert!(cache.get(&host, "/").is_none());
    assert!(cache.generation(&host) > generation);
}

#[tokio::test]
async fn post_delete_invalidates_the_page_cache() {
    let app = TestApp::new().await;
    app.create_site().await;
    app.api(
        "PUT",
        &post_path("old"),
        Some(json!({"title": "Old", "body": "x"})),
    )
    .await;
    app.seed_cache();
    let res = app.api("DELETE", &post_path("old"), None).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert!(app.cache_is_cleared());
}
