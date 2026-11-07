//! Appearance options on public pages: palettes, width, theme, home layouts, social links, and the preview stylesheet.
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
use serde_json::{Value, json};

use crate::common::{HOST, POST_BODY, TestApp, blog_site};

/// Seeds the blog, then replaces the site brand with the Harbor accent and `brand`.
async fn seed_with(brand: Value) -> TestApp {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let mut site = blog_site();
    for (key, value) in brand.as_object().unwrap() {
        site["brand"][key] = value.clone();
    }
    let res = app.put_site(site).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    app
}

async fn page(app: &TestApp, path: &str) -> String {
    let res = app.get(path).await;
    assert_eq!(res.status, StatusCode::OK, "{path}: {}", res.text());
    res.text()
}

fn assert_contains(page: &str, needles: &[&str], what: &str) {
    for needle in needles {
        assert!(page.contains(needle), "{what} lacks {needle}:\n{page}");
    }
}

fn assert_lacks(page: &str, needles: &[&str], what: &str) {
    for needle in needles {
        assert!(!page.contains(needle), "{what} has {needle}:\n{page}");
    }
}

#[tokio::test]
async fn palette_and_width_reach_the_page() {
    let app = seed_with(json!({"palette": "sepia", "width": "wide"})).await;
    let html = page(&app, "/").await;
    assert_contains(
        &html,
        &[
            ":root{--color-bg:#f6efe2;",
            ":root[data-theme=dark]{--color-bg:#1a1612;",
            "@media (prefers-color-scheme:dark){:root:not([data-theme=light]){--color-bg:#1a1612;",
            "--measure:52rem;",
            r##"<meta name="theme-color" media="(prefers-color-scheme: light)" content="#f6efe2">"##,
            r##"<meta name="theme-color" media="(prefers-color-scheme: dark)" content="#1a1612">"##,
        ],
        "sepia page",
    );
    let default = seed_with(json!({})).await;
    let html = page(&default, "/").await;
    assert_contains(
        &html,
        &[":root{--color-bg:#f7f7f4;", "--measure:42rem;"],
        "paper page",
    );
    let narrow = seed_with(json!({"width": "narrow"})).await;
    assert_contains(
        &page(&narrow, "/").await,
        &["--measure:38rem;"],
        "narrow page",
    );
}

#[tokio::test]
async fn system_theme_has_the_toggle_and_no_forced_theme() {
    let app = seed_with(json!({})).await;
    let html = page(&app, "/sqlite-backups/").await;
    assert_contains(
        &html,
        &[
            r#"<html lang="en">"#,
            r#"<meta name="color-scheme" content="light dark">"#,
            r#"id="theme-switch""#,
            r#"localStorage.getItem("cuhlumn:theme")"#,
            r#"localStorage.setItem("cuhlumn:theme""#,
        ],
        "system theme",
    );
}

#[tokio::test]
async fn forced_theme_without_toggle() {
    let app = seed_with(json!({"theme": "dark", "theme_toggle": false})).await;
    let html = page(&app, "/sqlite-backups/").await;
    assert_contains(
        &html,
        &[
            r#"<html lang="en" data-theme="dark">"#,
            r#"<meta name="color-scheme" content="dark">"#,
            r##"<meta name="theme-color" content="#121417">"##,
        ],
        "forced dark",
    );
    assert_lacks(
        &html,
        &[
            "theme-switch\"",
            "cuhlumn:theme",
            "prefers-color-scheme: light",
        ],
        "forced dark",
    );
}

#[tokio::test]
async fn forced_theme_with_toggle_keeps_the_stored_choice() {
    let app = seed_with(json!({"theme": "light"})).await;
    let html = page(&app, "/").await;
    assert_contains(
        &html,
        &[
            r#"<html lang="en" data-theme="light">"#,
            r#"<meta name="color-scheme" content="light">"#,
            r#"id="theme-switch""#,
            r#"localStorage.getItem("cuhlumn:theme")"#,
        ],
        "forced light",
    );
}

#[tokio::test]
async fn list_layout_shows_summaries() {
    let app = seed_with(json!({"home_layout": "list"})).await;
    let html = page(&app, "/").await;
    assert_contains(
        &html,
        &[
            r#"<ol class="entry-list"><li><time class="entry-date""#,
            r#"<h2 class="entry-title"><a href="/sqlite-backups/">How we back up SQLite</a></h2><p class="entry-summary">"#,
        ],
        "list layout",
    );
}

