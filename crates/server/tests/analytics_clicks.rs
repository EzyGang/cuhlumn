//! Link-click analytics: outbound links and downloads of external sites and of blogs.
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

use crate::common::{
    HOST, TestApp,
    beacon::{DOCS, DOCS_ORIGIN, FIREFOX, GOOGLEBOT, assert_empty, docs_click, docs_view},
    request,
};

/// The limit of distinct link targets of one site and day.
const TARGET_LIMIT: usize = 1_000;

/// Returns the parts of a report that page views fill.
fn view_counts(report: &Value) -> Value {
    json!([
        report["views"],
        report["visitors"],
        report["days"],
        report["pages"],
        report["referrers"],
        report["devices"],
        report["browsers"],
    ])
}

/// Sends a Firefox click beacon of a blog page to `host`, without an `Origin` header.
async fn blog_click(app: &TestApp, host: &str, kind: &str, target: &str) {
    let body = json!({"click": kind, "target": target});
    let request = request("POST", host, "/_hit")
        .header("user-agent", FIREFOX)
        .body(Body::from(body.to_string()))
        .unwrap();
    assert_empty(&app.send_public(request).await, &body);
}

#[tokio::test]
async fn outbound_and_download_clicks_count_under_their_kind() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let clicks = [
        ("outbound", "github.com/EzyGang/cuhlumn"),
        ("outbound", "pypi.org/project/py-cachify/"),
        ("outbound", "github.com/EzyGang/cuhlumn"),
        ("download", "docs.example.org/files/harbor.zip"),
        (
            "download",
            "github.com/EzyGang/cuhlumn/releases/download/v1/cuhlumn.tar.gz",
        ),
        ("download", "docs.example.org/files/harbor.zip"),
        ("download", "docs.example.org/files/harbor.zip"),
        ("download", "github.com/EzyGang/cuhlumn"),
    ];
    for (kind, target) in clicks {
        app.beacon_from(DOCS_ORIGIN, docs_click(kind, target)).await;
    }
    let report = app.analytics(DOCS).await;
    assert_eq!(
        report["outbound"],
        json!([
            {"target": "github.com/EzyGang/cuhlumn", "clicks": 2},
            {"target": "pypi.org/project/py-cachify/", "clicks": 1},
        ])
    );
    assert_eq!(
        report["downloads"],
        json!([
            {"target": "docs.example.org/files/harbor.zip", "clicks": 3},
            {"target": "github.com/EzyGang/cuhlumn", "clicks": 1},
            {"target": "github.com/EzyGang/cuhlumn/releases/download/v1/cuhlumn.tar.gz", "clicks": 1},
        ])
    );
    let blog = app.analytics(HOST).await;
    assert_eq!(blog["outbound"], json!([]));
    assert_eq!(blog["downloads"], json!([]));
}

#[tokio::test]
async fn clicks_change_no_view_or_visitor_count() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let empty = app.analytics(DOCS).await;
    app.beacon_from(DOCS_ORIGIN, docs_click("outbound", "github.com/EzyGang"))
        .await;
    let clicked = app.analytics(DOCS).await;
    assert_eq!(view_counts(&clicked), view_counts(&empty));
    assert_eq!(clicked["views"], 0);
    for table in ["analytics_days", "analytics_visitors", "analytics_pages"] {
        assert_eq!(app.analytics_rows(table, DOCS).await, 0, "{table}");
    }
    app.beacon_from(DOCS_ORIGIN, docs_view("/guide/", "")).await;
    let viewed = app.analytics(DOCS).await;
    assert_eq!(
        (&viewed["views"], &viewed["visitors"]),
        (&json!(1), &json!(1))
    );
    for kind in ["outbound", "download"] {
        app.beacon_from(DOCS_ORIGIN, docs_click(kind, "github.com/EzyGang"))
            .await;
    }
    let report = app.analytics(DOCS).await;
    assert_eq!(view_counts(&report), view_counts(&viewed));
    assert_eq!(
        report["outbound"],
        json!([{"target": "github.com/EzyGang", "clicks": 2}])
    );
    assert_eq!(
        report["downloads"],
        json!([{"target": "github.com/EzyGang", "clicks": 1}])
    );
}

#[tokio::test]
async fn clicks_of_other_origins_and_of_bots_do_not_count() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let click = docs_click("outbound", "github.com/EzyGang/cuhlumn");
    for origin in [
        "https://evil.example.net",
        "https://blog.example.com",
        "null",
    ] {
        app.beacon_from(origin, click.clone()).await;
    }
    assert_empty(
        &app.beacon(&[("user-agent", FIREFOX)], &click).await,
        &click,
    );
    let unregistered = json!({"host": "evil.example.net", "click": "download", "target": "evil.example.net/a.zip"});
    app.beacon_from("https://evil.example.net", unregistered)
        .await;
    for agent in [GOOGLEBOT, "curl/8.10.1", ""] {
        let headers = [("user-agent", agent), ("origin", DOCS_ORIGIN)];
        assert_empty(&app.beacon(&headers, &click).await, &click);
    }
    assert_empty(
        &app.beacon(&[("origin", DOCS_ORIGIN)], &click).await,
        &click,
    );
    for host in [DOCS, HOST] {
        let report = app.analytics(host).await;
        assert_eq!(report["outbound"], json!([]), "{host}");
        assert_eq!(report["downloads"], json!([]), "{host}");
    }
    assert_eq!(app.analytics_rows("analytics_clicks", DOCS).await, 0);
}

