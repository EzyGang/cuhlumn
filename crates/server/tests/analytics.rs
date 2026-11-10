//! Page-view analytics: the beacon script, the beacon endpoint, daily visitors, and the report.
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
use chrono::DateTime;
use cuhlumn_api::{
    analytics::{Browser, Device},
    site::host::Host,
};
use cuhlumn_server::{
    analytics::{View, agent::Client},
    db::{DbError, unix_now},
};
use serde_json::{Value, json};

use crate::common::{HOST, Res, TestApp, request};

const FIREFOX: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0";
const IPHONE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1";
const SCRIPT: &str = include_str!("../assets/hit.js");
const DAY: i64 = 86_400;

/// Sends a beacon to `host` with a User-Agent and extra headers.
async fn beacon_on(app: &TestApp, host: &str, body: &str, headers: &[(&str, &str)]) -> Res {
    let mut builder =
        request("POST", host, "/_hit").header("content-type", "text/plain;charset=UTF-8");
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    app.send_public(builder.body(Body::from(body.to_owned())).unwrap())
        .await
}

/// Sends a Firefox beacon for `path` to `HOST` and checks the empty answer.
async fn hit(app: &TestApp, path: &str, referrer: &str, headers: &[(&str, &str)]) {
    let mut all = vec![("user-agent", FIREFOX)];
    all.extend_from_slice(headers);
    let body = json!({"path": path, "referrer": referrer}).to_string();
    let res = beacon_on(app, HOST, &body, &all).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{path}: {}", res.text());
    assert_eq!(res.header("cache-control"), "no-store");
    assert!(res.body.is_empty());
}

/// Returns the report of `HOST` for `query`.
async fn report(app: &TestApp, query: &str) -> Value {
    let res = app
        .api("GET", &format!("/api/sites/{HOST}/analytics{query}"), None)
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.json()
}

/// Returns the number of rows in `table`.
async fn rows(app: &TestApp, table: &'static str) -> i64 {
    app.state
        .db()
        .call(move |db| {
            let sql = format!("SELECT COUNT(*) FROM {table}");
            Ok::<_, DbError>(db.query_row(&sql, [], |row| row.get(0))?)
        })
        .await
        .unwrap()
}

fn date(time: i64) -> String {
    DateTime::from_timestamp(time, 0)
        .unwrap()
        .format("%Y-%m-%d")
        .to_string()
}

/// Returns the entry of `days` for the date of `time`.
fn day_entry(report: &Value, time: i64) -> Value {
    let date = date(time);
    let days = report["days"].as_array().unwrap();
    days.iter()
        .find(|day| day["date"] == date.as_str())
        .unwrap_or_else(|| panic!("no day {date}"))
        .clone()
}

#[tokio::test]
async fn html_pages_contain_the_script() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let tag = format!("<script>{}</script></body>", SCRIPT.trim_end());
    for path in ["/", "/sqlite-backups/", "/about/", "/tags/databases/"] {
        let page = app.get(path).await;
        assert_eq!(page.status, StatusCode::OK, "{path}");
        assert!(page.text().contains(&tag), "{path} lacks the beacon script");
    }
    for path in ["/feed.xml", "/sitemap.xml", "/sqlite-backups.md"] {
        assert!(!app.get(path).await.text().contains("sendBeacon"), "{path}");
    }
}

#[tokio::test]
async fn beacons_count_for_home_post_and_tag_pages() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    for path in [
        "/",
        "/sqlite-backups/",
        "/sqlite-backups/",
        "/tags/databases/",
        "/about/",
    ] {
        hit(&app, path, "", &[]).await;
    }
    let body = json!({"path": "/", "referrer": ""}).to_string();
    let res = beacon_on(&app, HOST, &body, &[("user-agent", IPHONE)]).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    let report = report(&app, "?period=week").await;
    assert_eq!(report["views"], 6, "{report}");
    assert_eq!(
        report["pages"],
        json!([
            {"path": "/", "title": "Harbor Notes", "views": 2},
            {"path": "/sqlite-backups/", "title": "How we back up SQLite", "views": 2},
            {"path": "/about/", "title": "About", "views": 1},
            {"path": "/tags/databases/", "title": null, "views": 1},
        ])
    );
    assert_eq!(
        report["devices"],
        json!([{"device": "desktop", "views": 5}, {"device": "mobile", "views": 1}])
    );
    assert_eq!(
        report["browsers"],
        json!([{"browser": "firefox", "views": 5}, {"browser": "safari", "views": 1}])
    );
}

