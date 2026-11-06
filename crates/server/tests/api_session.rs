//! Console sessions: login, cookie access, the Origin check, logout, and expiry.
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
use cuhlumn_server::{
    config::Token,
    db::{DbError, sessions, unix_now},
};
use serde_json::json;
use tower::ServiceExt;

use crate::common::{CONSOLE, HOST, Res, TOKEN, TestApp, json_body, request};

const ORIGIN: &str = "https://console.test";

async fn login(app: &TestApp, host: &str, token: &str) -> Res {
    let builder = request("POST", host, "/api/session").header("origin", ORIGIN);
    app.send(json_body(builder, Some(json!({"token": token}))))
        .await
}

async fn cookie(app: &TestApp) -> String {
    let res = login(app, CONSOLE, TOKEN).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.header("set-cookie")
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

async fn with_cookie(
    app: &TestApp,
    method: &str,
    host: &str,
    path: &str,
    cookie: &str,
    origin: Option<&str>,
) -> Res {
    let mut builder = request(method, host, path).header("cookie", cookie);
    if let Some(origin) = origin {
        builder = builder.header("origin", origin);
    }
    app.send(builder.body(Body::empty()).unwrap()).await
}

#[tokio::test]
async fn login_sets_a_strict_cookie() {
    let app = TestApp::new().await;
    let res = login(&app, CONSOLE, TOKEN).await;
    assert_eq!(res.status, StatusCode::OK);
    let set_cookie = res.header("set-cookie");
    assert!(set_cookie.starts_with("cuhlumn_session="), "{set_cookie}");
    for attribute in [
        "HttpOnly",
        "SameSite=Strict",
        "Path=/",
        "Max-Age=604800",
        "Secure",
    ] {
        assert!(
            set_cookie.contains(attribute),
            "{set_cookie} lacks {attribute}"
        );
    }
    let id = set_cookie
        .split(';')
        .next()
        .unwrap()
        .trim_start_matches("cuhlumn_session=");
    assert_eq!(id.len(), 64);
    let expires_at = res.json()["expires_at"].as_i64().unwrap();
    assert!((expires_at - unix_now() - 604_800).abs() < 5);
}

#[tokio::test]
async fn login_failures() {
    let app = TestApp::new().await;
    login(&app, CONSOLE, "wrong-token")
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
    login(&app, HOST, TOKEN)
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    let foreign =
        request("POST", CONSOLE, "/api/session").header("origin", "https://evil.example.com");
    let res = app
        .send(json_body(foreign, Some(json!({"token": TOKEN}))))
        .await;
    res.assert_error(StatusCode::FORBIDDEN, "forbidden");
}

#[tokio::test]
async fn cookie_works_only_on_the_console_host() {
    let app = TestApp::new().await;
    let cookie = cookie(&app).await;
    let check = with_cookie(&app, "GET", CONSOLE, "/api/session", &cookie, None).await;
    assert_eq!(check.status, StatusCode::OK);
    assert!(check.json()["expires_at"].is_i64());
    assert_eq!(
        with_cookie(&app, "GET", CONSOLE, "/api/sites", &cookie, None)
            .await
            .status,
        StatusCode::OK
    );
    let blog = with_cookie(&app, "GET", HOST, "/api/sites", &cookie, None).await;
    blog.assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
    let wrong_bearer = request("GET", CONSOLE, "/api/sites")
        .header("cookie", &cookie)
        .header("authorization", "Bearer wrong-token");
    app.send(wrong_bearer.body(Body::empty()).unwrap())
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn cookie_writes_need_the_console_origin() {
    let app = TestApp::new().await;
    let cookie = cookie(&app).await;
    let path = format!("/api/sites/{HOST}");
    let put = |origin: Option<&'static str>| {
        let mut builder = request("PUT", CONSOLE, &path).header("cookie", &cookie);
        if let Some(origin) = origin {
            builder = builder.header("origin", origin);
        }
        json_body(builder, Some(json!({"title": "Notes"})))
    };
    app.send(put(Some("https://evil.example.com")))
        .await
        .assert_error(StatusCode::FORBIDDEN, "forbidden");
    app.send(put(None))
        .await
        .assert_error(StatusCode::FORBIDDEN, "forbidden");
    let ok = app.send(put(Some("https://console.test:8443"))).await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.text());
}

#[tokio::test]
async fn logout_ends_the_session() {
    let app = TestApp::new().await;
    let cookie = cookie(&app).await;
    let foreign = with_cookie(
        &app,
        "DELETE",
        CONSOLE,
        "/api/session",
        &cookie,
        Some("https://evil.example.com"),
    )
    .await;
    foreign.assert_error(StatusCode::FORBIDDEN, "forbidden");
    let res = with_cookie(
        &app,
        "DELETE",
        CONSOLE,
        "/api/session",
        &cookie,
        Some(ORIGIN),
    )
    .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert!(res.header("set-cookie").contains("Max-Age=0"));
    with_cookie(&app, "GET", CONSOLE, "/api/session", &cookie, None)
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
    with_cookie(&app, "GET", CONSOLE, "/api/sites", &cookie, None)
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn expired_session_is_unauthorized() {
    let app = TestApp::new().await;
    let token = Token::new(TOKEN).unwrap();
    let (old, new) = ("a".repeat(64), "b".repeat(64));
    let (old_hash, new_hash) = (token.session_hash(&old), token.session_hash(&new));
    app.state
        .db()
        .call(move |db| {
            sessions::create(db, &old_hash, unix_now() - 1)?;
            sessions::create(db, &new_hash, unix_now() + 60)
        })
        .await
        .map_err(|err: DbError| err)
        .unwrap();
    let valid = format!("cuhlumn_session={new}");
    let res = with_cookie(&app, "GET", CONSOLE, "/api/sites", &valid, None).await;
    assert_eq!(res.status, StatusCode::OK);
    let id = old;
    let cookie = format!("cuhlumn_session={id}");
    with_cookie(&app, "GET", CONSOLE, "/api/session", &cookie, None)
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
    with_cookie(&app, "GET", CONSOLE, "/api/sites", &cookie, None)
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
    let check = request("GET", CONSOLE, "/api/session")
        .body(Body::empty())
        .unwrap();
    app.send(check)
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn a_new_token_ends_old_sessions() {
    let app = TestApp::new().await;
    let cookie = cookie(&app).await;
    let rotated = app.reopen("another-token-0123456789-abcdefghijk").await;
    let request = request("GET", CONSOLE, "/api/sites")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    let res = rotated.oneshot(request).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let same = with_cookie(&app, "GET", CONSOLE, "/api/sites", &cookie, None).await;
    assert_eq!(same.status, StatusCode::OK);
}

#[tokio::test]
async fn other_authorization_schemes_fall_through_to_the_cookie() {
    let app = TestApp::new().await;
    let cookie = cookie(&app).await;
    let send = |authorization: &'static str| {
        request("GET", CONSOLE, "/api/sites")
            .header("cookie", &cookie)
            .header("authorization", authorization)
            .body(Body::empty())
            .unwrap()
    };
    assert_eq!(
        app.send(send("Basic dXNlcjpwYXNz")).await.status,
        StatusCode::OK
    );
    app.send(send("Bearer"))
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
    let basic_only = request("GET", CONSOLE, "/api/sites")
        .header("authorization", "Basic dXNlcjpwYXNz")
        .body(Body::empty())
        .unwrap();
    app.send(basic_only)
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
}
