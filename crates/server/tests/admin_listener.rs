//! The admin listener: the public router serves only blogs, and the admin router serves the console and the API.
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
use serde_json::{Value, json};

use crate::common::{CONSOLE, HOST, Res, Setup, TOKEN, TestApp, json_body, request};

/// The `Host` header of a request to the admin listener on a NetBird address.
const ADMIN: &str = "100.92.3.4:8081";
const ADMIN_ORIGIN: &str = "http://100.92.3.4:8081";
const CSP_PREFIX: &str = "default-src 'self';";

/// An app with an admin listener and the environment overrides of [`Setup::default`].
async fn with_env_token() -> TestApp {
    TestApp::open(Setup {
        admin_listener: true,
        ..Setup::default()
    })
    .await
}

/// An app with an admin listener and no overrides, as on a first launch.
async fn fresh() -> TestApp {
    TestApp::open(Setup {
        admin_listener: true,
        ..Setup::fresh()
    })
    .await
}

async fn public_get(app: &TestApp, host: &str, path: &str) -> Res {
    let builder = request("GET", host, path).header("authorization", format!("Bearer {TOKEN}"));
    app.send_public(builder.body(Body::empty()).unwrap()).await
}

async fn admin_get(app: &TestApp, host: &str, path: &str) -> Res {
    app.send(request("GET", host, path).body(Body::empty()).unwrap())
        .await
}

fn assert_console(res: &Res) {
    assert!(
        res.header("content-security-policy")
            .starts_with(CSP_PREFIX),
        "{}",
        res.text()
    );
}

fn assert_not_console(res: &Res) {
    assert_eq!(res.header("content-security-policy"), "");
    assert_eq!(
        (res.status, res.text().as_str()),
        (StatusCode::NOT_FOUND, "No blog lives at this address.")
    );
}

/// Sends a JSON request to the admin listener with optional extra headers.
async fn admin_send(
    app: &TestApp,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<Value>,
) -> Res {
    let mut builder = request(method, ADMIN, path);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    app.send(json_body(builder, body)).await
}

