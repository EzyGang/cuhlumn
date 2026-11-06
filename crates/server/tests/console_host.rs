//! The console host: embedded files, security headers, and the API next to them.
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
use cuhlumn_server::console;

use crate::common::{CONSOLE, Res, TOKEN, TestApp, request};

const CSP: &str = "default-src 'self'; img-src 'self' https: data:; style-src 'self' 'unsafe-inline'; frame-ancestors 'none'";
const CSP_HTTP: &str = "default-src 'self'; img-src 'self' http: https: data:; style-src 'self' 'unsafe-inline'; frame-ancestors 'none'";

async fn get(app: &TestApp, path: &str) -> Res {
    app.send(request("GET", CONSOLE, path).body(Body::empty()).unwrap())
        .await
}

fn assert_security_headers(res: &Res) {
    assert_eq!(res.header("content-security-policy"), CSP);
    assert_eq!(res.header("x-content-type-options"), "nosniff");
    assert_eq!(res.header("referrer-policy"), "same-origin");
}

#[tokio::test]
async fn console_paths_get_files_or_the_missing_build_notice() {
    let app = TestApp::new().await;
    let root = get(&app, "/").await;
    assert_security_headers(&root);
    if !console::is_built() {
        assert_eq!(root.status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(root.text().contains("pnpm build"), "{}", root.text());
        return;
    }
    assert_eq!(root.status, StatusCode::OK);
    assert!(root.header("content-type").starts_with("text/html"));
    assert_eq!(root.header("cache-control"), "no-cache");
    let route = get(&app, "/sites/blog.example.com/posts").await;
    assert_eq!(
        (route.status, route.body.clone()),
        (StatusCode::OK, root.body.clone())
    );
    assert_security_headers(&route);
    assert_eq!(
        get(&app, "/assets/missing-file.js").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&app, "/../Cargo.toml").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn dotted_client_routes_get_the_index() {
    if !console::is_built() {
        return;
    }
    let app = TestApp::new().await;
    let index = get(&app, "/").await;
    let route = get(&app, "/sites/blog.example.com").await;
    assert_eq!(route.header("cache-control"), "no-cache");
    assert_eq!((route.status, route.body), (StatusCode::OK, index.body));
}

#[tokio::test]
async fn http_scheme_allows_http_images() {
    let app = TestApp::http().await;
    let res = get(&app, "/").await;
    assert_eq!(res.header("content-security-policy"), CSP_HTTP);
}

#[tokio::test]
async fn hashed_assets_are_immutable() {
    if !console::is_built() {
        return;
    }
    let app = TestApp::new().await;
    let index = get(&app, "/").await.text();
    let Some(start) = index.find("/assets/") else {
        return;
    };
    let path: String = index[start..].chars().take_while(|c| *c != '"').collect();
    let asset = get(&app, &path).await;
    assert_eq!(asset.status, StatusCode::OK, "{path}");
    assert_eq!(
        asset.header("cache-control"),
        "public, max-age=31536000, immutable"
    );
    assert_security_headers(&asset);
}

#[tokio::test]
async fn console_is_off_without_a_token() {
    let app = TestApp::without_token().await;
    let res = get(&app, "/").await;
    assert_eq!(res.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_security_headers(&res);
}

#[tokio::test]
async fn console_host_serves_the_api() {
    let app = TestApp::new().await;
    assert_eq!(get(&app, "/api/health").await.status, StatusCode::OK);
    get(&app, "/api/unknown")
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    let bearer =
        request("GET", CONSOLE, "/api/sites").header("authorization", format!("Bearer {TOKEN}"));
    assert_eq!(
        app.send(bearer.body(Body::empty()).unwrap()).await.status,
        StatusCode::OK
    );
    let post = app
        .send(request("POST", CONSOLE, "/").body(Body::empty()).unwrap())
        .await;
    assert_eq!(post.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_security_headers(&post);
}

#[tokio::test]
async fn other_hosts_never_get_the_console() {
    let app = TestApp::new().await;
    let res = app
        .send(
            request("GET", "nobody.example.org", "/")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.header("content-security-policy"), "");
}
