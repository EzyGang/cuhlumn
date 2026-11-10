//! Ordered schema migrations, tracked by `PRAGMA user_version`.

use rusqlite::Connection;

/// Applies every migration that the database does not have yet.
pub(crate) fn apply(connection: &mut Connection) -> rusqlite::Result<()> {
    let applied: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (version, sql) in (1_i64..).zip(MIGRATIONS) {
        if version <= applied {
            continue;
        }
        let tx = connection.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
        tracing::info!(version, "applied database migration");
    }
    Ok(())
}

/// The migrations in order. Never change a released migration; add a new one.
pub(super) const MIGRATIONS: &[&str] = &[
    r"
CREATE TABLE sites (
  host TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  author TEXT NOT NULL,
  author_url TEXT,
  project_name TEXT NOT NULL,
  project_url TEXT,
  cta TEXT NOT NULL,
  lang TEXT NOT NULL,
  home TEXT,
  indexnow_key TEXT NOT NULL,
  accent TEXT NOT NULL,
  font TEXT NOT NULL,
  logo TEXT,
  favicon TEXT,
  og_image TEXT,
  links TEXT NOT NULL,
  head TEXT NOT NULL,
  footer TEXT NOT NULL,
  credit INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
) STRICT;

CREATE TABLE site_aliases (
  alias TEXT PRIMARY KEY,
  host TEXT NOT NULL REFERENCES sites (host) ON DELETE CASCADE
) STRICT;
CREATE INDEX site_aliases_host ON site_aliases (host);

CREATE TABLE posts (
  host TEXT NOT NULL REFERENCES sites (host) ON DELETE CASCADE,
  slug TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  body TEXT NOT NULL,
  tags TEXT NOT NULL,
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  cover TEXT,
  og_image TEXT,
  published_at INTEGER,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (host, slug)
) STRICT;

CREATE TABLE media (
  host TEXT NOT NULL REFERENCES sites (host) ON DELETE CASCADE,
  name TEXT NOT NULL,
  content_type TEXT NOT NULL,
  size INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (host, name)
) STRICT;

CREATE TABLE sessions (
  id_hash TEXT PRIMARY KEY,
  expires_at INTEGER NOT NULL
) STRICT;
",
    r"
ALTER TABLE sites ADD COLUMN palette TEXT NOT NULL DEFAULT 'paper';
ALTER TABLE sites ADD COLUMN theme TEXT NOT NULL DEFAULT 'system';
ALTER TABLE sites ADD COLUMN theme_toggle INTEGER NOT NULL DEFAULT 1;
ALTER TABLE sites ADD COLUMN width TEXT NOT NULL DEFAULT 'normal';
ALTER TABLE sites ADD COLUMN home_layout TEXT NOT NULL DEFAULT 'list';
ALTER TABLE sites ADD COLUMN social TEXT NOT NULL DEFAULT '[]';
ALTER TABLE sites ADD COLUMN social_in_header INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sites ADD COLUMN code_theme TEXT NOT NULL DEFAULT 'github';
ALTER TABLE sites ADD COLUMN card_style TEXT NOT NULL DEFAULT 'dark';
ALTER TABLE sites ADD COLUMN card_meta INTEGER NOT NULL DEFAULT 1;
ALTER TABLE sites ADD COLUMN indexnow INTEGER NOT NULL DEFAULT 1;
ALTER TABLE sites ADD COLUMN announcement TEXT;

CREATE TABLE settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  token_digest TEXT,
  console_host TEXT,
  scheme TEXT,
  updated_at INTEGER NOT NULL
) STRICT;
INSERT INTO settings (id, updated_at) VALUES (1, CAST(strftime('%s', 'now') AS INTEGER));

CREATE TABLE fonts (
  name TEXT PRIMARY KEY,
  families TEXT NOT NULL,
  source TEXT NOT NULL,
  size INTEGER NOT NULL,
  created_at INTEGER NOT NULL
) STRICT;
",
    r"
ALTER TABLE sites ADD COLUMN show_cta INTEGER NOT NULL DEFAULT 1;
ALTER TABLE posts ADD COLUMN show_cta INTEGER;
",
    r"
ALTER TABLE sites ADD COLUMN font_family TEXT;
",
    r"
CREATE TABLE analytics_days (
  host TEXT NOT NULL REFERENCES sites (host) ON DELETE CASCADE,
  day INTEGER NOT NULL, -- days since 1970-01-01 (UTC)
  views INTEGER NOT NULL DEFAULT 0,
  visitors INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (host, day)
) STRICT, WITHOUT ROWID;

CREATE TABLE analytics_pages (
  host TEXT NOT NULL REFERENCES sites (host) ON DELETE CASCADE,
  day INTEGER NOT NULL,
  path TEXT NOT NULL,
  views INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (host, day, path)
) STRICT, WITHOUT ROWID;

CREATE TABLE analytics_referrers (
  host TEXT NOT NULL REFERENCES sites (host) ON DELETE CASCADE,
  day INTEGER NOT NULL,
  referrer TEXT NOT NULL, -- '' means direct
  views INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (host, day, referrer)
) STRICT, WITHOUT ROWID;

CREATE TABLE analytics_clients (
  host TEXT NOT NULL REFERENCES sites (host) ON DELETE CASCADE,
  day INTEGER NOT NULL,
  device TEXT NOT NULL,
  browser TEXT NOT NULL,
  views INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (host, day, device, browser)
) STRICT, WITHOUT ROWID;

-- The hashes of the current UTC day only.
CREATE TABLE analytics_visitors (
  host TEXT NOT NULL REFERENCES sites (host) ON DELETE CASCADE,
  day INTEGER NOT NULL,
  hash BLOB NOT NULL,
  PRIMARY KEY (host, day, hash)
) STRICT, WITHOUT ROWID;

CREATE TABLE analytics_salt (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  day INTEGER NOT NULL,
  salt BLOB NOT NULL
) STRICT;
",
];