#[tokio::test]
async fn beacons_for_other_pages_hosts_and_bots_are_ignored() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    for path in [
        "/unreleased/",
        "/nothing-here/",
        "/tags/unknown/",
        "/feed.xml",
        "/_hit",
    ] {
        hit(&app, path, "", &[]).await;
    }
    let home = json!({"path": "/", "referrer": ""}).to_string();
    let alias = beacon_on(
        &app,
        "www.blog.example.com",
        &home,
        &[("user-agent", FIREFOX)],
    )
    .await;
    assert_eq!(alias.status, StatusCode::NO_CONTENT);
    let unknown = beacon_on(&app, "other.example.org", &home, &[("user-agent", FIREFOX)]).await;
    assert_eq!(unknown.status, StatusCode::NO_CONTENT);
    let googlebot = "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)";
    for agent in [googlebot, "curl/8.10.1", ""] {
        let res = beacon_on(&app, HOST, &home, &[("user-agent", agent)]).await;
        assert_eq!(res.status, StatusCode::NO_CONTENT, "{agent}");
        assert_eq!(res.header("cache-control"), "no-store");
    }
    let without_agent = beacon_on(&app, HOST, &home, &[]).await;
    assert_eq!(without_agent.status, StatusCode::NO_CONTENT);
    let report = report(&app, "?period=week").await;
    assert_eq!(report["views"], 0, "{report}");
    assert_eq!(report["pages"], json!([]));
    assert_eq!(rows(&app, "analytics_visitors").await, 0);
}

#[tokio::test]
async fn bad_beacons_are_rejected() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let agent = [("user-agent", FIREFOX)];
    let large = json!({"path": "/", "referrer": "x".repeat(2_100)}).to_string();
    let res = beacon_on(&app, HOST, &large, &agent).await;
    assert_eq!(res.status, StatusCode::PAYLOAD_TOO_LARGE);
    for body in [
        "not json",
        "",
        r#"{"path": "/"}"#,
        r#"{"path": "/", "referrer": "", "title": "Harbor"}"#,
        r#"{"path": 1, "referrer": ""}"#,
    ] {
        let res = beacon_on(&app, HOST, body, &agent).await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "{body}");
    }
    let get = app.get("/_hit").await;
    assert_eq!(get.status, StatusCode::NOT_FOUND);
    let put = app
        .send_public(request("PUT", HOST, "/_hit").body(Body::empty()).unwrap())
        .await;
    assert_eq!(put.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(report(&app, "").await["views"], 0);
}

#[tokio::test]
async fn visitors_are_unique_per_day() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let now = unix_now();
    let yesterday = now - DAY;
    let view = View {
        host: Host::new(HOST).unwrap(),
        path: "/".to_owned(),
        referrer: None,
        client: Client {
            device: Device::Desktop,
            browser: Browser::Firefox,
        },
        ip: "203.0.113.5".parse().ok(),
        user_agent: FIREFOX.to_owned(),
    };
    app.state.analytics().record(view, yesterday);
    app.state
        .analytics()
        .flush(app.state.db(), yesterday)
        .await
        .unwrap();
    assert_eq!(rows(&app, "analytics_visitors").await, 1);
    let first = [("x-real-ip", "203.0.113.5")];
    hit(&app, "/", "", &first).await;
    hit(&app, "/sqlite-backups/", "", &first).await;
    hit(&app, "/", "", &[("x-real-ip", "198.51.100.7")]).await;
    let report = report(&app, "?period=week").await;
    assert_eq!(
        day_entry(&report, yesterday),
        json!({"date": date(yesterday), "views": 1, "visitors": 1})
    );
    assert_eq!(
        day_entry(&report, now),
        json!({"date": date(now), "views": 3, "visitors": 2})
    );
    assert_eq!(report["views"], 4);
    assert_eq!(report["visitors"], 3);
    assert_eq!(rows(&app, "analytics_visitors").await, 2);
    let past = app
        .state
        .db()
        .call(move |db| {
            Ok::<i64, DbError>(db.query_row(
                "SELECT COUNT(*) FROM analytics_visitors WHERE day < ?1",
                [now.div_euclid(DAY)],
                |row| row.get(0),
            )?)
        })
        .await
        .unwrap();
    assert_eq!(past, 0);
}

