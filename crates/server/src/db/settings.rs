//! The server settings row.
//!
//! The row stores the SHA-256 hex digest of the admin token, never the token.

use cuhlumn_api::{settings::Scheme, site::host::Host};
use rusqlite::{Connection, Row, params};

use crate::db::{DbError, parse_optional};

/// The stored server settings. `None` means that the value is not set.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StoredSettings {
    /// The SHA-256 hex digest of the admin token.
    pub token_digest: Option<String>,
    pub console_host: Option<Host>,
    pub scheme: Option<Scheme>,
    /// The time of the last change, in unix seconds.
    pub updated_at: i64,
}

/// Returns the settings row.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails or a stored value is invalid.
pub fn get(connection: &Connection) -> Result<StoredSettings, DbError> {
    let settings = connection.query_row(
        "SELECT token_digest, console_host, scheme, updated_at FROM settings WHERE id = 1",
        [],
        settings_row,
    )?;
    Ok(settings)
}

/// Stores the token digest only when no digest is stored yet.
///
/// Returns `true` when this call stored the digest, so the first caller wins.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn set_token_if_empty(
    connection: &Connection,
    digest: &str,
    now: i64,
) -> Result<bool, DbError> {
    let changed = connection.execute(
        "UPDATE settings SET token_digest = ?1, updated_at = ?2
         WHERE id = 1 AND token_digest IS NULL",
        params![digest, now],
    )?;
    Ok(changed > 0)
}

/// Replaces the token digest and deletes every session in one transaction.
///
/// # Errors
///
/// Returns [`DbError`] when a query fails.
pub fn replace_token(connection: &mut Connection, digest: &str, now: i64) -> Result<(), DbError> {
    let tx = connection.transaction()?;
    tx.execute(
        "UPDATE settings SET token_digest = ?1, updated_at = ?2 WHERE id = 1",
        params![digest, now],
    )?;
    tx.execute("DELETE FROM sessions", [])?;
    tx.commit()?;
    Ok(())
}

/// Stores the console host and the scheme.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn set_server(
    connection: &Connection,
    console_host: Option<&Host>,
    scheme: Scheme,
    now: i64,
) -> Result<(), DbError> {
    connection.execute(
        "UPDATE settings SET console_host = ?1, scheme = ?2, updated_at = ?3 WHERE id = 1",
        params![console_host.map(Host::as_str), scheme.as_str(), now],
    )?;
    Ok(())
}

fn settings_row(row: &Row<'_>) -> rusqlite::Result<StoredSettings> {
    Ok(StoredSettings {
        token_digest: row.get("token_digest")?,
        console_host: parse_optional(row, "console_host")?,
        scheme: parse_optional(row, "scheme")?,
        updated_at: row.get("updated_at")?,
    })
}
