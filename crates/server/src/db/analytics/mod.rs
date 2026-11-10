//! Page-view analytics: daily counts of each site, the visitor hashes of today, and the visitor salt.

pub mod check;
pub mod report;
pub mod write;

use rusqlite::{Connection, OptionalExtension};

use crate::db::DbError;

/// The stored visitor salt and its day.
#[derive(Clone, PartialEq, Eq)]
pub struct StoredSalt {
    /// The day, in days since 1970-01-01 (UTC).
    pub day: i64,
    pub bytes: Vec<u8>,
}

/// Returns the stored visitor salt, or `None` before the first write.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn salt(connection: &Connection) -> Result<Option<StoredSalt>, DbError> {
    let salt = connection
        .query_row(
            "SELECT day, salt FROM analytics_salt WHERE id = 1",
            [],
            |row| {
                Ok(StoredSalt {
                    day: row.get("day")?,
                    bytes: row.get("salt")?,
                })
            },
        )
        .optional()?;
    Ok(salt)
}
