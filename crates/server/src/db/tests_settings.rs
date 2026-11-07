use cuhlumn_api::{
    fonts::{FontItem, FontSource},
    settings::Scheme,
    site::{SiteInput, brand::Brand, host::Host, indexnow_key::IndexNowKey, theme::Palette},
};
use rusqlite::Connection;
use serde_json::json;

use crate::db::{Database, fonts, migrations, sessions, settings, sites};

fn open() -> (tempfile::TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    drop(Database::open(&path).unwrap());
    let connection = Connection::open(&path).unwrap();
    (dir, connection)
}

fn harbor() -> Host {
    Host::new("harbor.example.com").unwrap()
}

fn key() -> IndexNowKey {
    IndexNowKey::new("0123456789abcdef0123456789abcdef").unwrap()
}

#[test]
fn new_site_fields_round_trip() {
    let (_dir, mut db) = open();
    let input: SiteInput = serde_json::from_value(json!({
        "title": "Harbor Notes",
        "indexnow": false,
        "announcement": {
            "text": "How we back up SQLite",
            "link": "/sqlite-backups/",
            "tone": "accent",
            "starts_at": 10,
            "ends_at": 20,
            "dismissible": false
        },
        "brand": {
            "accent": "#0e7490",
            "palette": "sepia",
            "theme": "dark",
            "theme_toggle": false,
            "width": "wide",
            "home_layout": "cards",
            "social": [{"network": "github", "url": "https://github.com/maralind"}],
            "social_in_header": true,
            "code_theme": "none",
            "card_style": "accent",
            "card_meta": false
        }
    }))
    .unwrap();
    sites::put(&mut db, &harbor(), &input, &key(), 1).unwrap();
    let site = sites::get(&db, &harbor()).unwrap().unwrap();
    assert_eq!(site.brand, input.brand);
    assert_eq!(site.announcement, input.announcement);
    assert!(!site.indexnow);
    let cleared = SiteInput {
        announcement: None,
        ..SiteInput::from(site)
    };
    sites::put(&mut db, &harbor(), &cleared, &key(), 2).unwrap();
    let site = sites::get(&db, &harbor()).unwrap().unwrap();
    assert_eq!(site.announcement, None);
}

#[test]
fn migration_two_fills_defaults_for_existing_sites() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old.db");
    let mut db = Connection::open(&path).unwrap();
    db.execute_batch(migrations::MIGRATIONS[0]).unwrap();
    db.pragma_update(None, "user_version", 1).unwrap();
    db.execute(
        "INSERT INTO sites (host, title, description, author, project_name, cta, lang,
           indexnow_key, accent, font, links, head, footer, credit, updated_at)
         VALUES ('harbor.example.com', 'Harbor Notes', '', 'Mara Lind', '', '', 'en',
           '0123456789abcdef0123456789abcdef', '#0e7490', 'serif', '[]', '', '', 1, 1)",
        [],
    )
    .unwrap();
    migrations::apply(&mut db).unwrap();
    let site = sites::get(&db, &harbor()).unwrap().unwrap();
    let expected = Brand {
        accent: site.brand.accent.clone(),
        ..Brand::default()
    };
    assert_eq!(site.brand, expected);
    assert_eq!(site.brand.palette, Palette::Paper);
    assert!(site.indexnow);
    assert_eq!(site.announcement, None);
    let stored = settings::get(&db).unwrap();
    assert_eq!(stored.token_digest, None);
    assert_eq!(stored.scheme, None);
}

#[test]
fn first_token_wins() {
    let (_dir, mut db) = open();
    assert!(settings::set_token_if_empty(&db, "first", 1).unwrap());
    assert!(!settings::set_token_if_empty(&db, "second", 2).unwrap());
    assert_eq!(
        settings::get(&db).unwrap().token_digest.as_deref(),
        Some("first")
    );
    sessions::create(&db, "hash", 100).unwrap();
    settings::replace_token(&mut db, "third", 3).unwrap();
    let stored = settings::get(&db).unwrap();
    assert_eq!(stored.token_digest.as_deref(), Some("third"));
    assert_eq!(stored.updated_at, 3);
    assert_eq!(sessions::find(&db, "hash", 0).unwrap(), None);
}

#[test]
fn server_settings_are_stored() {
    let (_dir, db) = open();
    settings::set_server(&db, Some(&harbor()), Scheme::Http, 5).unwrap();
    let stored = settings::get(&db).unwrap();
    assert_eq!(stored.console_host, Some(harbor()));
    assert_eq!(stored.scheme, Some(Scheme::Http));
    settings::set_server(&db, None, Scheme::Https, 6).unwrap();
    assert_eq!(settings::get(&db).unwrap().console_host, None);
}

#[test]
fn fonts_are_listed_and_deleted() {
    let (_dir, db) = open();
    let font = FontItem {
        name: "noto-sans-jp.ttf".to_owned(),
        families: vec!["Noto Sans JP".to_owned()],
        source: FontSource::Google,
        size: 1024,
        created_at: 7,
    };
    fonts::insert(&db, &font).unwrap();
    assert_eq!(fonts::list(&db).unwrap(), std::slice::from_ref(&font));
    assert_eq!(fonts::get(&db, &font.name).unwrap(), Some(font.clone()));
    assert!(fonts::delete(&db, &font.name).unwrap());
    assert!(!fonts::delete(&db, &font.name).unwrap());
    assert_eq!(fonts::get(&db, &font.name).unwrap(), None);
}
