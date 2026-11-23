//! Shared helpers of the analytics tests: external sites, beacons of their pages, and reports.

use axum::{body::Body, http::StatusCode};
use cuhlumn_server::db::DbError;
use serde_json::{Value, json};

use crate::common::{HOST, Res, TestApp, request};

pub const FIREFOX: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0";
pub const GOOGLEBOT: &str =
    "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)";
/// The external site of the tests.
pub const DOCS: &str = "docs.example.org";
/// The `Origin` header that a browser sends from a page of [`DOCS`].
pub const DOCS_ORIGIN: &str = "https://docs.example.org";

impl TestApp {
    /// Registers the external site `host` and returns the response body.
    pub async fn register_external(&self, host: &str) -> Value {
        let site = json!({"title": "Harbor Docs", "kind": "external"});
        let res = self
            .api("PUT", &format!("/api/sites/{host}"), Some(site))
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        res.json()
    }

    /// Sends a beacon with `headers` to the blog host `HOST`, which is the host that serves the script.
    pub async fn beacon(&self, headers: &[(&str, &str)], body: &Value) -> Res {
        let mut builder =
            request("POST", HOST, "/_hit").header("content-type", "text/plain;charset=UTF-8");
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        self.send_public(builder.body(Body::from(body.to_string())).unwrap())
            .await
    }

    /// Sends a Firefox beacon from a page of `origin` and checks the empty answer.
    pub async fn beacon_from(&self, origin: &str, body: Value) {
        let headers = [("user-agent", FIREFOX), ("origin", origin)];
        let res = self.beacon(&headers, &body).await;
        assert_empty(&res, &body);
    }

    /// Returns the analytics report of `host` for the last 7 days.
    pub async fn analytics(&self, host: &str) -> Value {
        let path = format!("/api/sites/{host}/analytics?period=week");
        let res = self.api("GET", &path, None).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        res.json()
    }

    /// Returns the number of rows that `table` has for `host`.
    pub async fn analytics_rows(&self, table: &'static str, host: &'static str) -> i64 {
        self.state
            .db()
            .call(move |db| {
                let sql = format!("SELECT COUNT(*) FROM {table} WHERE host = ?1");
                Ok::<_, DbError>(db.query_row(&sql, [host], |row| row.get(0))?)
            })
            .await
            .unwrap()
    }
}

/// Checks the answer of a well-formed beacon: 204 without a body, never stored.
pub fn assert_empty(res: &Res, body: &Value) {
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{body}: {}", res.text());
    assert_eq!(res.header("cache-control"), "no-store", "{body}");
    assert!(res.body.is_empty(), "{body}");
}

/// Returns the body of a page view of [`DOCS`].
pub fn docs_view(path: &str, referrer: &str) -> Value {
    json!({"host": DOCS, "path": path, "referrer": referrer})
}

/// Returns the body of a link click on a page of [`DOCS`].
pub fn docs_click(kind: &str, target: &str) -> Value {
    json!({"host": DOCS, "click": kind, "target": target})
}
