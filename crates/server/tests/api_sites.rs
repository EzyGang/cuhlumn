//! Site routes, authentication, error bodies, and host dispatch.
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

use crate::common::{ADMIN_HOST, HOST, TOKEN, TestApp, json_body, request};

#[tokio::test]
async fn health_needs_no_token() {
    let app = TestApp::without_token().await;
    let res = app
        .send(
            request("GET", ADMIN_HOST, "/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(
        res.json(),
        json!({"ok": true, "version": env!("CARGO_PKG_VERSION"), "needs_setup": true})
    );
}

#[tokio::test]
async fn without_token_the_api_is_off_and_blogs_are_served() {
    let app = TestApp::without_token().await;
    app.api("GET", "/api/sites", None)
        .await
        .assert_error(StatusCode::SERVICE_UNAVAILABLE, "needs_setup");
    let login = json_body(
        request("POST", common::CONSOLE, "/api/session"),
        Some(json!({"token": TOKEN})),
    );
    app.send(login)
        .await
        .assert_error(StatusCode::SERVICE_UNAVAILABLE, "needs_setup");
    let blog = app
        .send(request("GET", HOST, "/").body(Body::empty()).unwrap())
        .await;
    assert_eq!(
        (blog.status, blog.text().as_str()),
        (StatusCode::NOT_FOUND, "No blog lives at this address.")
    );
}

#[tokio::test]
async fn missing_or_wrong_token_is_unauthorized() {
    let app = TestApp::new().await;
    let none = app
        .send(
            request("GET", ADMIN_HOST, "/api/sites")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    none.assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
    let wrong =
        request("GET", ADMIN_HOST, "/api/sites").header("authorization", "Bearer wrong-token");
    app.send(wrong.body(Body::empty()).unwrap())
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn create_read_list_and_delete_a_site() {
    let app = TestApp::new().await;
    let site = app.create_site().await;
    assert_eq!(site["host"], HOST);
    assert_eq!(site["aliases"], json!(["www.blog.example.com"]));
    assert_eq!(site["brand"]["accent"], "#2563eb");
    assert_eq!(site["brand"]["credit"], true);
    assert_eq!(site["indexnow_key"].as_str().unwrap().len(), 32);
    let other = json!({"title": "Another"});
    assert_eq!(
        app.api("PUT", "/api/sites/a.example.org", Some(other))
            .await
            .status,
        StatusCode::OK
    );
    let list = app.api("GET", "/api/sites", None).await.json();
    let hosts: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|site| site["host"].as_str().unwrap())
        .collect();
    assert_eq!(hosts, ["a.example.org", HOST]);
    assert_eq!(
        app.api("GET", "/api/sites/BLOG.example.com", None)
            .await
            .json()["title"],
        "Harbor Notes"
    );
    let alias = app
        .api("GET", "/api/sites/www.blog.example.com", None)
        .await;
    alias.assert_error(StatusCode::NOT_FOUND, "not_found");
    assert_eq!(
        app.api("DELETE", &format!("/api/sites/{HOST}"), None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    app.api("GET", &format!("/api/sites/{HOST}"), None)
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    app.api("DELETE", &format!("/api/sites/{HOST}"), None)
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn validation_failures_name_their_field() {
    let app = TestApp::new().await;
    let path = format!("/api/sites/{HOST}");
    let cases = [
        (
            json!({"title": "x", "project_url": "javascript:alert(1)"}),
            "project_url",
        ),
        (
            json!({"title": "x", "brand": {"accent": "red;}body{display:none"}}),
            "brand.accent",
        ),
        (
            json!({"title": "x", "brand": {"font": "comic"}}),
            "brand.font",
        ),
        (
            json!({"title": "x", "brand": {"links": [{"label": "x", "url": "javascript:alert(1)"}]}}),
            "brand.links[0].url",
        ),
        (
            json!({"title": "x", "indexnow_key": "../x.txt"}),
            "indexnow_key",
        ),
        (
            json!({"title": "x", "brand": {"logo": "//evil.com/a.png"}}),
            "brand.logo",
        ),
        (json!({"title": "   "}), "title"),
        (json!({"description": "no title"}), "title"),
        (json!({"title": "x", "accent": "#abc"}), "accent"),
    ];
    for (body, field) in cases {
        let res = app.api("PUT", &path, Some(body)).await;
        res.assert_error(StatusCode::BAD_REQUEST, "validation");
        assert_eq!(res.field(), field, "{}", res.text());
    }
    let bad_host = app
        .api("PUT", "/api/sites/not_a_host", Some(json!({"title": "x"})))
        .await;
    bad_host.assert_error(StatusCode::BAD_REQUEST, "validation");
    assert_eq!(bad_host.field(), "host");
}

#[tokio::test]
async fn malformed_bodies_get_json_errors() {
    let app = TestApp::new().await;
    let path = format!("/api/sites/{HOST}");
    let malformed = request("PUT", ADMIN_HOST, &path)
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "application/json")
        .body(Body::from("{\"title\": "))
        .unwrap();
    let res = app.send(malformed).await;
    res.assert_error(StatusCode::BAD_REQUEST, "bad_request");
    assert_eq!(res.field(), serde_json::Value::Null);
    let no_type = request("PUT", ADMIN_HOST, &path)
        .header("authorization", format!("Bearer {TOKEN}"))
        .body(Body::from("{\"title\": \"x\"}"))
        .unwrap();
    app.send(no_type)
        .await
        .assert_error(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type");
}

#[tokio::test]
async fn unknown_paths_and_methods_get_json_errors() {
    let app = TestApp::new().await;
    app.api("GET", "/api/nothing", None)
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    app.api("GET", "/api", None)
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    app.api("POST", "/api/sites", None)
        .await
        .assert_error(StatusCode::METHOD_NOT_ALLOWED, "method_not_allowed");
    let patch = app.api("PATCH", &format!("/api/sites/{HOST}"), None).await;
    patch.assert_error(StatusCode::METHOD_NOT_ALLOWED, "method_not_allowed");
}

#[tokio::test]
async fn indexnow_key_is_kept_or_replaced() {
    let app = TestApp::new().await;
    let key = app.create_site().await["indexnow_key"].clone();
    let path = format!("/api/sites/{HOST}");
    let again = app
        .api("PUT", &path, Some(json!({"title": "Harbor Notes"})))
        .await
        .json();
    assert_eq!(again["indexnow_key"], key);
    let own = json!({"title": "Harbor Notes", "indexnow_key": "143f4ce9a39c0691d2e85e8b486880a4"});
    let replaced = app.api("PUT", &path, Some(own)).await.json();
    assert_eq!(replaced["indexnow_key"], "143f4ce9a39c0691d2e85e8b486880a4");
}

#[tokio::test]
async fn names_of_other_sites_conflict() {
    let app = TestApp::new().await;
    app.create_site().await;
    let alias_taken = json!({"title": "B", "aliases": ["www.blog.example.com"]});
    let res = app
        .api("PUT", "/api/sites/b.example.com", Some(alias_taken))
        .await;
    res.assert_error(StatusCode::CONFLICT, "conflict");
    let host_taken = json!({"title": "B", "aliases": [HOST]});
    app.api("PUT", "/api/sites/b.example.com", Some(host_taken))
        .await
        .assert_error(StatusCode::CONFLICT, "conflict");
    let alias_as_host = json!({"title": "W"});
    let res = app
        .api(
            "PUT",
            "/api/sites/www.blog.example.com",
            Some(alias_as_host),
        )
        .await;
    res.assert_error(StatusCode::CONFLICT, "conflict");
    let own = json!({"title": "Harbor Notes", "aliases": [HOST, "www.blog.example.com", "WWW.blog.example.com"]});
    let site = app
        .api("PUT", &format!("/api/sites/{HOST}"), Some(own))
        .await
        .json();
    assert_eq!(site["aliases"], json!(["www.blog.example.com"]));
}

#[tokio::test]
async fn home_must_be_a_page_of_the_site() {
    let app = TestApp::new().await;
    app.create_site().await;
    let path = format!("/api/sites/{HOST}");
    let missing = app
        .api("PUT", &path, Some(json!({"title": "x", "home": "about"})))
        .await;
    missing.assert_error(StatusCode::BAD_REQUEST, "validation");
    assert_eq!(missing.field(), "home");
    let post = json!({"title": "Post", "body": "Text."});
    app.api("PUT", &format!("{path}/posts/news"), Some(post))
        .await;
    let not_page = app
        .api("PUT", &path, Some(json!({"title": "x", "home": "news"})))
        .await;
    not_page.assert_error(StatusCode::BAD_REQUEST, "validation");
    let page = json!({"title": "About", "body": "A short note about the team.", "kind": "page"});
    app.api("PUT", &format!("{path}/posts/about"), Some(page))
        .await;
    let ok = app
        .api("PUT", &path, Some(json!({"title": "x", "home": "about"})))
        .await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.text());
    assert_eq!(ok.json()["home"], "about");
}

#[tokio::test]
async fn blog_hosts_get_the_blog_and_the_api() {
    let app = TestApp::new().await;
    app.create_site().await;
    let home = app
        .send(request("GET", HOST, "/").body(Body::empty()).unwrap())
        .await;
    assert_eq!(home.status, StatusCode::OK);
    assert!(home.text().contains("Harbor Notes"));
    let stranger = app
        .send(
            request("GET", "nobody.example.org", "/")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(stranger.status, StatusCode::NOT_FOUND);
    let post = app
        .send(request("POST", HOST, "/").body(Body::empty()).unwrap())
        .await;
    assert_eq!(post.status, StatusCode::METHOD_NOT_ALLOWED);
    let on_blog =
        request("GET", HOST, "/api/sites").header("authorization", format!("Bearer {TOKEN}"));
    assert_eq!(
        app.send(on_blog.body(Body::empty()).unwrap()).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn site_writes_invalidate_the_page_cache() {
    let app = TestApp::new().await;
    app.create_site().await;
    app.seed_cache();
    let edit = json!({"title": "Renamed Notes"});
    let path = format!("/api/sites/{HOST}");
    assert_eq!(
        app.api("PUT", &path, Some(edit)).await.status,
        StatusCode::OK
    );
    assert!(app.cache_is_cleared());
    app.seed_cache();
    assert_eq!(
        app.api("DELETE", &path, None).await.status,
        StatusCode::NO_CONTENT
    );
    assert!(app.cache_is_cleared());
}
