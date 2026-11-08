//! Blog fonts: a library family in the site stylesheet, the public font files, and the site validation.
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
    CONSOLE, HOST, Setup, TestApp, blog_site,
    font::{FaceStyle, REGULAR, minimal_ttf, upload},
    request,
};

const FAMILY: &str = "Harbor Serif";
const BOLD_ITALIC: FaceStyle = FaceStyle {
    weight: 700,
    italic: true,
};

/// A blog font: the file names and bytes of its regular and bold italic faces.
struct Library {
    regular: String,
    bold_italic: String,
    bytes: Vec<u8>,
}

/// Uploads the regular and the bold italic face of [`FAMILY`].
async fn upload_family(app: &TestApp) -> Library {
    let mut names = Vec::new();
    let bytes = minimal_ttf(FAMILY, REGULAR);
    for font in [bytes.clone(), minimal_ttf(FAMILY, BOLD_ITALIC)] {
        let res = upload(app, font).await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        names.push(res.json()["name"].as_str().unwrap().to_owned());
    }
    let [regular, bold_italic] = <[String; 2]>::try_from(names).unwrap();
    Library {
        regular,
        bold_italic,
        bytes,
    }
}

fn site_with(family: Value) -> Value {
    let mut site = blog_site();
    site["brand"]["font"] = json!("sans");
    site["brand"]["font_family"] = family;
    site
}

/// Seeds the blog, uploads the family, and makes it the blog font.
async fn blog_with_family(app: &TestApp) -> Library {
    app.seed_blog().await;
    let library = upload_family(app).await;
    let res = app.put_site(site_with(json!(FAMILY))).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(res.json()["brand"]["font_family"], FAMILY);
    library
}

async fn page(app: &TestApp, path: &str) -> String {
    let res = app.get(path).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    res.text()
}

fn face_rule(name: &str, weight: u16, style: &str) -> String {
    format!(
        "@font-face{{font-family:\"{FAMILY}\";src:url(/_fonts/{name}) format(\"truetype\");font-weight:{weight};font-style:{style};font-display:swap}}"
    )
}

fn assert_font_file(res: &common::Res, bytes: &[u8]) {
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    assert_eq!(res.body, bytes);
    assert_eq!(res.header("content-type"), "font/ttf");
    assert_eq!(
        res.header("cache-control"),
        "public, max-age=31536000, immutable"
    );
    assert_eq!(res.header("access-control-allow-origin"), "*");
    assert_eq!(res.header("x-content-type-options"), "nosniff");
}

#[tokio::test]
async fn a_library_family_styles_the_blog() {
    let app = TestApp::new().await;
    let library = blog_with_family(&app).await;
    let html = page(&app, "/sqlite-backups/").await;
    assert!(
        html.contains(&face_rule(&library.regular, 400, "normal")),
        "{html}"
    );
    assert!(html.contains(&face_rule(&library.bold_italic, 700, "italic")));
    assert!(html.contains(&format!(
        ":root{{--font-body:\"{FAMILY}\",var(--font-sans);--font-ui:\"{FAMILY}\",var(--font-sans)}}"
    )));
    let preload = format!(
        "<link rel=\"preload\" href=\"/_fonts/{}\" as=\"font\" type=\"font/ttf\" crossorigin>",
        library.regular
    );
    assert!(html.contains(&preload), "{html}");
    assert!(!html.contains(&format!("href=\"/_fonts/{}\"", library.bold_italic)));
    let preview = app
        .api(
            "POST",
            &format!("/api/sites/{HOST}/preview"),
            Some(json!({"body": "Text."})),
        )
        .await;
    assert_eq!(preview.status, StatusCode::OK, "{}", preview.text());
    let css = preview.json()["css"].as_str().unwrap().to_owned();
    assert!(css.contains(&face_rule(&library.regular, 400, "normal")));
}

#[tokio::test]
async fn font_files_are_public_on_blog_and_console_hosts() {
    let app = TestApp::new().await;
    let library = blog_with_family(&app).await;
    let path = format!("/_fonts/{}", library.regular);
    assert_font_file(&app.get(&path).await, &library.bytes);
    assert_font_file(&app.get_on(CONSOLE, &path, &[]).await, &library.bytes);
    for missing in [
        "/_fonts/..%2Fcuhlumn.db",
        "/_fonts/../cuhlumn.db",
        "/_fonts/Missing-00000000.ttf",
        "/_fonts/x.woff2",
    ] {
        assert_eq!(
            app.get(missing).await.status,
            StatusCode::NOT_FOUND,
            "{missing}"
        );
        let console = app.get_on(CONSOLE, missing, &[]).await;
        assert_eq!(console.status, StatusCode::NOT_FOUND, "{missing}");
    }
}

#[tokio::test]
async fn font_files_are_public_with_an_admin_listener() {
    let app = TestApp::open(Setup {
        admin_listener: true,
        ..Setup::default()
    })
    .await;
    app.seed_blog().await;
    let library = upload_family(&app).await;
    let path = format!("/_fonts/{}", library.regular);
    let public = request("GET", HOST, &path).body(Body::empty()).unwrap();
    assert_font_file(&app.send_public(public).await, &library.bytes);
    assert_font_file(&app.get_on(CONSOLE, &path, &[]).await, &library.bytes);
}

#[tokio::test]
async fn site_writes_need_a_library_family() {
    let app = TestApp::new().await;
    app.seed_blog().await;
    let res = app.put_site(site_with(json!("Unknown Serif"))).await;
    res.assert_error(StatusCode::BAD_REQUEST, "validation");
    assert_eq!(res.field(), "brand.font_family");
    let res = app.put_site(site_with(Value::Null)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
}

#[tokio::test]
async fn a_deleted_font_leaves_the_stylesheet() {
    let app = TestApp::new().await;
    let library = blog_with_family(&app).await;
    assert!(page(&app, "/").await.contains("/_fonts/"));
    for name in [&library.regular, &library.bold_italic] {
        let res = app.api("DELETE", &format!("/api/fonts/{name}"), None).await;
        assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.text());
    }
    let html = page(&app, "/sqlite-backups/").await;
    assert!(!html.contains("/_fonts/"), "{html}");
    assert!(!html.contains(FAMILY));
    assert!(html.contains(":root{--font-body:var(--font-ui)}"));
    let path = format!("/_fonts/{}", library.regular);
    assert_eq!(app.get(&path).await.status, StatusCode::NOT_FOUND);
    let res = app.put_site(site_with(json!(FAMILY))).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
}
