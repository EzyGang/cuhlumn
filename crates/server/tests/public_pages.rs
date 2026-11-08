//! Public pages: home, posts, tags, redirects, caching, branding, the 404 page, and the preview endpoint.
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
use cuhlumn_api::site::host::Host;
use cuhlumn_server::db::unix_now;
use serde_json::{Value, json};

use crate::common::{
    ADMIN_HOST, HOST, PNG, POST_BODY, Res, TOKEN, TestApp, blog_site, json_body, request,
};

fn assert_contains(page: &str, needles: &[&str], what: &str) {
    for needle in needles {
        assert!(page.contains(needle), "{what} lacks {needle}:\n{page}");
    }
}

fn site_with(changes: Value) -> Value {
    let mut site = blog_site();
    for (key, value) in changes.as_object().unwrap() {
        site[key] = value.clone();
    }
    site
}

async fn upload(app: &TestApp, bytes: &[u8]) -> String {
    let request = request("POST", ADMIN_HOST, &format!("/api/sites/{HOST}/media"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "image/png")
        .body(Body::from(bytes.to_vec()))
        .unwrap();
    let res = app.send(request).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let url = res.json()["url"].as_str().unwrap().to_owned();
    url.trim_start_matches("https://blog.example.com")
        .to_owned()
}

fn assert_redirect(res: &Res, location: &str) {
    assert_eq!(res.status, StatusCode::MOVED_PERMANENTLY, "{}", res.text());
    assert_eq!(res.header("location"), location);
}

#[tokio::test]
async fn home_page_lists_published_posts() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let res = app.get("/").await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.header("content-type"), "text/html; charset=utf-8");
    let html = res.text();
    assert_contains(
        &html,
        &[
            "<title>Harbor Notes</title>",
            r#"rel="canonical" href="https://blog.example.com/""#,
            "og:image",
            r#""@type":"Blog""#,
            "How we back up SQLite",
            r#"href="/about/""#,
            r#"href="https://harbor.example.com""#,
            "twitter:card",
            r#"Published with <a href="https://github.com/EzyGang/cuhlumn">Cuhlumn</a>"#,
            r#"rel="icon" href="/favicon.svg""#,
            "--color-accent:#0e7490",
            r#"localStorage.getItem("cuhlumn:theme")"#,
            r##"href="#content""##,
        ],
        "home page",
    );
    assert!(!html.contains("Unreleased plans"));
}

#[tokio::test]
async fn post_page_has_structured_data_outline_and_call_to_action() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let html = app.get("/sqlite-backups/").await.text();
    assert_contains(
        &html,
        &[
            r#""@type":"BlogPosting""#,
            r#""@type":"BreadcrumbList""#,
            r#"id="why-one-file""#,
            r#"id="why-one-file-2""#,
            r##"href="#the-command""##,
            "<summary>Contents</summary>",
            r#"id="theme-switch""#,
            r#"aria-pressed="false""#,
            "Harbor runs your app on a server that you own.",
            "article:published_time",
            r#"property="og:type" content="article""#,
            r#"href="/tags/databases/""#,
            "https://blog.example.com/sqlite-backups.md",
            "og/sqlite-backups.png?v=",
            r#"property="og:image:width" content="1200""#,
            "1 min read",
            r#"<a href="https://mara.example.com" rel="author">Mara Lind</a>"#,
        ],
        "post page",
    );
}

#[tokio::test]
async fn short_posts_have_no_table_of_contents() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let post = json!({"title": "Two headings", "body": "## One\n\nA.\n\n## Two\n\nB.", "status": "published"});
    app.put_post("two", post).await;
    let html = app.get("/two/").await.text();
    assert!(html.contains(r#"id="one""#) && !html.contains("<summary>Contents</summary>"));
}

#[tokio::test]
async fn pages_are_websites_without_more_posts() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let other = json!({"title": "Second post", "body": "More.", "status": "published"});
    app.put_post("second", other).await;
    let page = app.get("/about/").await.text();
    assert_contains(
        &page,
        &[
            r#"property="og:type" content="website""#,
            r#""@type":"WebPage""#,
            "Harbor runs your app on a server that you own.",
        ],
        "page",
    );
    assert!(!page.contains("More posts") && !page.contains("article:published_time"));
    let post = app.get("/second/").await.text();
    assert!(post.contains("More posts") && post.contains("How we back up SQLite"));
}

