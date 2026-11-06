//! Media routes: upload checks, listing, deletion, and odd names.
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

use crate::common::{ADMIN_HOST, HOST, PNG, Res, TOKEN, TestApp, request};

async fn upload(app: &TestApp, host: &str, content_type: &str, body: Vec<u8>) -> Res {
    let request = request("POST", ADMIN_HOST, &format!("/api/sites/{host}/media"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", content_type)
        .body(Body::from(body))
        .unwrap();
    app.send(request).await
}

#[tokio::test]
async fn upload_list_and_delete_an_image() {
    let app = TestApp::new().await;
    app.create_site().await;
    let res = upload(&app, HOST, "image/png", PNG.to_vec()).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text());
    let saved = res.json();
    let name = saved["name"].as_str().unwrap().to_owned();
    assert_eq!(name.len(), 36);
    assert!(name.ends_with(".png"));
    assert_eq!(saved["url"], format!("https://{HOST}/media/{name}"));
    let file = app.data().join("media").join(HOST).join(&name);
    assert_eq!(std::fs::read(&file).unwrap(), PNG);
    let again = upload(&app, HOST, "image/png; charset=binary", PNG.to_vec()).await;
    assert_eq!(again.json()["name"], name.as_str());
    let list = app
        .api("GET", &format!("/api/sites/{HOST}/media"), None)
        .await
        .json();
    let items = list.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["content_type"], "image/png");
    assert_eq!(items[0]["size"], PNG.len());
    assert!(items[0]["created_at"].is_i64());
    let path = format!("/api/sites/{HOST}/media/{name}");
    assert_eq!(
        app.api("DELETE", &path, None).await.status,
        StatusCode::NO_CONTENT
    );
    assert!(!file.exists());
    app.api("DELETE", &path, None)
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    assert_eq!(
        app.api("GET", &format!("/api/sites/{HOST}/media"), None)
            .await
            .json(),
        serde_json::json!([])
    );
}

#[tokio::test]
async fn uploads_must_be_supported_images() {
    let app = TestApp::new().await;
    app.create_site().await;
    upload(&app, HOST, "image/jpeg", PNG.to_vec())
        .await
        .assert_error(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type");
    upload(&app, HOST, "image/svg+xml", b"<svg/>".to_vec())
        .await
        .assert_error(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type");
    upload(&app, HOST, "text/plain", b"hello".to_vec())
        .await
        .assert_error(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type");
    upload(&app, HOST, "image/png", Vec::new())
        .await
        .assert_error(StatusCode::BAD_REQUEST, "bad_request");
    upload(&app, "none.example.com", "image/png", PNG.to_vec())
        .await
        .assert_error(StatusCode::NOT_FOUND, "not_found");
    assert!(!app.data().join("media").join("none.example.com").exists());
}

#[tokio::test]
async fn uploads_above_16_mib_are_too_large() {
    let app = TestApp::new().await;
    app.create_site().await;
    let mut big = PNG.to_vec();
    big.resize(16 * 1024 * 1024 + 1, 0);
    upload(&app, HOST, "image/png", big)
        .await
        .assert_error(StatusCode::PAYLOAD_TOO_LARGE, "payload_too_large");
    let mut max = PNG.to_vec();
    max.resize(3 * 1024 * 1024, 0);
    assert_eq!(
        upload(&app, HOST, "image/png", max).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn odd_media_names_never_reach_the_disk() {
    let app = TestApp::new().await;
    app.create_site().await;
    for name in [
        "..%2F..%2Fcuhlumn.db",
        "..%5C..%5Ccuhlumn.db",
        "cuhlumn.db",
        "ba7816bf8f01cfea414140de5dae2223.exe",
    ] {
        let res = app
            .api("DELETE", &format!("/api/sites/{HOST}/media/{name}"), None)
            .await;
        res.assert_error(StatusCode::BAD_REQUEST, "validation");
        assert_eq!(res.field(), "name");
    }
    assert!(app.data().join("cuhlumn.db").exists());
    let traversal = app
        .api(
            "DELETE",
            &format!("/api/sites/{HOST}/media/../../cuhlumn.db"),
            None,
        )
        .await;
    assert_eq!(traversal.json()["error"]["code"], "not_found");
}

#[tokio::test]
async fn deleting_a_site_removes_its_images() {
    let app = TestApp::new().await;
    app.create_site().await;
    upload(&app, HOST, "image/png", PNG.to_vec()).await;
    let folder = app.data().join("media").join(HOST);
    assert!(folder.exists());
    assert_eq!(
        app.api("DELETE", &format!("/api/sites/{HOST}"), None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert!(!folder.exists());
}

#[tokio::test]
async fn media_writes_invalidate_the_page_cache() {
    let app = TestApp::new().await;
    app.create_site().await;
    app.seed_cache();
    let saved = upload(&app, HOST, "image/png", PNG.to_vec()).await;
    assert_eq!(saved.status, StatusCode::OK);
    assert!(app.cache_is_cleared());
    app.seed_cache();
    let name = saved.json()["name"].as_str().unwrap().to_owned();
    let path = format!("/api/sites/{HOST}/media/{name}");
    assert_eq!(
        app.api("DELETE", &path, None).await.status,
        StatusCode::NO_CONTENT
    );
    assert!(app.cache_is_cleared());
}