#[tokio::test]
async fn compact_layout_shows_date_and_title_only() {
    let app = seed_with(json!({"home_layout": "compact"})).await;
    for path in ["/", "/tags/databases/"] {
        let html = page(&app, path).await;
        assert_contains(
            &html,
            &[
                r#"<ol class="entry-list entry-compact"><li><time class="entry-date""#,
                r#"<h2 class="entry-title"><a href="/sqlite-backups/">How we back up SQLite</a></h2></li>"#,
            ],
            path,
        );
        assert_lacks(&html, &[r#"<p class="entry-summary">"#], path);
    }
}

#[tokio::test]
async fn cards_layout_shows_covers_when_posts_have_them() {
    let app = seed_with(json!({"home_layout": "cards"})).await;
    let html = page(&app, "/").await;
    assert_contains(
        &html,
        &[
            r#"<ol class="entry-cards"><li class="entry-card"><div class="entry-card-body"><time class="entry-date""#,
            r#"<p class="entry-summary">"#,
        ],
        "cards without cover",
    );
    assert_lacks(&html, &["entry-cover\""], "cards without cover");
    let res = app
        .put_post(
            "sqlite-backups",
            json!({"title": "How we back up SQLite", "body": POST_BODY, "tags": ["Databases"],
                   "status": "published", "cover": "/media/backups.png"}),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    for path in ["/", "/tags/databases/"] {
        assert_contains(
            &page(&app, path).await,
            &[
                r#"<li class="entry-card"><img class="entry-cover" src="/media/backups.png" alt="" width="1200" height="630" fetchpriority="high">"#,
            ],
            path,
        );
    }
}

#[tokio::test]
async fn social_links_show_in_the_footer_and_the_publisher_data() {
    let social = json!([
        {"network": "github", "url": "https://github.com/maralind"},
        {"network": "mastodon", "url": "https://mastodon.example.com/@mara"},
        {"network": "email", "url": "mailto:mara@harbor.example.com"},
    ]);
    let app = seed_with(json!({"social": social})).await;
    let html = page(&app, "/").await;
    let footer = &html[html.find("<footer").unwrap()..];
    assert_contains(
        footer,
        &[
            r#"<ul class="social-links" aria-label="Social links">"#,
            r#"<a href="https://github.com/maralind" rel="me noopener" aria-label="GitHub"><svg"#,
            r#"<a href="https://mastodon.example.com/@mara" rel="me noopener" aria-label="Mastodon">"#,
            r#"<a href="mailto:mara@harbor.example.com" rel="me noopener" aria-label="Email">"#,
        ],
        "footer",
    );
    assert_eq!(html.matches(r#"class="social-links""#).count(), 1);
    assert_contains(
        &html,
        &[r#""sameAs":["https://github.com/maralind","https://mastodon.example.com/@mara"]"#],
        "home JSON-LD",
    );
    let post = page(&app, "/sqlite-backups/").await;
    assert_contains(
        &post,
        &[r#""sameAs":["https://github.com/maralind","https://mastodon.example.com/@mara"]"#],
        "post JSON-LD",
    );

    let header = seed_with(json!({"social": social, "social_in_header": true})).await;
    let html = page(&header, "/").await;
    let head = &html[html.find("<header").unwrap()..html.find("</header>").unwrap()];
    assert_contains(
        head,
        &[r#"rel="me noopener" aria-label="GitHub""#],
        "header",
    );
    assert_eq!(html.matches(r#"class="social-links""#).count(), 2);
}

#[tokio::test]
async fn sites_without_social_links_have_no_list_or_same_as() {
    let app = seed_with(json!({})).await;
    let html = page(&app, "/").await;
    assert_lacks(&html, &[r#"class="social-links""#, "sameAs"], "plain site");
}

#[tokio::test]
async fn preview_returns_the_site_stylesheet() {
    let app = seed_with(json!({"palette": "mono", "width": "narrow"})).await;
    let res = app
        .api(
            "POST",
            &format!("/api/sites/{HOST}/preview"),
            Some(json!({"body": POST_BODY})),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let css = res.json()["css"].as_str().unwrap().to_owned();
    assert_contains(
        &css,
        &[
            ".post-body{",
            ":root{--color-bg:#ffffff;",
            "--measure:38rem;",
            "--color-accent:#0e7490;",
        ],
        "preview css",
    );
}