#[tokio::test]
async fn caching_answers_304_and_gzip() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let first = app.get("/sqlite-backups/").await;
    assert_eq!(first.header("cache-control"), "public, max-age=60");
    let host = Host::new(HOST).unwrap();
    assert!(
        app.state
            .cache()
            .get(&host, "/sqlite-backups/", unix_now())
            .is_some()
    );
    let etag = first.header("etag").to_owned();
    let fresh = app
        .get_on(HOST, "/sqlite-backups/", &[("if-none-match", &etag)])
        .await;
    assert_eq!(fresh.status, StatusCode::NOT_MODIFIED);
    let gzip = app
        .get_on(HOST, "/sqlite-backups/", &[("accept-encoding", "gzip, br")])
        .await;
    assert_eq!(gzip.header("content-encoding"), "gzip");
    assert!(gzip.body.len() < first.body.len());
}

#[tokio::test]
async fn redirects_add_slashes_and_follow_aliases() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    assert_redirect(&app.get("/sqlite-backups").await, "/sqlite-backups/");
    assert_redirect(&app.get("/favicon.ico").await, "/favicon.svg");
    assert_redirect(&app.get("/tags/databases").await, "/tags/databases/");
    assert_redirect(
        &app.get("/sqlite-backups?utm_source=x&b=1").await,
        "/sqlite-backups/?utm_source=x&b=1",
    );
    assert_redirect(
        &app.get("/tags/databases?page=2").await,
        "/tags/databases/?page=2",
    );
    let alias = app
        .get_on("www.blog.example.com", "/sqlite-backups/?ref=x", &[])
        .await;
    assert_redirect(&alias, "https://blog.example.com/sqlite-backups/?ref=x");
    let post = json!({"title": "Tea", "body": "Leaves.", "tags": ["Thé"], "status": "published"});
    app.put_post("tea", post).await;
    assert_redirect(&app.get("/tags/th%C3%A9").await, "/tags/th%C3%A9/");
    assert_eq!(app.get("/tags/th%C3%A9/").await.status, StatusCode::OK);
    assert_eq!(app.get("/tags/unknown").await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn site_change_gives_post_cards_a_new_address() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let card_address = |html: &str| {
        let start = html.find("/og/sqlite-backups.png?v=").unwrap();
        let rest = &html[start..];
        rest[..rest.find('"').unwrap()].to_owned()
    };
    let before = card_address(&app.get("/sqlite-backups/").await.text());
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let res = app
        .put_site(site_with(json!({"brand": {"accent": "#2563eb"}})))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let after = card_address(&app.get("/sqlite-backups/").await.text());
    assert_ne!(before, after);
    let site_time = res.json()["updated_at"].as_i64().unwrap();
    assert_eq!(after, format!("/og/sqlite-backups.png?v={site_time}"));
}

#[tokio::test]
async fn unknown_host_gets_plain_404() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let res = app.get_on("nobody.example.org", "/", &[]).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.text(), "No blog lives at this address.");
}

#[tokio::test]
async fn drafts_are_hidden_on_every_page() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    for path in [
        "/unreleased/",
        "/unreleased",
        "/unreleased.md",
        "/og/unreleased.png",
    ] {
        assert_eq!(app.get(path).await.status, StatusCode::NOT_FOUND, "{path}");
    }
    for path in ["/", "/tags/databases/", "/sqlite-backups/", "/nothing/"] {
        assert!(
            !app.get(path).await.text().contains("Unreleased plans"),
            "{path}"
        );
    }
}

