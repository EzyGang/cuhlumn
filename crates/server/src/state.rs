//! Shared state of all request handlers.

use std::{io, path::PathBuf, sync::Arc};

use crate::{
    config::Config,
    db::{Database, DbError, sessions, unix_now},
    indexnow::IndexNow,
    media::MediaStore,
    web::cache::PageCache,
};

/// The shared state: settings, database, media files, page cache, and IndexNow client.
///
/// Clones share one instance.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

/// A failure while the server prepares its state.
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    /// A data folder cannot be created.
    #[error("cannot create the folder {path}: {source}")]
    Folder { path: PathBuf, source: io::Error },
    /// The database cannot be opened.
    #[error(transparent)]
    Database(#[from] DbError),
    /// The HTTP client for IndexNow cannot be built.
    #[error("cannot build the IndexNow HTTP client: {0}")]
    Http(#[from] reqwest::Error),
}

struct Inner {
    config: Config,
    db: Database,
    media: MediaStore,
    cache: PageCache,
    indexnow: IndexNow,
}

const DATABASE_FILE: &str = "cuhlumn.db";
const MEDIA_FOLDER: &str = "media";

impl AppState {
    /// Creates the data folders, opens the database, and deletes expired sessions.
    ///
    /// # Errors
    ///
    /// Returns [`StateError`] when a folder, the database, or the HTTP client cannot be prepared.
    pub async fn open(config: Config) -> Result<Self, StateError> {
        let media_root = config.data.join(MEDIA_FOLDER);
        tokio::fs::create_dir_all(&media_root)
            .await
            .map_err(|source| StateError::Folder {
                path: media_root.clone(),
                source,
            })?;
        let path = config.data.join(DATABASE_FILE);
        let db = tokio::task::spawn_blocking(move || Database::open(&path))
            .await
            .map_err(DbError::from)??;
        let expired = db
            .call(|connection| sessions::delete_expired(connection, unix_now()))
            .await?;
        tracing::debug!(expired, "deleted expired sessions");
        let indexnow = IndexNow::new(config.indexnow)?;
        Ok(Self {
            inner: Arc::new(Inner {
                media: MediaStore::new(media_root),
                cache: PageCache::default(),
                config,
                db,
                indexnow,
            }),
        })
    }

    /// Returns the server settings.
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    /// Returns the database.
    #[must_use]
    pub fn db(&self) -> &Database {
        &self.inner.db
    }

    /// Returns the media file store.
    #[must_use]
    pub fn media(&self) -> &MediaStore {
        &self.inner.media
    }

    /// Returns the page cache.
    #[must_use]
    pub fn cache(&self) -> &PageCache {
        &self.inner.cache
    }

    /// Returns the IndexNow client.
    #[must_use]
    pub fn indexnow(&self) -> &IndexNow {
        &self.inner.indexnow
    }
}
