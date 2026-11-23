//! Analytics of external sites: registration, the script file, cross-origin page views, and their limits.
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

use crate::common::{
    HOST, TestApp,
    beacon::{DOCS, DOCS_ORIGIN, FIREFOX, GOOGLEBOT, assert_empty, docs_view},
    request,
};

const SCRIPT: &str = include_str!("../assets/hit.js");
/// A second external site.
const API_DOCS: &str = "api.example.org";
/// The limit of distinct page paths of one external site and day.
const PAGE_LIMIT: usize = 1_000;

#[tokio::test]
async fn blog_hosts_serve_the_script_as_a_small_cacheable_file() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let script = app.get("/_hit.js").await;
    assert_eq!(script.status, StatusCode::OK);
    assert_eq!(
        script.header("content-type"),
        "text/javascript; charset=utf-8"
    );
    assert_eq!(script.header("cache-control"), "public, max-age=86400");
    assert_eq!(script.header("set-cookie"), "");
    assert_eq!(script.text(), SCRIPT);
    assert!(script.body.len() < 1024, "{} bytes", script.body.len());
    for word in ["webdriver", "prerendering", "sendBeacon"] {
        assert!(SCRIPT.contains(word), "the script lacks {word}");
    }
    for word in ["cookie", "Storage", "indexedDB", "preventDefault"] {
        assert!(!SCRIPT.contains(word), "the script uses {word}");
    }
    let etag = [("if-none-match", script.header("etag"))];
    let again = app.get_on(HOST, "/_hit.js", &etag).await;
    assert_eq!(again.status, StatusCode::NOT_MODIFIED);
    let alias = app.get_on("www.blog.example.com", "/_hit.js", &[]).await;
    assert_eq!(alias.status, StatusCode::MOVED_PERMANENTLY);
    assert_eq!(alias.header("location"), "https://blog.example.com/_hit.js");
    for host in ["other.example.net", DOCS] {
        let missing = app.get_on(host, "/_hit.js", &[]).await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND, "{host}");
    }
}

#[tokio::test]
async fn cross_origin_views_count_for_the_registered_site() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let views = [
        ("/", "https://news.example.org/item?id=7"),
        ("/guide/install/", "https://docs.example.org/"),
        ("/guide/install/", ""),
    ];
    for (path, referrer) in views {
        app.beacon_from(DOCS_ORIGIN, docs_view(path, referrer))
            .await;
    }
    let with_port = docs_view("/", "https://www.docs.example.org/guide/");
    app.beacon_from("https://Docs.Example.org:8443", with_port)
        .await;
    let report = app.analytics(DOCS).await;
    assert_eq!(report["views"], 4, "{report}");
    assert_eq!(report["visitors"], 1);
    assert_eq!(
        report["pages"],
        json!([
            {"path": "/", "title": null, "views": 2},
            {"path": "/guide/install/", "title": null, "views": 2},
        ])
    );
    assert_eq!(
        report["referrers"],
        json!([{"host": null, "views": 3}, {"host": "news.example.org", "views": 1}])
    );
    assert_eq!(
        report["devices"],
        json!([{"device": "desktop", "views": 4}])
    );
    assert_eq!(
        report["browsers"],
        json!([{"browser": "firefox", "views": 4}])
    );
    assert_eq!(app.analytics(HOST).await["views"], 0);
}

#[tokio::test]
async fn visitors_are_daily_unique_for_each_external_site() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    app.register_external(API_DOCS).await;
    let send = async |site: &str, ip: &'static str| {
        let origin = format!("https://{site}");
        let headers = [
            ("user-agent", FIREFOX),
            ("origin", origin.as_str()),
            ("x-real-ip", ip),
        ];
        let body = json!({"host": site, "path": "/", "referrer": ""});
        assert_empty(&app.beacon(&headers, &body).await, &body);
    };
    send(DOCS, "203.0.113.5").await;
    send(DOCS, "203.0.113.5").await;
    send(DOCS, "198.51.100.7").await;
    send(API_DOCS, "203.0.113.5").await;
    let docs = app.analytics(DOCS).await;
    assert_eq!((&docs["views"], &docs["visitors"]), (&json!(3), &json!(2)));
    let api = app.analytics(API_DOCS).await;
    assert_eq!((&api["views"], &api["visitors"]), (&json!(1), &json!(1)));
    assert_eq!(app.analytics_rows("analytics_visitors", DOCS).await, 2);
}

