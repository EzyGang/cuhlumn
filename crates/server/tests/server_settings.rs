//! First launch setup, server settings, token rotation, and page cache expiry.
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

use axum::{body::Body, http::StatusCode};
use cuhlumn_api::{settings::Scheme, site::host::Host};
use cuhlumn_server::{db::unix_now, web::cache::cached::Cached};
use serde_json::{Value, json};

use crate::common::{ADMIN_HOST, CONSOLE, HOST, Res, Setup, TOKEN, TestApp, json_body, request};

const LOCAL: &str = "localhost";
const LOCAL_ORIGIN: &str = "http://localhost:5173";
const NEW_TOKEN: &str = "harbor-rotated-token-0123456789-abcdef";

async fn health(app: &TestApp) -> Value {
    let res = app
        .send(
            request("GET", ADMIN_HOST, "/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    res.json()
}

async fn setup(app: &TestApp, token: &str) -> Res {
    let builder = request("POST", LOCAL, "/api/setup").header("origin", LOCAL_ORIGIN);
    app.send(json_body(builder, Some(json!({"token": token}))))
        .await
}

fn session_cookie(res: &Res) -> String {
    let set_cookie = res.header("set-cookie");
    assert!(set_cookie.starts_with("cuhlumn_session="), "{set_cookie}");
    set_cookie.split(';').next().unwrap().to_owned()
}

async fn bearer(app: &TestApp, token: &str, method: &str, path: &str, body: Option<Value>) -> Res {
    let builder =
        request(method, ADMIN_HOST, path).header("authorization", format!("Bearer {token}"));
    app.send(json_body(builder, body)).await
}

async fn with_cookie(
    app: &TestApp,
    cookie: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> Res {
    let builder = request(method, LOCAL, path)
        .header("cookie", cookie)
        .header("origin", LOCAL_ORIGIN);
    app.send(json_body(builder, body)).await
}

/// A fresh server with an environment token, so the admin API is on.
async fn with_env_token() -> TestApp {
    TestApp::open(Setup {
        token: Some(TOKEN),
        ..Setup::fresh()
    })
    .await
}

#[tokio::test]
async fn first_setup_wins_and_health_flips() {
    let app = TestApp::open(Setup::fresh()).await;
    assert_eq!(health(&app).await["needs_setup"], true);
    let short = setup(&app, "short").await;
    short.assert_error(StatusCode::BAD_REQUEST, "validation");
    let first = setup(&app, TOKEN).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text());
    assert!(first.json()["expires_at"].is_i64());
    let cookie = session_cookie(&first);
    assert!(first.header("set-cookie").contains("Secure"));
    assert_eq!(health(&app).await["needs_setup"], false);
    setup(&app, NEW_TOKEN)
        .await
        .assert_error(StatusCode::CONFLICT, "conflict");
    let sites = with_cookie(&app, &cookie, "GET", "/api/sites", None).await;
    assert_eq!(sites.status, StatusCode::OK, "{}", sites.text());
    let res = bearer(&app, TOKEN, "GET", "/api/sites", None).await;
    assert_eq!(res.status, StatusCode::OK);
    bearer(&app, NEW_TOKEN, "GET", "/api/sites", None)
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn setup_on_another_host_sets_no_cookie() {
    let app = TestApp::open(Setup {
        console_host: Some(CONSOLE),
        ..Setup::fresh()
    })
    .await;
    let builder = request("POST", ADMIN_HOST, "/api/setup");
    let res = app
        .send(json_body(builder, Some(json!({"token": TOKEN}))))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(res.header("set-cookie"), "");
}

#[tokio::test]
async fn admin_routes_need_setup_first() {
    let app = TestApp::open(Setup::fresh()).await;
    for (method, path) in [
        ("GET", "/api/sites"),
        ("GET", "/api/settings"),
        ("PUT", "/api/settings/token"),
        ("GET", "/api/fonts"),
        ("DELETE", "/api/fonts/Inter-1a2b3c4d.ttf"),
    ] {
        bearer(&app, TOKEN, method, path, None)
            .await
            .assert_error(StatusCode::SERVICE_UNAVAILABLE, "needs_setup");
    }
}

#[tokio::test]
async fn environment_values_lock_their_fields() {
    let app = TestApp::new().await;
    setup(&app, NEW_TOKEN)
        .await
        .assert_error(StatusCode::CONFLICT, "conflict");
    let settings = app.api("GET", "/api/settings", None).await.json();
    assert_eq!(
        settings,
        json!({"console_host": CONSOLE, "scheme": "https", "admin_listener": false,
               "locked": {"token": true, "console_host": true, "scheme": true}})
    );
    let rotate = app
        .api(
            "PUT",
            "/api/settings/token",
            Some(json!({"token": NEW_TOKEN})),
        )
        .await;
    rotate.assert_error(StatusCode::CONFLICT, "locked");
    assert_eq!(rotate.field(), "token");
    let scheme = app
        .api(
            "PUT",
            "/api/settings",
            Some(json!({"console_host": CONSOLE, "scheme": "http"})),
        )
        .await;
    scheme.assert_error(StatusCode::CONFLICT, "locked");
    assert_eq!(scheme.field(), "scheme");
    let host = app
        .api(
            "PUT",
            "/api/settings",
            Some(json!({"console_host": null, "scheme": "https"})),
        )
        .await;
    host.assert_error(StatusCode::CONFLICT, "locked");
    assert_eq!(host.field(), "console_host");
    let same = app
        .api(
            "PUT",
            "/api/settings",
            Some(json!({"console_host": CONSOLE, "scheme": "https"})),
        )
        .await;
    assert_eq!(same.status, StatusCode::OK, "{}", same.text());
}

#[tokio::test]
async fn rotation_ends_old_sessions() {
    let app = TestApp::open(Setup::fresh()).await;
    let old = session_cookie(&setup(&app, TOKEN).await);
    let rotated = with_cookie(
        &app,
        &old,
        "PUT",
        "/api/settings/token",
        Some(json!({"token": NEW_TOKEN})),
    )
    .await;
    assert_eq!(rotated.status, StatusCode::OK, "{}", rotated.text());
    let new = session_cookie(&rotated);
    with_cookie(&app, &old, "GET", "/api/sites", None)
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
    let res = with_cookie(&app, &new, "GET", "/api/sites", None).await;
    assert_eq!(res.status, StatusCode::OK);
    bearer(&app, TOKEN, "GET", "/api/sites", None)
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
    let res = bearer(&app, NEW_TOKEN, "GET", "/api/sites", None).await;
    assert_eq!(res.status, StatusCode::OK);
    let reopened = app.reopen_with(&Setup::fresh()).await;
    assert!(reopened.settings().token().unwrap().matches(NEW_TOKEN));
}

#[tokio::test]
async fn scheme_change_rebuilds_page_addresses() {
    let app = with_env_token().await;
    app.seed_blog().await;
    let page = app.get("/").await.text();
    assert!(
        page.contains("https://blog.example.com/sqlite-backups/"),
        "{page}"
    );
    let res = app
        .api(
            "PUT",
            "/api/settings",
            Some(json!({"console_host": null, "scheme": "http"})),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(res.json()["scheme"], "http");
    let page = app.get("/").await.text();
    assert!(
        page.contains("http://blog.example.com/sqlite-backups/"),
        "{page}"
    );
    assert!(
        !page.contains("https://blog.example.com/sqlite-backups/"),
        "{page}"
    );
    let post = app
        .api(
            "GET",
            &format!("/api/sites/{HOST}/posts/sqlite-backups"),
            None,
        )
        .await;
    assert_eq!(
        post.json()["url"],
        "http://blog.example.com/sqlite-backups/"
    );
}

#[tokio::test]
async fn settings_survive_a_restart() {
    let app = with_env_token().await;
    let input = json!({"console_host": "admin.harbor.example.com", "scheme": "http"});
    let res = app.api("PUT", "/api/settings", Some(input)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let reopened = app
        .reopen_with(&Setup {
            token: Some(TOKEN),
            ..Setup::fresh()
        })
        .await;
    let settings = reopened.settings();
    assert_eq!(settings.scheme(), Scheme::Http);
    assert_eq!(
        settings.console_host().map(Host::as_str),
        Some("admin.harbor.example.com")
    );
    let overridden = app
        .reopen_with(&Setup {
            token: Some(TOKEN),
            scheme: Some(Scheme::Https),
            ..Setup::fresh()
        })
        .await
        .settings();
    assert_eq!(overridden.scheme(), Scheme::Https);
    assert!(overridden.locked().scheme);
    assert!(!overridden.locked().console_host);
    assert_eq!(
        overridden.console_host().map(Host::as_str),
        Some("admin.harbor.example.com")
    );
}

#[tokio::test]
async fn cached_pages_expire_at_valid_until() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    assert!(app.get("/").await.text().contains("How we back up SQLite"));
    let host = Host::new(HOST).unwrap();
    let cache = app.state.cache();
    let stale = Cached::new("text/html; charset=utf-8", "stale", 60).valid_until(Some(1));
    cache.insert(
        &host,
        "/".to_owned(),
        cache.generation(&host),
        Arc::new(stale),
    );
    let fresh = app.get("/").await.text();
    assert!(fresh.contains("How we back up SQLite"), "{fresh}");
    let now = unix_now();
    let mut site = common::blog_site();
    site["announcement"] = json!({"text": "Harbor maintenance tonight", "starts_at": now + 3600});
    assert_eq!(app.put_site(site).await.status, StatusCode::OK);
    assert_eq!(app.get("/").await.status, StatusCode::OK);
    assert!(cache.get(&host, "/", now).is_some());
    assert!(cache.get(&host, "/", now + 3600).is_none());
}
