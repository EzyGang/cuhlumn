//! Runs the `cuhlumn` binary against a real server on a local port.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "tests fail loudly on unexpected values"
)]

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

use cuhlumn_server::{
    config::{Config, Token},
    indexnow::IndexNow,
    router,
    state::AppState,
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::{net::TcpListener, runtime::Runtime};

const TOKEN: &str = "cli-test-token-0123456789-abcdefghijk";
const SITE: &str = "blog.example.com";
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDRtest-image";

/// A server on a local port with its own data folder.
struct Server {
    url: String,
    files: TempDir,
    _data: TempDir,
    _runtime: Runtime,
}

/// The result of one CLI run.
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Server {
    fn start() -> Self {
        let data = tempfile::tempdir().unwrap();
        let runtime = Runtime::new().unwrap();
        let config = Config {
            port: 0,
            admin_addr: None,
            data: data.path().to_owned(),
            token: Some(Token::new(TOKEN).unwrap()),
            console_host: None,
            scheme: None,
            command: None,
        };
        let indexnow = IndexNow::with_endpoint("http://127.0.0.1:9/indexnow".to_owned()).unwrap();
        let state = runtime
            .block_on(AppState::open_with(config, indexnow))
            .unwrap();
        let listener = runtime.block_on(TcpListener::bind("127.0.0.1:0")).unwrap();
        let address = listener.local_addr().unwrap();
        runtime.spawn(async move { axum::serve(listener, router(state)).await });
        Self {
            url: format!("http://{address}"),
            files: tempfile::tempdir().unwrap(),
            _data: data,
            _runtime: runtime,
        }
    }

    /// Runs `cuhlumn` with the server, the token, and the site in its environment.
    fn run(&self, args: &[&str]) -> Run {
        self.run_with(
            args,
            &[("CUHLUMN_TOKEN", TOKEN), ("CUHLUMN_SITE", SITE)],
            None,
        )
    }

    fn run_with(&self, args: &[&str], env: &[(&str, &str)], stdin: Option<&str>) -> Run {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cuhlumn"));
        command
            .args(args)
            .current_dir(self.files.path())
            .env_remove("CUHLUMN_TOKEN")
            .env_remove("CUHLUMN_SITE")
            .env("CUHLUMN_SERVER", &self.url)
            .envs(env.iter().copied())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().unwrap();
        let mut input = child.stdin.take().unwrap();
        input
            .write_all(stdin.unwrap_or_default().as_bytes())
            .unwrap();
        drop(input);
        let Output {
            status,
            stdout,
            stderr,
        } = child.wait_with_output().unwrap();
        Run {
            code: status.code().unwrap(),
            stdout: String::from_utf8(stdout).unwrap(),
            stderr: String::from_utf8(stderr).unwrap(),
        }
    }

    /// Runs a command that must succeed and returns its JSON output.
    fn json(&self, args: &[&str]) -> Value {
        let run = self.run(args);
        assert_eq!(run.code, 0, "{args:?} failed: {}", run.stderr);
        serde_json::from_str(&run.stdout).unwrap()
    }

    fn write(&self, name: &str, contents: &[u8]) {
        fs::write(self.files.path().join(name), contents).unwrap();
    }

    fn path(&self, name: &str) -> String {
        self.files.path().join(name).display().to_string()
    }

    fn create_site(&self) {
        let run = self.run_with(
            &["site", "apply", "-"],
            &[("CUHLUMN_TOKEN", TOKEN), ("CUHLUMN_SITE", SITE)],
            Some(r##"{"title": "Harbor Notes", "description": "Notes.", "brand": {"accent": "#0e7490"}}"##),
        );
        assert_eq!(run.code, 0, "{}", run.stderr);
    }
}

fn post_file(status: &str) -> String {
    format!(
        "---\ntitle: How we back up SQLite\ndescription: Why one file.\ntags: [Ops]\n{status}---\n\nWe copy the file.\n\n## Why\n\nIt is easy to copy.\n"
    )
}

#[test]
fn site_post_and_media_round_trip() {
    let server = Server::start();
    server.create_site();

    server.write("logo.png", PNG);
    let upload = server.json(&["media", "upload", &server.path("logo.png")]);
    let path = upload["path"].as_str().unwrap();
    assert!(
        path.starts_with("/media/") && path.ends_with(".png"),
        "{upload}"
    );
    let site = server.json(&["site", "set", &format!("brand.logo={path}"), "cta=Try it"]);
    assert_eq!(site["brand"]["logo"], json!(path));
    assert_eq!(site["brand"]["accent"], json!("#0e7490"));
    assert_eq!(site["description"], json!("Notes."));

    server.write("sqlite-backups.md", post_file("").as_bytes());
    let saved = server.json(&["post", "push", "sqlite-backups.md"]);
    assert_eq!(saved["post"]["slug"], json!("sqlite-backups"));
    assert_eq!(saved["post"]["status"], json!("draft"));
    let drafts = server.json(&["post", "list", "--status", "draft"]);
    assert_eq!(drafts.as_array().unwrap().len(), 1);
    assert!(drafts[0].get("body").is_none(), "{drafts}");

    let published = server.json(&["post", "publish", "sqlite-backups"]);
    assert_eq!(
        published["url"],
        json!("https://blog.example.com/sqlite-backups/")
    );
    let first_time = published["post"]["published_at"].clone();
    assert!(first_time.is_i64());

    let markdown = server.run(&["post", "get", "sqlite-backups", "--markdown"]);
    assert_eq!(markdown.code, 0, "{}", markdown.stderr);
    server.write("copy.md", markdown.stdout.as_bytes());
    let again = server.json(&["post", "push", "copy.md"]);
    assert_eq!(again["post"]["slug"], json!("sqlite-backups"));
    assert_eq!(again["post"]["status"], json!("published"));
    assert_eq!(again["post"]["published_at"], first_time);
    assert_eq!(again["post"]["tags"], json!(["Ops"]));

    let preview = server.json(&["post", "preview", "sqlite-backups.md"]);
    assert_eq!(preview["toc"][0]["label"], json!("Why"));
    assert!(
        preview["html"]
            .as_str()
            .unwrap()
            .contains("<p>We copy the file.</p>")
    );

    let input = server.json(&["site", "get", "--input"]);
    assert!(input.get("host").is_none() && input.get("updated_at").is_none());
    assert_eq!(
        input["indexnow_key"],
        server.json(&["site", "get"])["indexnow_key"]
    );
}

#[test]
fn push_does_not_unpublish_without_draft_flag() {
    let server = Server::start();
    server.create_site();
    server.write(
        "sqlite-backups.md",
        post_file("status: published\n").as_bytes(),
    );
    server.json(&["post", "push", "sqlite-backups.md"]);

    server.write("sqlite-backups.md", post_file("").as_bytes());
    let refused = server.run(&["post", "push", "sqlite-backups.md"]);
    assert_eq!(refused.code, 1);
    assert!(
        refused.stderr.contains("sqlite-backups is published"),
        "{}",
        refused.stderr
    );
    assert_eq!(
        server.json(&["post", "get", "sqlite-backups"])["status"],
        json!("published")
    );

    let kept = server.json(&["post", "push", "sqlite-backups.md", "--publish"]);
    assert_eq!(kept["post"]["status"], json!("published"));
    let hidden = server.json(&["post", "push", "sqlite-backups.md", "--draft"]);
    assert_eq!(hidden["post"]["status"], json!("draft"));
}

#[test]
fn invalid_input_is_rejected_before_the_request() {
    let server = Server::start();
    server.create_site();

    let accent = server.run(&["site", "set", "brand.accent=red"]);
    assert_eq!(accent.code, 1);
    assert!(accent.stderr.contains("brand.accent"), "{}", accent.stderr);
    let unknown = server.run(&["site", "set", "brand.colour=red"]);
    assert_eq!(unknown.code, 1);
    assert!(unknown.stderr.contains("colour"), "{}", unknown.stderr);
    assert_eq!(
        server.json(&["site", "get"])["brand"]["accent"],
        json!("#0e7490")
    );

    server.write("Bad Name.md", b"---\ntitle: Hi\n---\nText.");
    let slug = server.run(&["post", "push", "Bad Name.md"]);
    assert_eq!(slug.code, 1);
    assert!(slug.stderr.contains("slug field"), "{}", slug.stderr);

    server.write("notes.txt", b"text");
    let media = server.run(&["media", "upload", "notes.txt"]);
    assert_eq!(media.code, 1);
    assert!(media.stderr.contains(".png"), "{}", media.stderr);
}

#[test]
fn server_errors_show_their_code() {
    let server = Server::start();
    server.create_site();

    let missing = server.run(&["post", "get", "nothing"]);
    assert_eq!(missing.code, 1);
    assert!(
        missing.stderr.starts_with("error: not_found:"),
        "{}",
        missing.stderr
    );

    let wrong = server.run_with(
        &["site", "list"],
        &[("CUHLUMN_TOKEN", "wrong-token-0123456789-abcdefghijklm")],
        None,
    );
    assert_eq!(wrong.code, 1);
    assert!(wrong.stderr.contains("unauthorized"), "{}", wrong.stderr);
}

#[test]
fn missing_settings_and_confirmations_stop_the_command() {
    let server = Server::start();
    server.create_site();

    let no_token = server.run_with(&["site", "list"], &[], None);
    assert_eq!(no_token.code, 1);
    assert!(
        no_token.stderr.contains("CUHLUMN_TOKEN"),
        "{}",
        no_token.stderr
    );
    let health = server.run_with(&["health"], &[], None);
    assert_eq!(health.code, 0, "{}", health.stderr);

    let no_site = server.run_with(&["post", "list"], &[("CUHLUMN_TOKEN", TOKEN)], None);
    assert_eq!(no_site.code, 1);
    assert!(
        no_site.stderr.contains("CUHLUMN_SITE"),
        "{}",
        no_site.stderr
    );

    let unconfirmed = server.run(&["site", "delete"]);
    assert_eq!(unconfirmed.code, 1);
    assert!(
        unconfirmed.stderr.contains("--yes"),
        "{}",
        unconfirmed.stderr
    );
    assert_eq!(server.json(&["site", "list"]).as_array().unwrap().len(), 1);

    let deleted = server.run(&["site", "delete", "--yes"]);
    assert_eq!(deleted.code, 0, "{}", deleted.stderr);
    assert_eq!(server.json(&["site", "list"]), json!([]));
}

#[test]
fn files_resolve_relative_to_the_working_folder() {
    let server = Server::start();
    server.create_site();
    let nested = Path::new("posts");
    fs::create_dir(server.files.path().join(nested)).unwrap();
    server.write("posts/hello.md", post_file("").as_bytes());
    let saved = server.json(&["post", "push", "posts/hello.md"]);
    assert_eq!(saved["post"]["slug"], json!("hello"));
}

#[test]
fn announce_set_show_and_clear() {
    let server = Server::start();
    server.create_site();
    assert_eq!(
        server.json(&["announce", "show"]),
        json!({"status": "off", "announcement": null})
    );

    let set = server.json(&[
        "announce",
        "set",
        "  Maintenance on Sunday.  ",
        "--link",
        "https://harbor.example.com/maintenance",
        "--label",
        "Details",
        "--tone",
        "warning",
        "--from",
        "2026-01-01T10:00:00+02:00",
        "--until",
        "4102444800",
        "--no-dismiss",
    ]);
    let expected = json!({
        "status": "live",
        "announcement": {
            "text": "Maintenance on Sunday.",
            "link": "https://harbor.example.com/maintenance",
            "link_label": "Details",
            "tone": "warning",
            "starts_at": 1_767_254_400,
            "ends_at": 4_102_444_800_i64,
            "dismissible": false,
        },
    });
    assert_eq!(set, expected);
    assert_eq!(server.json(&["announce", "show"]), expected);
    assert_eq!(
        server.json(&["site", "get"])["announcement"],
        expected["announcement"]
    );

    let scheduled = server.json(&["announce", "set", "Soon.", "--from", "4102444800"]);
    assert_eq!(scheduled["status"], json!("scheduled"));
    assert_eq!(scheduled["announcement"]["link_label"], json!("Learn more"));
    assert_eq!(scheduled["announcement"]["tone"], json!("info"));
    assert_eq!(scheduled["announcement"]["dismissible"], json!(true));

    let ended = server.json(&["announce", "set", "Done.", "--until", "1000"]);
    assert_eq!(ended["status"], json!("ended"));

    let cleared = server.json(&["announce", "clear"]);
    assert_eq!(cleared, json!({"status": "off", "announcement": null}));
    assert_eq!(server.json(&["site", "get"])["announcement"], json!(null));
}

#[test]
fn announce_rejects_invalid_input() {
    let server = Server::start();
    server.create_site();

    let time = server.run(&["announce", "set", "Hello.", "--from", "tomorrow"]);
    assert_eq!(time.code, 2);
    assert!(time.stderr.contains("RFC 3339"), "{}", time.stderr);

    let order = server.run(&[
        "announce",
        "set",
        "Hello.",
        "--from",
        "2026-11-10T09:00:00Z",
        "--until",
        "2026-11-09T09:00:00Z",
    ]);
    assert_eq!(order.code, 1);
    assert!(order.stderr.contains("ends_at"), "{}", order.stderr);

    let long = "x".repeat(161);
    let text = server.run(&["announce", "set", &long]);
    assert_eq!(text.code, 1);
    assert!(text.stderr.contains("text"), "{}", text.stderr);
    assert_eq!(server.json(&["site", "get"])["announcement"], json!(null));
}