#[tokio::test]
async fn tag_page_is_noindex_and_tags_root_is_missing() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let tag = app.get("/tags/databases/").await;
    assert_eq!(tag.status, StatusCode::OK);
    assert_contains(
        &tag.text(),
        &[
            "noindex",
            "<h1>#Databases</h1>",
            r#""@type":"CollectionPage""#,
        ],
        "tag page",
    );
    for path in ["/tags/", "/tags"] {
        assert_eq!(app.get(path).await.status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn only_get_and_head_are_allowed() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    for method in ["POST", "PUT", "DELETE"] {
        let res = app
            .send(
                request(method, HOST, "/sqlite-backups/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;
        assert_eq!(res.status, StatusCode::METHOD_NOT_ALLOWED, "{method}");
        assert_eq!(res.header("allow"), "GET, HEAD");
    }
    let head = app
        .send(request("HEAD", HOST, "/").body(Body::empty()).unwrap())
        .await;
    assert_eq!(head.status, StatusCode::OK);
}

#[tokio::test]
async fn not_found_page_has_no_canonical_or_json_ld() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let res = app.get("/nothing/here").await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.header("content-type"), "text/html; charset=utf-8");
    let html = res.text();
    assert_contains(
        &html,
        &[
            "<h1>Page not found</h1>",
            "<title>Page not found · Harbor Notes</title>",
            "noindex",
            "How we back up SQLite",
        ],
        "404 page",
    );
    assert!(!html.contains("rel=\"canonical\"") && !html.contains("application/ld+json"));
    let host = Host::new(HOST).unwrap();
    assert!(
        app.state
            .cache()
            .get(&host, "/nothing/here", unix_now())
            .is_none()
    );
}

#[tokio::test]
async fn edit_invalidates_the_cache() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    assert!(app.get("/").await.text().contains("How we back up SQLite"));
    let edit = json!({"title": "Moving everything to one server", "body": POST_BODY, "status": "published"});
    app.put_post("sqlite-backups", edit).await;
    assert!(
        app.get("/")
            .await
            .text()
            .contains("Moving everything to one server")
    );
}

#[tokio::test]
async fn write_on_one_site_keeps_the_cache_of_another() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let other = "other.example.com";
    let res = app
        .api(
            "PUT",
            &format!("/api/sites/{other}"),
            Some(json!({"title": "Other"})),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(app.get_on(other, "/", &[]).await.status, StatusCode::OK);
    app.get("/").await;
    let (other, host) = (Host::new(other).unwrap(), Host::new(HOST).unwrap());
    assert!(app.state.cache().get(&other, "/", unix_now()).is_some());
    app.put_post(
        "sqlite-backups",
        json!({"title": "Edited", "body": "New.", "status": "published"}),
    )
    .await;
    assert!(app.state.cache().get(&host, "/", unix_now()).is_none());
    assert!(app.state.cache().get(&other, "/", unix_now()).is_some());
}

