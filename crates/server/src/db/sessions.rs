//! Console session queries.
//!
//! The table stores the SHA-256 hex digest of each session identifier, never the identifier.

use rusqlite::{Connection, OptionalExtension, params};

use crate::db::DbError;

/// Stores a session that expires at `expires_at`.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn create(connection: &Connection, id_hash: &str, expires_at: i64) -> Result<(), DbError> {
    connection.execute(
        "INSERT INTO sessions (id_hash, expires_at) VALUES (?1, ?2)",
        params![id_hash, expires_at],
    )?;
    Ok(())
}

/// Returns the expiry time of a session that has not expired at `now`.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn find(connection: &Connection, id_hash: &str, now: i64) -> Result<Option<i64>, DbError> {
    let expires_at = connection
        .query_row(
            "SELECT expires_at FROM sessions WHERE id_hash = ?1 AND expires_at > ?2",
            params![id_hash, now],
            |row| row.get(0),
        )
        .optional()?;
    Ok(expires_at)
}

/// Deletes one session. Returns `false` when the session does not exist.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn delete(connection: &Connection, id_hash: &str) -> Result<bool, DbError> {
    Ok(connection.execute("DELETE FROM sessions WHERE id_hash = ?1", [id_hash])? > 0)
}

/// Deletes the sessions that have expired at `now` and returns their count.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn delete_expired(connection: &Connection, now: i64) -> Result<usize, DbError> {
    Ok(connection.execute("DELETE FROM sessions WHERE expires_at <= ?1", [now])?)
}