#[tokio::test]
async fn referrers_store_the_host_only() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    hit(&app, "/", "https://blog.example.com/sqlite-backups/", &[]).await;
    hit(&app, "/", "", &[]).await;
    hit(&app, "/", "https://news.example.org/item?id=7", &[]).await;
    hit(
        &app,
        "/sqlite-backups/",
        "https://www.News.example.org:443/",
        &[],
    )
    .await;
    hit(
        &app,
        "/sqlite-backups/",
        "https://search.example/?q=sqlite",
        &[],
    )
    .await;
    let report = report(&app, "?period=week").await;
    assert_eq!(
        report["referrers"],
        json!([
            {"host": null, "views": 2},
            {"host": "news.example.org", "views": 2},
            {"host": "search.example", "views": 1},
        ])
    );
}

#[tokio::test]
async fn report_has_zero_filled_days() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    hit(&app, "/", "", &[]).await;
    let now = unix_now();
    let week = report(&app, "?period=week").await;
    assert_eq!(week["period"], "week");
    assert_eq!(week["from"], date(now - 6 * DAY));
    assert_eq!(week["to"], date(now));
    let days = week["days"].as_array().unwrap();
    let expected: Vec<Value> = (0..7)
        .rev()
        .map(|back| {
            let views = i32::from(back == 0);
            json!({"date": date(now - back * DAY), "views": views, "visitors": views})
        })
        .collect();
    assert_eq!(days, &expected);
    for (query, period, length) in [
        ("", "month", 30),
        ("?period=month", "month", 30),
        ("?period=quarter", "quarter", 90),
        ("?period=year", "year", 365),
    ] {
        let report = report(&app, query).await;
        assert_eq!(report["period"], period, "{query}");
        assert_eq!(report["days"].as_array().unwrap().len(), length, "{query}");
        assert_eq!(report["views"], 1, "{query}");
    }
}

#[tokio::test]
async fn report_rejects_bad_periods_and_unknown_sites() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    for query in ["?period=decade", "?period=", "?range=week"] {
        let res = app
            .api("GET", &format!("/api/sites/{HOST}/analytics{query}"), None)
            .await;
        res.assert_error(StatusCode::BAD_REQUEST, "validation");
        assert_eq!(res.field(), "period", "{query}");
    }
    let res = app
        .api("GET", "/api/sites/notes.example.com/analytics", None)
        .await;
    res.assert_error(StatusCode::NOT_FOUND, "not_found");
    let res = app
        .send(
            request("GET", "admin.test", &format!("/api/sites/{HOST}/analytics"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    res.assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
}

#[tokio::test]
async fn deleting_the_site_deletes_its_analytics() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    hit(&app, "/", "https://news.example.org/", &[]).await;
    assert_eq!(report(&app, "").await["views"], 1);
    for table in [
        "analytics_days",
        "analytics_pages",
        "analytics_referrers",
        "analytics_clients",
        "analytics_visitors",
    ] {
        assert_eq!(rows(&app, table).await, 1, "{table}");
    }
    let res = app.api("DELETE", &format!("/api/sites/{HOST}"), None).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());
    for table in [
        "analytics_days",
        "analytics_pages",
        "analytics_referrers",
        "analytics_clients",
        "analytics_visitors",
    ] {
        assert_eq!(rows(&app, table).await, 0, "{table}");
    }
}

#[tokio::test]
async fn a_site_deleted_before_the_flush_is_skipped() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    hit(&app, "/", "", &[]).await;
    let res = app.api("DELETE", &format!("/api/sites/{HOST}"), None).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());
    app.state
        .analytics()
        .flush(app.state.db(), unix_now())
        .await
        .unwrap();
    assert_eq!(rows(&app, "analytics_days").await, 0);
    assert_eq!(rows(&app, "analytics_visitors").await, 0);
    assert_eq!(rows(&app, "analytics_salt").await, 1);
}
