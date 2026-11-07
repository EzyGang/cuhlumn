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
use serde_json::json;

use crate::common::{ADMIN_HOST, CONSOLE, HOST, Res, Setup, TOKEN, TestApp, json_body, request};

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
async fn console_loads_before_the_first_setup() {
    let app = TestApp::without_token().await;
    let res = get(&app, "/").await;
    let expected = match console::is_built() {
        true => StatusCode::OK,
        false => StatusCode::SERVICE_UNAVAILABLE,
    };
    assert_eq!(res.status, expected);
    assert!(!res.text().contains("CUHLUMN_TOKEN"), "{}", res.text());
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

/// Returns a fresh server with a token but without a console host, and the site `HOST`.
async fn bootstrap() -> TestApp {
    let app = TestApp::open(Setup {
        token: Some(TOKEN),
        ..Setup::fresh()
    })
    .await;
    app.create_site().await;
    app
}

#[tokio::test]
async fn without_a_console_host_other_hosts_get_the_console() {
    let app = bootstrap().await;
    let console = app
        .send(
            request("GET", "localhost", "/")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(console.header("content-security-policy"), CSP);
    for host in [HOST, "www.blog.example.com"] {
        let blog = app
            .send(request("GET", host, "/").body(Body::empty()).unwrap())
            .await;
        assert_eq!(blog.header("content-security-policy"), "", "{host}");
        assert!(!blog.text().contains("<div id=\"root\">"), "{host}");
    }
}

#[tokio::test]
async fn without_a_console_host_cookies_work_only_on_console_hosts() {
    let app = bootstrap().await;
    let login = json_body(
        request("POST", "localhost", "/api/session").header("origin", "http://localhost:5173"),
        Some(json!({"token": TOKEN})),
    );
    let res = app.send(login).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let cookie = res
        .header("set-cookie")
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let on = |host: &str| {
        request("GET", host, "/api/sites")
            .header("cookie", &cookie)
            .body(Body::empty())
            .unwrap()
    };
    assert_eq!(app.send(on("localhost")).await.status, StatusCode::OK);
    app.send(on(HOST))
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
    let foreign = request("DELETE", "localhost", &format!("/api/sites/{HOST}"))
        .header("cookie", &cookie)
        .header("origin", "https://evil.example.org")
        .body(Body::empty())
        .unwrap();
    app.send(foreign)
        .await
        .assert_error(StatusCode::FORBIDDEN, "forbidden");
    let session = request("GET", HOST, "/api/session")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();
    app.send(session)
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
}

#[tokio::test]
async fn console_host_cannot_be_a_site() {
    let app = bootstrap().await;
    let taken = json!({"console_host": "www.blog.example.com", "scheme": "https"});
    let res = app.api("PUT", "/api/settings", Some(taken)).await;
    res.assert_error(StatusCode::CONFLICT, "conflict");
    assert_eq!(res.field(), "console_host");
    let free = json!({"console_host": "admin.harbor.example.com", "scheme": "https"});
    let res = app.api("PUT", "/api/settings", Some(free)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let site = json!({"title": "Admin", "aliases": ["admin.harbor.example.com"]});
    let res = app
        .api("PUT", "/api/sites/harbor.example.com", Some(site))
        .await;
    res.assert_error(StatusCode::CONFLICT, "conflict");
    assert_eq!(res.field(), "aliases[0]");
    let res = app
        .api(
            "PUT",
            "/api/sites/admin.harbor.example.com",
            Some(json!({"title": "x"})),
        )
        .await;
    res.assert_error(StatusCode::CONFLICT, "conflict");
    assert_eq!(res.field(), "host");
    let other = app
        .send(request("GET", ADMIN_HOST, "/").body(Body::empty()).unwrap())
        .await;
    assert_eq!(other.header("content-security-policy"), "");
}