#[tokio::test]
async fn media_round_trip_through_the_public_address() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let path = upload(&app, PNG).await;
    let image = app.get(&path).await;
    assert_eq!(image.status, StatusCode::OK);
    assert_eq!(image.body, PNG);
    assert_eq!(image.header("content-type"), "image/png");
    assert_eq!(
        image.header("cache-control"),
        "public, max-age=31536000, immutable"
    );
    assert_eq!(image.header("x-content-type-options"), "nosniff");
    for path in [
        "/media/../../cuhlumn.db",
        "/media/..%2F..%2Fcuhlumn.db",
        "/media/0123456789abcdef0123456789abcdef.png",
    ] {
        assert_eq!(app.get(path).await.status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn a_page_can_be_the_home_page() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let res = app.put_site(site_with(json!({"home": "about"}))).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let home = app.get("/").await.text();
    assert_contains(
        &home,
        &[
            "A short note about the team.",
            "Latest posts",
            r#""@type":"WebSite""#,
            "<title>About</title>",
        ],
        "home page",
    );
    assert!(!home.contains(r#"href="/about/""#));
    assert_redirect(&app.get("/about/").await, "/");
    assert_redirect(&app.get("/about").await, "/");
    assert!(!app.get("/sitemap.xml").await.text().contains("/about/"));
}

#[tokio::test]
async fn branding_changes_the_page() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let logo = upload(&app, PNG).await;
    let brand = json!({
        "accent": "#0e7490",
        "font": "sans",
        "logo": logo,
        "og_image": "https://cdn.example.com/share.png",
        "links": [{"label": "Status", "url": "https://status.harbor.example.com"}],
        "head": r#"<meta name="google-site-verification" content="abc123">"#,
        "footer": "© 2026 Example Inc.",
        "credit": false,
    });
    let res = app.put_site(site_with(json!({"brand": brand}))).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let home = app.get("/").await.text();
    let icon = format!(r#"rel="icon" href="{logo}""#);
    assert_contains(
        &home,
        &[
            r#"class="brand-logo""#,
            r#"property="og:image" content="https://cdn.example.com/share.png""#,
            ">Status</a>",
            r#"<meta name="google-site-verification" content="abc123">"#,
            "© 2026 Example Inc.",
            "--font-body:var(--font-ui)",
            &icon,
        ],
        "branded home",
    );
    assert!(!home.contains("Published with") && !home.contains("og:image:width"));
    let own = json!({"title": "How we back up SQLite", "body": POST_BODY, "status": "published", "og_image": "/media/own.png"});
    app.put_post("sqlite-backups", own).await;
    let post = app.get("/sqlite-backups/").await.text();
    assert!(
        post.contains(r#"property="og:image" content="https://blog.example.com/media/own.png""#)
    );
}

#[test]
fn tables_stack_on_narrow_screens() {
    use cuhlumn_server::render::markdown::render;
    let two = render(
        "| Setting | Meaning |\n|---|---|\n| `KEEP_DAYS` | Days to [keep](https://docs.example.org) copies |\n",
    );
    assert!(
        two.html.contains("<dl class=\"term-list\">"),
        "{}",
        two.html
    );
    assert!(
        two.html.contains(
            "<dt><code>KEEP_DAYS</code></dt><dd>Days to <a href=\"https://docs.example.org\">keep</a> copies</dd>"
        ),
        "{}",
        two.html
    );
    let wide = render(
        "| Region | *Latency* | Nodes |\n|---|---|---|\n| Frankfurt | 12 ms | 3 |\n| Oslo | 19 ms |  |\n",
    );
    assert!(
        wide.html.contains("<div class=\"grid-table\"><table>"),
        "{}",
        wide.html
    );
    assert!(
        wide.html.contains("<th scope=\"row\">Frankfurt</th><td data-label=\"Latency\"><span>12 ms</span></td><td data-label=\"Nodes\"><span>3</span></td>"),
        "{}",
        wide.html
    );
    assert!(
        wide.html
            .contains("<th scope=\"col\"><em>Latency</em></th>"),
        "{}",
        wide.html
    );
    assert!(
        wide.text.contains("Frankfurt 12 ms 3"),
        "plain text keeps the cells: {}",
        wide.text
    );
}

#[tokio::test]
async fn preview_renders_markdown_and_requires_auth() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let path = format!("/api/sites/{HOST}/preview");
    let res = app
        .api("POST", &path, Some(json!({"body": POST_BODY})))
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let preview = res.json();
    assert!(
        preview["html"]
            .as_str()
            .unwrap()
            .contains(r#"<h2 id="why-one-file">"#)
    );
    let ids: Vec<&str> = preview["toc"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "why-one-file",
            "how-the-backup-runs",
            "the-command",
            "why-one-file-2"
        ]
    );
    assert_eq!(preview["toc"][2]["level"], 3);
    assert_eq!(preview["word_count"], 36);
    let anonymous = json_body(
        request("POST", ADMIN_HOST, &path),
        Some(json!({"body": "x"})),
    );
    app.send(anonymous)
        .await
        .assert_error(StatusCode::UNAUTHORIZED, "unauthorized");
    app.api(
        "POST",
        "/api/sites/nobody.example.org/preview",
        Some(json!({"body": "x"})),
    )
    .await
    .assert_error(StatusCode::NOT_FOUND, "not_found");
}
