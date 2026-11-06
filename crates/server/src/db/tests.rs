use cuhlumn_api::{
    post::{PostInput, Status, slug::Slug},
    site::{SiteInput, host::Host, indexnow_key::IndexNowKey},
};
use rusqlite::Connection;
use serde_json::json;

use crate::{
    config::Scheme,
    db::{Database, DbError, media, posts, sessions, sites},
    media::{format::ImageFormat, name::MediaName},
};

fn open() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    drop(Database::open(&path).unwrap());
    let connection = Connection::open(&path).unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    (dir, connection)
}

fn host(name: &str) -> Host {
    Host::new(name).unwrap()
}

fn site_input(aliases: &[&str]) -> SiteInput {
    serde_json::from_value(json!({"title": "Notes", "aliases": aliases})).unwrap()
}

fn post_input(status: &str, published_at: Option<i64>) -> PostInput {
    serde_json::from_value(
        json!({"title": "Hi", "body": "Text.", "status": status, "published_at": published_at}),
    )
    .unwrap()
}

fn key() -> IndexNowKey {
    IndexNowKey::new("0123456789abcdef0123456789abcdef").unwrap()
}

#[test]
fn migrations_are_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    drop(Database::open(&path).unwrap());
    drop(Database::open(&path).unwrap());
    let connection = Connection::open(&path).unwrap();
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 1);
}

#[test]
fn aliases_are_unique_and_resolve_to_their_site() {
    let (_dir, mut db) = open();
    let a = host("a.example.com");
    sites::put(
        &mut db,
        &a,
        &site_input(&["www.a.example.com", "a.example.com"]),
        &key(),
        1,
    )
    .unwrap();
    let site = sites::get(&db, &a).unwrap().unwrap();
    assert_eq!(site.aliases, [host("www.a.example.com")]);
    let resolved = sites::resolve(&db, &host("www.a.example.com"))
        .unwrap()
        .unwrap();
    assert_eq!(resolved.host, a);
    assert_eq!(
        sites::owner(&db, &host("www.a.example.com")).unwrap(),
        Some(a.clone())
    );
    let b = host("b.example.com");
    let taken = sites::put(&mut db, &b, &site_input(&["www.a.example.com"]), &key(), 1);
    assert!(matches!(taken, Err(DbError::Sqlite(_))));
    assert!(
        sites::get(&db, &b).unwrap().is_none(),
        "the failed transaction left a site"
    );
}

#[test]
fn indexnow_key_stays_without_input() {
    let (_dir, mut db) = open();
    let a = host("a.example.com");
    sites::put(&mut db, &a, &site_input(&[]), &key(), 1).unwrap();
    let other = IndexNowKey::new("ffffffffffffffff").unwrap();
    sites::put(&mut db, &a, &site_input(&[]), &other, 2).unwrap();
    assert_eq!(sites::get(&db, &a).unwrap().unwrap().indexnow_key, key());
}

#[test]
fn deleting_a_site_cascades() {
    let (_dir, mut db) = open();
    let a = host("a.example.com");
    sites::put(&mut db, &a, &site_input(&["www.a.example.com"]), &key(), 1).unwrap();
    let slug = Slug::new("hello").unwrap();
    posts::put(&db, Scheme::Https, &a, &slug, &post_input("draft", None), 1).unwrap();
    let name = MediaName::for_content(ImageFormat::Png, b"png");
    media::insert(&db, &a, &name, 3, 1).unwrap();
    assert!(sites::delete(&db, &a).unwrap());
    for table in ["site_aliases", "posts", "media"] {
        let count: i64 = db
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "{table} kept rows");
    }
    assert!(!sites::delete(&db, &a).unwrap());
}

#[test]
fn first_publication_time_stays() {
    let (_dir, mut db) = open();
    let a = host("a.example.com");
    sites::put(&mut db, &a, &site_input(&[]), &key(), 1).unwrap();
    let slug = Slug::new("hello").unwrap();
    let draft = posts::put(
        &db,
        Scheme::Https,
        &a,
        &slug,
        &post_input("draft", None),
        10,
    )
    .unwrap();
    assert_eq!(draft.published_at, None);
    assert_eq!(draft.url, "https://a.example.com/hello/");
    let first = posts::put(
        &db,
        Scheme::Https,
        &a,
        &slug,
        &post_input("published", None),
        20,
    )
    .unwrap();
    assert_eq!(
        (first.status, first.published_at),
        (Status::Published, Some(20))
    );
    let edit = posts::put(
        &db,
        Scheme::Https,
        &a,
        &slug,
        &post_input("published", None),
        30,
    )
    .unwrap();
    assert_eq!(edit.published_at, Some(20));
    let hidden = posts::put(
        &db,
        Scheme::Https,
        &a,
        &slug,
        &post_input("draft", None),
        40,
    )
    .unwrap();
    assert_eq!(hidden.published_at, Some(20));
    let backdated = posts::put(
        &db,
        Scheme::Https,
        &a,
        &slug,
        &post_input("published", Some(5)),
        50,
    )
    .unwrap();
    assert_eq!(backdated.published_at, Some(5));
    let visible = posts::list(&db, Scheme::Https, &a, posts::Visibility::Published).unwrap();
    assert_eq!(visible.len(), 1);
}

#[test]
fn expired_sessions_are_not_found() {
    let (_dir, db) = open();
    sessions::create(&db, "old", 100).unwrap();
    sessions::create(&db, "new", 300).unwrap();
    assert_eq!(sessions::find(&db, "old", 200).unwrap(), None);
    assert_eq!(sessions::find(&db, "new", 200).unwrap(), Some(300));
    assert_eq!(sessions::delete_expired(&db, 200).unwrap(), 1);
    assert!(sessions::delete(&db, "new").unwrap());
    assert!(!sessions::delete(&db, "new").unwrap());
}

#[test]
fn invalid_stored_values_are_errors() {
    let (_dir, mut db) = open();
    let a = host("a.example.com");
    sites::put(&mut db, &a, &site_input(&[]), &key(), 1).unwrap();
    db.execute("UPDATE sites SET links = 'not json'", [])
        .unwrap();
    assert!(sites::get(&db, &a).is_err());
    db.execute("UPDATE sites SET links = '[]', accent = 'red'", [])
        .unwrap();
    assert!(sites::list(&db).is_err());
}
