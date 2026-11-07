//! Font library records.

use cuhlumn_api::fonts::FontItem;
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::db::{DbError, conversion, parse, parse_json};

/// Records a library font, or replaces the record with the same name.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn insert(connection: &Connection, font: &FontItem) -> Result<(), DbError> {
    let families = serde_json::to_string(&font.families)?;
    let size = i64::try_from(font.size).unwrap_or(i64::MAX);
    connection.execute(
        "INSERT INTO fonts (name, families, source, size, created_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (name) DO UPDATE SET families = excluded.families,
           source = excluded.source, size = excluded.size, created_at = excluded.created_at",
        params![
            font.name,
            families,
            font.source.as_str(),
            size,
            font.created_at
        ],
    )?;
    Ok(())
}

/// Returns every library font, ordered by name.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails or a stored value is invalid.
pub fn list(connection: &Connection) -> Result<Vec<FontItem>, DbError> {
    let fonts = connection
        .prepare(&format!("{SELECT} ORDER BY name"))?
        .query_map([], font_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(fonts)
}

/// Returns the library font with the file name `name`.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails or a stored value is invalid.
pub fn get(connection: &Connection, name: &str) -> Result<Option<FontItem>, DbError> {
    let font = connection
        .query_row(&format!("{SELECT} WHERE name = ?1"), [name], font_row)
        .optional()?;
    Ok(font)
}

/// Deletes the record of a library font. Returns `false` when it does not exist.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn delete(connection: &Connection, name: &str) -> Result<bool, DbError> {
    Ok(connection.execute("DELETE FROM fonts WHERE name = ?1", [name])? > 0)
}

const SELECT: &str = "SELECT name, families, source, size, created_at FROM fonts";

fn font_row(row: &Row<'_>) -> rusqlite::Result<FontItem> {
    let size: i64 = row.get("size")?;
    Ok(FontItem {
        name: row.get("name")?,
        families: parse_json(row, "families")?,
        source: parse(row, "source")?,
        size: u64::try_from(size).map_err(|err| conversion(row, "size", err))?,
        created_at: row.get("created_at")?,
    })
}
