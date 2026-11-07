//! Shared helpers: a router on a temporary data folder and request shortcuts.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Request, StatusCode, request::Builder},
};
use cuhlumn_api::{settings::Scheme, site::host::Host};
use cuhlumn_server::{
    config::{Config, Token},
    db::unix_now,
    indexnow::IndexNow,
    router,
    state::AppState,
    web::cache::cached::Cached,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;

pub const TOKEN: &str = "test-token-0123456789-abcdefghijklmnop";
pub const CONSOLE: &str = "console.test";
pub const HOST: &str = "blog.example.com";
pub const ADMIN_HOST: &str = "admin.test";
pub const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDRtest-image";
/// An IndexNow endpoint on a closed local port, so tests never reach the network.
pub const OFFLINE_INDEXNOW: &str = "http://127.0.0.1:9/indexnow";

/// The overrides and the IndexNow endpoint of a test server.
#[derive(Clone, Debug)]
pub struct Setup {
    pub token: Option<&'static str>,
    pub scheme: Option<Scheme>,
    pub console_host: Option<&'static str>,
    pub indexnow: String,
}

impl Default for Setup {
    fn default() -> Self {
        Self {
            token: Some(TOKEN),
            scheme: Some(Scheme::Https),
            console_host: Some(CONSOLE),
            indexnow: OFFLINE_INDEXNOW.to_owned(),
        }
    }
}

impl Setup {
    /// A server without overrides, as on a first launch.
    pub fn fresh() -> Self {
        Self {
            token: None,
            scheme: None,
            console_host: None,
            ..Self::default()
        }
    }
}

pub struct TestApp {
    pub router: Router,
    pub state: AppState,
    dir: TempDir,
}

pub struct Res {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl TestApp {
    pub async fn new() -> Self {
        Self::open(Setup::default()).await
    }

    /// Builds an app without a token, before the first setup.
    pub async fn without_token() -> Self {
        Self::open(Setup {
            token: None,
            ..Setup::default()
        })
        .await
    }

    /// Builds an app whose public addresses use `http`.
    pub async fn http() -> Self {
        Self::open(Setup {
            scheme: Some(Scheme::Http),
            ..Setup::default()
        })
        .await
    }

    pub async fn open(setup: Setup) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let state = open_state(dir.path(), &setup).await;
        Self {
            router: router(state.clone()),
            state,
            dir,
        }
    }

    /// Opens a second state on the same data folder with another token and returns its router.
    pub async fn reopen(&self, token: &'static str) -> Router {
        let setup = Setup {
            token: Some(token),
            ..Setup::default()
        };
        router(open_state(self.dir.path(), &setup).await)
    }

    /// Opens a second state on the same data folder with `setup`.
    pub async fn reopen_with(&self, setup: &Setup) -> AppState {
        open_state(self.dir.path(), setup).await
    }

    pub fn data(&self) -> PathBuf {
        self.dir.path().to_owned()
    }

    /// Stores a cached page for `HOST`, so a test can check that a write drops it.
    pub fn seed_cache(&self) {
        let host = Host::new(HOST).unwrap();
        let cache = self.state.cache();
        let generation = cache.generation(&host);
        let page = Arc::new(Cached::new("text/plain", "cached", 60));
        cache.insert(&host, "/".to_owned(), generation, page);
        assert!(cache.get(&host, "/", unix_now()).is_some());
    }

    /// Returns `true` when the cached page of `HOST` is gone.
    pub fn cache_is_cleared(&self) -> bool {
        self.state
            .cache()
            .get(&Host::new(HOST).unwrap(), "/", unix_now())
            .is_none()
    }

    pub async fn send(&self, request: Request<Body>) -> Res {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec();
        Res {
            status,
            headers,
            body,
        }
    }

    /// Sends an API request with the bearer token and an optional JSON body.
    pub async fn api(&self, method: &str, path: &str, body: Option<Value>) -> Res {
        let builder =
            request(method, ADMIN_HOST, path).header("authorization", format!("Bearer {TOKEN}"));
        self.send(json_body(builder, body)).await
    }

    /// Creates the site `HOST` with an alias and returns the response body.
    pub async fn create_site(&self) -> Value {
        let site = json!({"title": "Harbor Notes", "aliases": ["www.blog.example.com"]});
        let res = self
            .api("PUT", &format!("/api/sites/{HOST}"), Some(site))
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        res.json()
    }
}

impl Res {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or_else(|err| panic!("{err}: {}", self.text()))
    }

    /// Checks the status and the `ErrorBody` code of a failure.
    pub fn assert_error(&self, status: StatusCode, code: &str) {
        assert_eq!(self.status, status, "{}", self.text());
        let body = self.json();
        assert_eq!(body["error"]["code"], code, "{body}");
        assert!(body["error"]["message"].is_string(), "{body}");
    }

    pub fn field(&self) -> Value {
        self.json()["error"]["field"].clone()
    }

    pub fn header(&self, name: &str) -> &str {
        self.headers
            .get(name)
            .map_or("", |value| value.to_str().unwrap())
    }
}