#[tokio::test]
async fn views_of_other_origins_do_not_count() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    app.register_external(API_DOCS).await;
    let home = docs_view("/", "");
    for origin in [
        "https://evil.example.net",
        "https://api.example.org",
        "https://blog.example.com",
        "https://docs.example.org.evil.example.net",
        "null",
        "",
    ] {
        app.beacon_from(origin, home.clone()).await;
    }
    let without_origin = app.beacon(&[("user-agent", FIREFOX)], &home).await;
    assert_empty(&without_origin, &home);
    let unregistered = json!({"host": "evil.example.net", "path": "/", "referrer": ""});
    app.beacon_from("https://evil.example.net", unregistered)
        .await;
    let blog = json!({"host": HOST, "path": "/", "referrer": ""});
    app.beacon_from("https://blog.example.com", blog).await;
    for host in [DOCS, API_DOCS, HOST] {
        let report = app.analytics(host).await;
        assert_eq!(report["views"], 0, "{host}: {report}");
        assert_eq!(report["pages"], json!([]), "{host}");
    }
    assert_eq!(app.analytics_rows("analytics_visitors", DOCS).await, 0);
}

#[tokio::test]
async fn a_view_without_a_host_name_never_counts_for_an_external_site() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let body = json!({"path": "/", "referrer": ""});
    let request = request("POST", DOCS, "/_hit")
        .header("user-agent", FIREFOX)
        .header("origin", DOCS_ORIGIN)
        .body(Body::from(body.to_string()))
        .unwrap();
    assert_empty(&app.send_public(request).await, &body);
    assert_eq!(app.analytics(DOCS).await["views"], 0);
}

#[tokio::test]
async fn bots_and_empty_user_agents_are_ignored() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let home = docs_view("/", "");
    for agent in [GOOGLEBOT, "curl/8.10.1", ""] {
        let headers = [("user-agent", agent), ("origin", DOCS_ORIGIN)];
        assert_empty(&app.beacon(&headers, &home).await, &home);
    }
    let without_agent = app.beacon(&[("origin", DOCS_ORIGIN)], &home).await;
    assert_empty(&without_agent, &home);
    assert_eq!(app.analytics(DOCS).await["views"], 0);
}

#[tokio::test]
async fn stored_paths_have_no_query_no_fragment_and_a_length_limit() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let longest = format!("/{}", "a".repeat(255));
    let too_long = format!("{longest}a");
    for path in [
        "/guide/?utm_source=news&token=secret#setup",
        "/guide/#setup",
        "/caf%C3%A9/",
        longest.as_str(),
        too_long.as_str(),
        "guide/",
        "",
        "https://docs.example.org/guide/",
        "/line%0Abreak/",
    ] {
        app.beacon_from(DOCS_ORIGIN, docs_view(path, "")).await;
    }
    let report = app.analytics(DOCS).await;
    assert_eq!(report["views"], 4, "{report}");
    assert_eq!(
        report["pages"],
        json!([
            {"path": "/guide/", "title": null, "views": 2},
            {"path": longest, "title": null, "views": 1},
            {"path": "/café/", "title": null, "views": 1},
        ])
    );
}

#[tokio::test]
async fn distinct_paths_of_one_day_have_a_limit() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let send = async |pages: std::ops::Range<usize>| {
        for index in pages {
            let view = docs_view(&format!("/p{index:04}/"), "");
            app.beacon_from(DOCS_ORIGIN, view).await;
        }
    };
    send(0..600).await;
    assert_eq!(app.analytics(DOCS).await["views"], 600);
    assert_eq!(app.analytics_rows("analytics_pages", DOCS).await, 600);
    send(600..PAGE_LIMIT + 5).await;
    assert_eq!(app.analytics(DOCS).await["views"], PAGE_LIMIT + 5);
    assert_eq!(
        app.analytics_rows("analytics_pages", DOCS).await,
        i64::try_from(PAGE_LIMIT).unwrap()
    );
    send(0..1).await;
    send(PAGE_LIMIT + 5..PAGE_LIMIT + 10).await;
    let report = app.analytics(DOCS).await;
    assert_eq!(report["views"], PAGE_LIMIT + 11, "{report}");
    assert_eq!(
        report["pages"][0],
        json!({"path": "/p0000/", "title": null, "views": 2})
    );
    assert_eq!(
        app.analytics_rows("analytics_pages", DOCS).await,
        i64::try_from(PAGE_LIMIT).unwrap()
    );
}