#[tokio::test]
async fn stored_targets_have_no_query_no_fragment_and_a_length_limit() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let longest = format!("github.com/{}", "a".repeat(245));
    let too_long = format!("{longest}a");
    for target in [
        "GitHub.com/EzyGang/cuhlumn?tab=readme&token=secret#install",
        "github.com/EzyGang/cuhlumn#install",
        "github.com",
        "files.example.net/Harbor%20Notes.pdf?download=1",
        longest.as_str(),
        too_long.as_str(),
        "https://github.com/EzyGang/cuhlumn",
        "github.com:8443/EzyGang",
        "/EzyGang/cuhlumn",
        "",
        "docs.example.org/guide/",
    ] {
        app.beacon_from(DOCS_ORIGIN, docs_click("outbound", target))
            .await;
    }
    let report = app.analytics(DOCS).await;
    assert_eq!(
        report["outbound"],
        json!([
            {"target": "github.com/EzyGang/cuhlumn", "clicks": 2},
            {"target": "files.example.net/Harbor Notes.pdf", "clicks": 1},
            {"target": "github.com/", "clicks": 1},
            {"target": longest, "clicks": 1},
        ])
    );
}

#[tokio::test]
async fn distinct_targets_of_one_day_have_a_limit() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let send = async |targets: std::ops::Range<usize>| {
        for index in targets {
            let click = docs_click("outbound", &format!("t{index:04}.example.net/"));
            app.beacon_from(DOCS_ORIGIN, click).await;
        }
    };
    send(0..600).await;
    app.analytics(DOCS).await;
    assert_eq!(app.analytics_rows("analytics_clicks", DOCS).await, 600);
    send(600..TARGET_LIMIT + 5).await;
    app.analytics(DOCS).await;
    assert_eq!(
        app.analytics_rows("analytics_clicks", DOCS).await,
        i64::try_from(TARGET_LIMIT).unwrap()
    );
    send(0..1).await;
    app.beacon_from(DOCS_ORIGIN, docs_click("download", "t0000.example.net/"))
        .await;
    let report = app.analytics(DOCS).await;
    assert_eq!(
        report["outbound"][0],
        json!({"target": "t0000.example.net/", "clicks": 2})
    );
    assert_eq!(report["downloads"], json!([]));
    assert_eq!(
        app.analytics_rows("analytics_clicks", DOCS).await,
        i64::try_from(TARGET_LIMIT).unwrap()
    );
}

#[tokio::test]
async fn blog_pages_count_clicks_without_a_change_of_their_views() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let home = json!({"path": "/", "referrer": ""});
    assert_empty(&app.beacon(&[("user-agent", FIREFOX)], &home).await, &home);
    let viewed = app.analytics(HOST).await;
    assert_eq!(
        (&viewed["views"], &viewed["visitors"]),
        (&json!(1), &json!(1))
    );
    blog_click(&app, HOST, "outbound", "harbor.example.com/").await;
    blog_click(&app, HOST, "outbound", "harbor.example.com/?ref=blog").await;
    blog_click(&app, HOST, "download", "blog.example.com/media/notes.pdf").await;
    blog_click(&app, HOST, "outbound", "blog.example.com/about/").await;
    blog_click(
        &app,
        "www.blog.example.com",
        "outbound",
        "alias.example.net/",
    )
    .await;
    blog_click(
        &app,
        "other.example.net",
        "outbound",
        "unknown.example.net/",
    )
    .await;
    let report = app.analytics(HOST).await;
    assert_eq!(view_counts(&report), view_counts(&viewed));
    assert_eq!(
        report["outbound"],
        json!([{"target": "harbor.example.com/", "clicks": 2}])
    );
    assert_eq!(
        report["downloads"],
        json!([{"target": "blog.example.com/media/notes.pdf", "clicks": 1}])
    );
}

#[tokio::test]
async fn bad_click_beacons_are_rejected() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let headers = [("user-agent", FIREFOX), ("origin", DOCS_ORIGIN)];
    let large = docs_click("outbound", &format!("github.com/{}", "a".repeat(2_100)));
    let res = app.beacon(&headers, &large).await;
    assert_eq!(res.status, StatusCode::PAYLOAD_TOO_LARGE);
    for body in [
        json!({"host": DOCS, "click": "outbound"}),
        json!({"host": DOCS, "click": "print", "target": "github.com/"}),
        json!({"host": DOCS, "click": "outbound", "target": 7}),
        json!({"host": DOCS, "click": "outbound", "target": "github.com/", "text": "Source"}),
        json!({"host": DOCS, "click": "outbound", "target": "github.com/", "path": "/", "referrer": ""}),
        json!({"host": 7, "click": "outbound", "target": "github.com/"}),
    ] {
        let res = app.beacon(&headers, &body).await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "{body}");
    }
    assert_eq!(app.analytics(DOCS).await["outbound"], json!([]));
}

#[tokio::test]
async fn deleting_the_site_deletes_its_clicks() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    app.beacon_from(DOCS_ORIGIN, docs_click("download", "github.com/a.zip"))
        .await;
    app.analytics(DOCS).await;
    assert_eq!(app.analytics_rows("analytics_clicks", DOCS).await, 1);
    let res = app.api("DELETE", &format!("/api/sites/{DOCS}"), None).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());
    assert_eq!(app.analytics_rows("analytics_clicks", DOCS).await, 0);
}
