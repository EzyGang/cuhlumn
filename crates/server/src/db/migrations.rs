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
const MIGRATIONS: &[&str] = &[r"
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
"];
