//! Files for feed readers, search engines, and AI assistants: feed, sitemap, robots, llms, Markdown copies, key file, favicon.
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

use axum::http::StatusCode;
use serde_json::json;

use crate::common::{TestApp, blog_site};

#[tokio::test]
async fn markdown_copy_has_title_and_address() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let res = app.get("/sqlite-backups.md").await;
    assert_eq!(res.header("content-type"), "text/markdown; charset=utf-8");
    assert_eq!(res.header("cache-control"), "public, max-age=300");
    let copy = res.text();
    assert!(
        copy.starts_with("# How we back up SQLite\n\nPublished "),
        "{copy}"
    );
    assert!(copy.contains("on Harbor Notes: https://blog.example.com/sqlite-backups/\n\n"));
    assert!(copy.contains("sqlite3 app.db \".backup backup.db\""));
    let page = app.get("/about.md").await.text();
    assert!(page.starts_with(
        "# About\n\nFrom Harbor Notes: https://blog.example.com/about/\n\nA short note about the team.\n"
    ));
}

#[tokio::test]
async fn sitemap_lists_published_entries() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let res = app.get("/sitemap.xml").await;
    assert_eq!(res.header("content-type"), "application/xml; charset=utf-8");
    let sitemap = res.text();
    assert!(sitemap.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset"));
    for loc in [
        "https://blog.example.com/",
        "https://blog.example.com/sqlite-backups/",
        "https://blog.example.com/about/",
    ] {
        assert!(
            sitemap.contains(&format!("<loc>{loc}</loc>")),
            "{loc}: {sitemap}"
        );
    }
    assert!(!sitemap.contains("/tags/"));
}

#[tokio::test]
async fn feed_has_posts_without_pages() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let image = json!({"title": "Pictures", "status": "published",
        "body": "![Door](/media/door.png) and [about](/about/) and [cdn](//cdn.example.com/x)"});
    app.put_post("pictures", image).await;
    let res = app.get("/feed.xml").await;
    assert_eq!(
        res.header("content-type"),
        "application/atom+xml; charset=utf-8"
    );
    let feed = res.text();
    assert!(feed.contains("<entry>") && feed.contains("<title>How we back up SQLite</title>"));
    assert!(!feed.contains("<title>About</title>"));
    assert!(feed.contains("<generator>Cuhlumn</generator>"));
    assert!(feed.contains("<link rel=\"self\" href=\"https://blog.example.com/feed.xml\"/>"));
    assert!(
        feed.contains("src=&quot;https://blog.example.com/media/door.png&quot;"),
        "{feed}"
    );
    assert!(
        feed.contains("href=&quot;https://blog.example.com/about/&quot;"),
        "{feed}"
    );
    assert!(
        feed.contains("href=&quot;//cdn.example.com/x&quot;"),
        "{feed}"
    );
}

#[tokio::test]
async fn llms_files_list_posts_and_pages() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let llms = app.get("/llms.txt").await.text();
    assert!(llms.starts_with(
        "# Harbor Notes\n\n> How a two-person team runs Harbor, a small hosting service.\n\n"
    ));
    assert!(
        llms.contains(
            "This blog is published by the makers of [Harbor](https://harbor.example.com)."
        )
    );
    assert!(llms.contains(
        "## Posts\n\n- [How we back up SQLite](https://blog.example.com/sqlite-backups.md): "
    ));
    assert!(llms.contains(
        "## Pages\n\n- [About](https://blog.example.com/about.md): A short note about the team.\n"
    ));
    let full = app.get("/llms-full.txt").await.text();
    assert!(full.starts_with(&llms));
    assert!(
        full.contains("---\n\n# How we back up SQLite")
            && full.contains("sqlite3 app.db \".backup backup.db\"")
    );
}

#[tokio::test]
async fn robots_points_to_the_sitemap() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let robots = app.get("/robots.txt").await;
    assert_eq!(robots.header("content-type"), "text/plain; charset=utf-8");
    assert_eq!(
        robots.text(),
        "User-agent: *\nAllow: /\nDisallow: /api/\n\nSitemap: https://blog.example.com/sitemap.xml\n"
    );
}

#[tokio::test]
async fn drafts_are_absent_from_every_crawler_file() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    for path in ["/feed.xml", "/sitemap.xml", "/llms.txt", "/llms-full.txt"] {
        let text = app.get(path).await.text();
        assert!(
            !text.contains("unreleased") && !text.contains("Unreleased plans"),
            "{path}: {text}"
        );
    }
}

#[tokio::test]
async fn key_file_follows_the_site_key() {
    let app = TestApp::new().await;
    let site = app.seed_blog().await;
    let key = site["indexnow_key"].as_str().unwrap().to_owned();
    assert_eq!(app.get(&format!("/{key}.txt")).await.text(), key);
    let own = "143f4ce9a39c0691d2e85e8b486880a4";
    let mut changed = blog_site();
    changed["indexnow_key"] = json!(own);
    assert_eq!(app.put_site(changed).await.status, StatusCode::OK);
    assert_eq!(app.get(&format!("/{own}.txt")).await.text(), own);
    assert_eq!(
        app.get(&format!("/{key}.txt")).await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn favicon_draws_the_initial_on_an_accent_disc() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let res = app.get("/favicon.svg").await;
    assert_eq!(res.header("content-type"), "image/svg+xml");
    assert_eq!(res.header("cache-control"), "public, max-age=86400");
    let svg = res.text();
    assert!(svg.starts_with("<svg"), "{svg}");
    assert!(
        svg.contains(r##"<circle cx="16" cy="16" r="16" fill="#0e7490"/>"##),
        "{svg}"
    );
    assert!(
        svg.contains(r##"fill="#ffffff""##) && svg.contains(">H</text>"),
        "{svg}"
    );
    let mut site = blog_site();
    site["brand"] = json!({"accent": "#facc15"});
    let res = app.put_site(site).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let svg = app.get("/favicon.svg").await.text();
    assert!(svg.contains(r##"fill="#facc15""##), "{svg}");
    assert!(
        svg.contains(r##"fill="#000000""##),
        "a light accent gets a black letter: {svg}"
    );
}