pub fn request(method: &str, host: &str, path: &str) -> Builder {
    Request::builder()
        .method(method)
        .uri(path)
        .header("host", host)
}

pub fn json_body(builder: Builder, body: Option<Value>) -> Request<Body> {
    match body {
        Some(body) => builder
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn open_state(data: &Path, setup: &Setup) -> AppState {
    let config = Config {
        port: 0,
        data: data.to_owned(),
        token: setup.token.map(|token| Token::new(token).unwrap()),
        console_host: setup.console_host.map(|host| host.parse().unwrap()),
        scheme: setup.scheme,
        command: None,
    };
    let indexnow = IndexNow::with_endpoint(setup.indexnow.clone()).unwrap();
    AppState::open_with(config, indexnow).await.unwrap()
}

/// The Markdown body of the published post `sqlite-backups` in [`TestApp::seed_blog`].
pub const POST_BODY: &str = "We back up **one SQLite file** every night.\n\n## Why one file\n\nIt is easy to copy.\n\n## How the backup runs\n\nA timer starts it at 03:00.\n\n### The command\n\n```sh\nsqlite3 app.db \".backup backup.db\"\n```\n\n## Why one file\n\nAgain.\n";

/// The site of [`TestApp::seed_blog`].
pub fn blog_site() -> Value {
    json!({
        "title": "Harbor Notes",
        "description": "How a two-person team runs Harbor, a small hosting service.",
        "aliases": ["www.blog.example.com"],
        "project_name": "Harbor",
        "project_url": "https://harbor.example.com",
        "cta": "Harbor runs your app on a server that you own.",
        "author": "Mara Lind",
        "author_url": "https://mara.example.com",
        "brand": {"accent": "#0e7490"},
    })
}

impl TestApp {
    /// Creates the blog `HOST` with the published post `sqlite-backups`, the draft `unreleased`, and the page `about`.
    pub async fn seed_blog(&self) -> Value {
        let site = self.put_site(blog_site()).await;
        assert_eq!(site.status, StatusCode::OK, "{}", site.text());
        let posts = [
            (
                "sqlite-backups",
                json!({"title": "How we back up SQLite", "body": POST_BODY,
                       "tags": ["Databases", "Operations"], "status": "published"}),
            ),
            (
                "unreleased",
                json!({"title": "Unreleased plans", "body": "Not yet.", "tags": ["Databases"]}),
            ),
            (
                "about",
                json!({"title": "About", "body": "A short note about the team.", "kind": "page", "status": "published"}),
            ),
        ];
        for (slug, post) in posts {
            let res = self.put_post(slug, post).await;
            assert_eq!(res.status, StatusCode::OK, "{}", res.text());
        }
        site.json()
    }

    /// Creates or replaces the site `HOST`.
    pub async fn put_site(&self, site: Value) -> Res {
        self.api("PUT", &format!("/api/sites/{HOST}"), Some(site))
            .await
    }

    /// Creates or replaces a post of the site `HOST`.
    pub async fn put_post(&self, slug: &str, post: Value) -> Res {
        self.api(
            "PUT",
            &format!("/api/sites/{HOST}/posts/{slug}"),
            Some(post),
        )
        .await
    }

    /// Sends a public GET request to `HOST`.
    pub async fn get(&self, path: &str) -> Res {
        self.get_on(HOST, path, &[]).await
    }

    /// Sends a public GET request to `host` with extra headers.
    pub async fn get_on(&self, host: &str, path: &str, headers: &[(&str, &str)]) -> Res {
        let mut builder = request("GET", host, path);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        self.send(builder.body(Body::empty()).unwrap()).await
    }
}
