//! Media record queries.

use cuhlumn_api::{settings::Scheme, site::host::Host, wire::MediaItem};
use rusqlite::{Connection, Row, params};

use crate::{
    db::{DbError, conversion, parse},
    media::{format::ImageFormat, name::MediaName},
};

/// Records an uploaded image. A repeated upload keeps the first record.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn insert(
    connection: &Connection,
    host: &Host,
    name: &MediaName,
    size: i64,
    now: i64,
) -> Result<(), DbError> {
    connection.execute(
        "INSERT INTO media (host, name, content_type, size, created_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (host, name) DO NOTHING",
        params![
            host.as_str(),
            name.as_str(),
            name.format().content_type(),
            size,
            now
        ],
    )?;
    Ok(())
}

/// Returns the images of the site at `host`, newest first.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails or a stored value is invalid.
pub fn list(
    connection: &Connection,
    scheme: Scheme,
    host: &Host,
) -> Result<Vec<MediaItem>, DbError> {
    let items = connection
        .prepare_cached(
            "SELECT name, size, created_at FROM media WHERE host = ?1
             ORDER BY created_at DESC, name",
        )?
        .query_map([host.as_str()], |row| item_row(row, scheme, host))?
        .collect::<Result<_, _>>()?;
    Ok(items)
}

/// Deletes one image record. Returns `false` when the record does not exist.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn delete(connection: &Connection, host: &Host, name: &MediaName) -> Result<bool, DbError> {
    let deleted = connection.execute(
        "DELETE FROM media WHERE host = ?1 AND name = ?2",
        [host.as_str(), name.as_str()],
    )?;
    Ok(deleted > 0)
}

fn item_row(row: &Row<'_>, scheme: Scheme, host: &Host) -> rusqlite::Result<MediaItem> {
    let name: MediaName = parse(row, "name")?;
    let format: ImageFormat = name.format();
    let size: i64 = row.get("size")?;
    let size = u64::try_from(size).map_err(|err| conversion(row, "size", err))?;
    Ok(MediaItem {
        url: name.url(scheme, host),
        content_type: format.content_type().to_owned(),
        size,
        created_at: row.get("created_at")?,
        name: name.into(),
    })
}
