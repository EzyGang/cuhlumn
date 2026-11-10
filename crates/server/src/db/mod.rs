//! Sites, posts, media records, sessions, and analytics in one SQLite file.

pub mod analytics;
pub mod fonts;
pub mod media;
mod migrations;
pub mod posts;
pub mod sessions;
pub mod settings;
pub mod sites;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_settings;

use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{Connection, Row, types::Type};

/// The database: one SQLite connection that the blocking thread pool uses.
#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
}

/// A database failure.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// SQLite returned an error.
    #[error("SQLite failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// A thread panicked while it held the connection.
    #[error("the database connection is poisoned")]
    Poisoned,
    /// A JSON column value cannot be encoded.
    #[error("JSON encoding failed: {0}")]
    Json(#[from] serde_json::Error),
    /// The blocking task did not finish.
    #[error("the database task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
}

/// Returns the current time in unix seconds.
#[must_use]
pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
        })
}

impl Database {
    /// Opens or creates the database file and applies the migrations.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] when SQLite cannot open the file or apply a migration.
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let mut connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.pragma_update(None, "busy_timeout", 5000)?;
        migrations::apply(&mut connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// Runs `f` with the connection on the blocking thread pool.
    ///
    /// # Errors
    ///
    /// Returns the error of `f`, or [`DbError`] when the connection is poisoned or the task fails.
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut Connection) -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: From<DbError> + Send + 'static,
    {
        let connection = Arc::clone(&self.connection);
        tokio::task::spawn_blocking(move || {
            let mut connection = connection.lock().map_err(|_| DbError::Poisoned)?;
            f(&mut connection)
        })
        .await
        .map_err(DbError::from)?
    }
}

impl DbError {
    /// Returns `true` when SQLite rejected a write because of a foreign key.
    #[must_use]
    pub fn is_foreign_key(&self) -> bool {
        matches!(
            self,
            Self::Sqlite(rusqlite::Error::SqliteFailure(failure, _))
                if failure.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY
        )
    }
}

/// Reads a text column and validates it with `T::try_from`.
fn parse<T>(row: &Row<'_>, column: &str) -> rusqlite::Result<T>
where
    T: TryFrom<String>,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    let value: String = row.get(column)?;
    T::try_from(value).map_err(|err| conversion(row, column, err))
}

/// Reads a nullable text column and validates it with `T::try_from`.
fn parse_optional<T>(row: &Row<'_>, column: &str) -> rusqlite::Result<Option<T>>
where
    T: TryFrom<String>,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    let value: Option<String> = row.get(column)?;
    value
        .map(T::try_from)
        .transpose()
        .map_err(|err| conversion(row, column, err))
}

/// Reads a JSON text column.
fn parse_json<T>(row: &Row<'_>, column: &str) -> rusqlite::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let value: String = row.get(column)?;
    serde_json::from_str(&value).map_err(|err| conversion(row, column, err))
}

fn conversion<E>(row: &Row<'_>, column: &str, err: E) -> rusqlite::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    let index = row.as_ref().column_index(column).unwrap_or_default();
    rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(err))
}