async fn login(app: &TestApp, headers: &[(&str, &str)]) -> Res {
    let res = admin_send(
        app,
        "POST",
        "/api/session",
        headers,
        Some(json!({"token": TOKEN})),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res
}

fn session_cookie(res: &Res) -> String {
    let set_cookie = res.header("set-cookie");
    assert!(set_cookie.starts_with("cuhlumn_session="), "{set_cookie}");
    set_cookie.split(';').next().unwrap().to_owned()
}

#[tokio::test]
async fn public_listener_serves_blogs_and_health_only() {
    let app = with_env_token().await;
    app.create_site().await;
    public_get(&app, HOST, "/api/sites")
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    public_get(&app, HOST, "/api/settings")
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    let health = public_get(&app, HOST, "/api/health").await;
    assert_eq!(health.status, StatusCode::OK, "{}", health.text());
    assert_eq!(health.json()["ok"], true);
    let blog = public_get(&app, HOST, "/").await;
    assert_eq!(blog.status, StatusCode::OK, "{}", blog.text());
    assert!(blog.text().contains("Harbor Notes"));
    for host in [CONSOLE, "localhost", ADMIN] {
        assert_not_console(&public_get(&app, host, "/").await);
        assert_not_console(&public_get(&app, host, "/sites").await);
    }
}

#[tokio::test]
async fn public_listener_never_serves_the_console_before_setup() {
    let app = fresh().await;
    assert_eq!(
        public_get(&app, "localhost", "/api/health").await.json()["needs_setup"],
        true
    );
    assert_not_console(&public_get(&app, "localhost", "/").await);
    public_get(&app, "localhost", "/api/setup")
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn admin_listener_serves_the_console_on_every_host() {
    let app = fresh().await;
    for host in [ADMIN, "localhost", HOST, "[fd00::1]:8081"] {
        assert_console(&admin_get(&app, host, "/").await);
        assert_console(&admin_get(&app, host, "/settings/server").await);
    }
    let setup = admin_send(
        &app,
        "POST",
        "/api/setup",
        &[("origin", ADMIN_ORIGIN)],
        Some(json!({"token": TOKEN})),
    )
    .await;
    assert_eq!(setup.status, StatusCode::OK, "{}", setup.text());
    assert_console(&admin_get(&app, ADMIN, "/").await);
    assert_console(&admin_get(&app, HOST, "/").await);
}

#[tokio::test]
async fn setup_and_bearer_work_on_the_admin_listener() {
    let app = fresh().await;
    let setup = admin_send(
        &app,
        "POST",
        "/api/setup",
        &[("origin", ADMIN_ORIGIN)],
        Some(json!({"token": TOKEN})),
    )
    .await;
    assert_eq!(setup.status, StatusCode::OK, "{}", setup.text());
    let cookie = session_cookie(&setup);
    assert!(!setup.header("set-cookie").contains("Secure"));
    let sites = admin_send(&app, "GET", "/api/sites", &[("cookie", &cookie)], None).await;
    assert_eq!(sites.status, StatusCode::OK, "{}", sites.text());
    let session = admin_send(&app, "GET", "/api/session", &[("cookie", &cookie)], None).await;
    assert_eq!(session.status, StatusCode::OK, "{}", session.text());
    let bearer = format!("Bearer {TOKEN}");
    let res = admin_send(
        &app,
        "GET",
        "/api/sites",
        &[("authorization", &bearer)],
        None,
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
}

#[tokio::test]
async fn cookie_is_secure_only_behind_a_tls_proxy() {
    let app = with_env_token().await;
    let plain = login(&app, &[("origin", ADMIN_ORIGIN)]).await;
    assert!(!plain.header("set-cookie").contains("Secure"));
    for proto in ["https", "HTTPS", "https, http"] {
        let headers = [("origin", ADMIN_ORIGIN), ("x-forwarded-proto", proto)];
        let res = login(&app, &headers).await;
        assert!(res.header("set-cookie").ends_with("; Secure"), "{proto}");
    }
    let http = login(&app, &[("x-forwarded-proto", "http")]).await;
    assert!(!http.header("set-cookie").contains("Secure"));
    let cookie = session_cookie(&plain);
    let headers = [
        ("cookie", cookie.as_str()),
        ("origin", ADMIN_ORIGIN),
        ("x-forwarded-proto", "https"),
    ];
    let logout = admin_send(&app, "DELETE", "/api/session", &headers, None).await;
    assert_eq!(logout.status, StatusCode::NO_CONTENT, "{}", logout.text());
    assert!(logout.header("set-cookie").ends_with("; Secure"));
}

#[tokio::test]
async fn cookie_requests_need_the_origin_of_the_host_header() {
    let app = with_env_token().await;
    let cookie = session_cookie(&login(&app, &[]).await);
    let body = json!({"console_host": null, "scheme": "https"});
    for origin in ["https://evil.example.com", "http://100.92.3.4:9000"] {
        let headers = [("cookie", cookie.as_str()), ("origin", origin)];
        admin_send(&app, "PUT", "/api/settings", &headers, Some(body.clone()))
            .await
            .assert_error(StatusCode::FORBIDDEN, "forbidden");
    }
    let no_origin = [("cookie", cookie.as_str())];
    admin_send(&app, "PUT", "/api/settings", &no_origin, Some(body.clone()))
        .await
        .assert_error(StatusCode::FORBIDDEN, "forbidden");
    let headers = [("cookie", cookie.as_str()), ("origin", ADMIN_ORIGIN)];
    let ok = admin_send(&app, "PUT", "/api/settings", &headers, Some(body)).await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.text());
}

#[tokio::test]
async fn settings_lock_the_console_host() {
    let app = with_env_token().await;
    let settings = app.api("GET", "/api/settings", None).await;
    assert_eq!(settings.status, StatusCode::OK, "{}", settings.text());
    let body = settings.json();
    assert_eq!(body["admin_listener"], true);
    assert_eq!(body["locked"]["console_host"], true);
    assert_eq!(body["console_host"], Value::Null);
    let host = json!({"console_host": "console.harbor.example.com", "scheme": "https"});
    let res = app.api("PUT", "/api/settings", Some(host)).await;
    res.assert_error(StatusCode::CONFLICT, "locked");
    assert_eq!(res.field(), "console_host");
    let none = json!({"console_host": null, "scheme": "https"});
    let ok = app.api("PUT", "/api/settings", Some(none)).await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.text());
    assert_eq!(ok.json()["admin_listener"], true);
}

#[tokio::test]
async fn site_writes_skip_the_console_host_check() {
    let app = with_env_token().await;
    let site = json!({"title": "Harbor Notes", "aliases": [CONSOLE]});
    let res = app.put_site(site).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
}