#[tokio::test]
async fn an_external_site_has_no_public_pages_posts_or_images() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let site = app.register_external(DOCS).await;
    assert_eq!(site["kind"], "external");
    assert_eq!(site["title"], "Harbor Docs");
    for path in ["/", "/feed.xml", "/sitemap.xml", "/robots.txt", "/og.png"] {
        let res = app.get_on(DOCS, path, &[]).await;
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{path}");
        assert_eq!(res.text(), "No blog lives at this address.", "{path}");
    }
    let post = json!({"title": "Hello", "body": "Text.", "status": "published"});
    let requests = [
        ("GET", format!("/api/sites/{DOCS}/posts"), None),
        ("PUT", format!("/api/sites/{DOCS}/posts/hello"), Some(post)),
        ("GET", format!("/api/sites/{DOCS}/media"), None),
        (
            "POST",
            format!("/api/sites/{DOCS}/preview"),
            Some(json!({"body": "Text."})),
        ),
    ];
    for (method, path, body) in requests {
        let res = app.api(method, &path, body).await;
        res.assert_error(StatusCode::CONFLICT, "conflict");
    }
    assert_eq!(app.analytics(DOCS).await["views"], 0);
}

#[tokio::test]
async fn sites_keep_their_kind() {
    let app = TestApp::new().await;
    let blog = app.seed_blog().await;
    assert_eq!(blog["kind"], "blog");
    app.register_external(DOCS).await;
    let list = app.api("GET", "/api/sites", None).await.json();
    let kinds: Vec<(&str, &str)> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|site| {
            (
                site["host"].as_str().unwrap(),
                site["kind"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(kinds, [(HOST, "blog"), (DOCS, "external")]);
    let to_blog = app
        .api(
            "PUT",
            &format!("/api/sites/{DOCS}"),
            Some(json!({"title": "Docs"})),
        )
        .await;
    to_blog.assert_error(StatusCode::CONFLICT, "conflict");
    assert_eq!(to_blog.field(), "kind");
    let to_external = app
        .put_site(json!({"title": "Harbor Notes", "kind": "external"}))
        .await;
    to_external.assert_error(StatusCode::CONFLICT, "conflict");
    assert_eq!(to_external.field(), "kind");
    let renamed = app
        .api(
            "PUT",
            &format!("/api/sites/{DOCS}"),
            Some(json!({"title": "Docs", "kind": "external"})),
        )
        .await;
    assert_eq!(renamed.status, StatusCode::OK, "{}", renamed.text());
    assert_eq!(renamed.json()["title"], "Docs");
    let with_alias =
        json!({"title": "Docs", "kind": "external", "aliases": ["www.docs.example.org"]});
    let res = app
        .api("PUT", &format!("/api/sites/{DOCS}"), Some(with_alias))
        .await;
    res.assert_error(StatusCode::BAD_REQUEST, "validation");
    assert_eq!(res.field(), "aliases");
    let as_alias = app
        .put_site(json!({"title": "Harbor Notes", "aliases": [DOCS]}))
        .await;
    as_alias.assert_error(StatusCode::CONFLICT, "conflict");
    let other = json!({"title": "Docs", "kind": "galaxy"});
    let res = app
        .api("PUT", "/api/sites/api.example.org", Some(other))
        .await;
    res.assert_error(StatusCode::BAD_REQUEST, "validation");
}

#[tokio::test]
async fn deleting_an_external_site_deletes_its_analytics() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    app.beacon_from(DOCS_ORIGIN, docs_view("/", "")).await;
    assert_eq!(app.analytics(DOCS).await["views"], 1);
    let res = app.api("DELETE", &format!("/api/sites/{DOCS}"), None).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());
    for table in ["analytics_days", "analytics_pages", "analytics_visitors"] {
        assert_eq!(app.analytics_rows(table, DOCS).await, 0, "{table}");
    }
    let res = app
        .api("GET", &format!("/api/sites/{DOCS}/analytics"), None)
        .await;
    res.assert_error(StatusCode::NOT_FOUND, "not_found");
    app.beacon_from(DOCS_ORIGIN, docs_view("/", "")).await;
    app.register_external(DOCS).await;
    assert_eq!(app.analytics(DOCS).await["views"], 0);
}

#[tokio::test]
async fn blog_views_with_a_same_origin_header_still_count() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    app.register_external(DOCS).await;
    let body = json!({"path": "/sqlite-backups/", "referrer": "https://docs.example.org/"});
    let headers = [
        ("user-agent", FIREFOX),
        ("origin", "https://blog.example.com"),
    ];
    assert_empty(&app.beacon(&headers, &body).await, &body);
    let report = app.analytics(HOST).await;
    assert_eq!(report["views"], 1, "{report}");
    assert_eq!(
        report["pages"],
        json!([{"path": "/sqlite-backups/", "title": "How we back up SQLite", "views": 1}])
    );
    assert_eq!(
        report["referrers"],
        json!([{"host": "docs.example.org", "views": 1}])
    );
    assert_eq!(report["outbound"], json!([]));
    assert_eq!(report["downloads"], json!([]));
}
