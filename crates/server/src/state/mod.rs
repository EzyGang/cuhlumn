//! Shared state of all request handlers.

pub mod settings;

use std::{
    io,
    path::{Path, PathBuf},
    sync::{Arc, PoisonError, RwLock},
};

use cuhlumn_api::site::host::Host;
use rusqlite::Connection;

use crate::{
    analytics::Analytics,
    config::Config,
    db::{
        Database, DbError, analytics as analytics_records, fonts as font_records, sessions,
        settings as stored, sites, unix_now,
    },
    fonts::{
        FontStore,
        faces::{self, LibraryFace},
    },
    indexnow::IndexNow,
    media::MediaStore,
    og,
    state::settings::Settings,
    web::cache::PageCache,
};

/// The shared state: settings, database, media files, fonts, page cache, analytics, and IndexNow client.
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
    /// An HTTP client cannot be built.
    #[error("cannot build an HTTP client: {0}")]
    Http(#[from] reqwest::Error),
    /// The operating system random source failed.
    #[error("the random source failed: {0}")]
    Random(getrandom::Error),
}

struct Inner {
    config: Config,
    settings: RwLock<Arc<Settings>>,
    db: Database,
    media: MediaStore,
    fonts: FontStore,
    /// The faces of the library fonts, for blog stylesheets.
    faces: RwLock<Arc<[LibraryFace]>>,
    cache: PageCache,
    indexnow: IndexNow,
    analytics: Analytics,
}

const DATABASE_FILE: &str = "cuhlumn.db";
const MEDIA_FOLDER: &str = "media";
const FONTS_FOLDER: &str = "fonts";

impl AppState {
    /// Opens the state with the public IndexNow endpoint.
    ///
    /// # Errors
    ///
    /// Returns [`StateError`] when a folder, the database, or an HTTP client cannot be prepared.
    pub async fn open(config: Config) -> Result<Self, StateError> {
        let indexnow = IndexNow::new()?;
        Self::open_with(config, indexnow).await
    }

    /// Creates the data folders, opens the database, deletes expired sessions, and loads the settings and the analytics salt.
    ///
    /// `indexnow` sends the pings, so tests can point it at a local address.
    ///
    /// # Errors
    ///
    /// Returns [`StateError`] when a folder, the database, an HTTP client, or the random source fails.
    pub async fn open_with(config: Config, indexnow: IndexNow) -> Result<Self, StateError> {
        let media_root = create_folder(&config.data.join(MEDIA_FOLDER)).await?;
        let fonts_root = create_folder(&config.data.join(FONTS_FOLDER)).await?;
        let path = config.data.join(DATABASE_FILE);
        let db = tokio::task::spawn_blocking(move || Database::open(&path))
            .await
            .map_err(DbError::from)??;
        let (expired, stored, salt) = db
            .call(|connection| {
                let expired = sessions::delete_expired(connection, unix_now())?;
                let salt = analytics_records::salt(connection)?;
                Ok::<_, DbError>((expired, stored::get(connection)?, salt))
            })
            .await?;
        let analytics = Analytics::new(salt, unix_now()).map_err(StateError::Random)?;
        tracing::debug!(expired, "deleted expired sessions");
        let settings = Settings::resolve(&config, &stored);
        og::fonts::init(fonts_root.clone());
        let fonts = FontStore::new(fonts_root)?;
        let library = load_faces(&db, &fonts).await?;
        Ok(Self {
            inner: Arc::new(Inner {
                settings: RwLock::new(Arc::new(settings)),
                media: MediaStore::new(media_root),
                fonts,
                faces: RwLock::new(library),
                cache: PageCache::default(),
                config,
                db,
                indexnow,
                analytics,
            }),
        })
    }

    /// Returns the flags and environment variables of the process.
    ///
    /// Requests use [`AppState::settings`], which applies the stored values too.
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    /// Returns the effective server settings.
    #[must_use]
    pub fn settings(&self) -> Arc<Settings> {
        let settings = self
            .inner
            .settings
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        Arc::clone(&settings)
    }

    /// Runs `change` on the database, then loads the effective settings again in the same database call.
    ///
    /// `change` gets the settings from before the change. The return value has the new settings.
    ///
    /// # Errors
    ///
    /// Returns the error of `change`, or a database failure.
    pub async fn change_settings<F, T, E>(&self, change: F) -> Result<(T, Arc<Settings>), E>
    where
        F: FnOnce(&mut Connection, &Settings) -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: From<DbError> + Send + 'static,
    {
        let state = self.clone();
        self.db()
            .call(move |connection| {
                let value = change(connection, &state.settings())?;
                let next = Arc::new(Settings::resolve(
                    &state.inner.config,
                    &stored::get(connection)?,
                ));
                let mut current = state
                    .inner
                    .settings
                    .write()
                    .unwrap_or_else(PoisonError::into_inner);
                *current = Arc::clone(&next);
                Ok((value, next))
            })
            .await
    }

    /// Returns `true` when `host` serves the console.
    ///
    /// With a console host, only that host does.
    /// Without one, every host that is not a site domain or alias does.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] when the site lookup fails.
    pub async fn serves_console(&self, host: &Host) -> Result<bool, DbError> {
        if let Some(console) = self.settings().console_host() {
            return Ok(console == host);
        }
        if self.cache().has_pages(host) {
            return Ok(false);
        }
        let name = host.clone();
        let owner = self
            .db()
            .call(move |connection| sites::owner(connection, &name))
            .await?;
        Ok(owner.is_none())
    }

    /// Loads the font library again for share images and blog stylesheets, and drops every cached page.
    ///
    /// # Errors
    ///
    /// Returns [`DbError`] when the font list cannot be read or a blocking task fails.
    pub async fn fonts_changed(&self) -> Result<(), DbError> {
        let library = load_faces(self.db(), self.fonts()).await?;
        tokio::task::spawn_blocking(og::fonts::reload).await?;
        *self
            .inner
            .faces
            .write()
            .unwrap_or_else(PoisonError::into_inner) = library;
        self.cache().clear();
        Ok(())
    }

    /// Returns the faces of every library font.
    #[must_use]
    pub fn library_faces(&self) -> Arc<[LibraryFace]> {
        let faces = self
            .inner
            .faces
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        Arc::clone(&faces)
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

    /// Returns the font library store.
    #[must_use]
    pub fn fonts(&self) -> &FontStore {
        &self.inner.fonts
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

    /// Returns the page-view analytics.
    #[must_use]
    pub fn analytics(&self) -> &Analytics {
        &self.inner.analytics
    }
}

/// Reads the faces of every recorded library font.
async fn load_faces(db: &Database, fonts: &FontStore) -> Result<Arc<[LibraryFace]>, DbError> {
    let names: Vec<String> = db
        .call(|connection| font_records::list(connection))
        .await?
        .into_iter()
        .map(|font| font.name)
        .collect();
    let root = fonts.root().to_owned();
    let faces = tokio::task::spawn_blocking(move || faces::load(&root, &names)).await?;
    Ok(Arc::from(faces))
}

async fn create_folder(path: &Path) -> Result<PathBuf, StateError> {
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|source| StateError::Folder {
            path: path.to_owned(),
            source,
        })?;
    Ok(path.to_owned())
}
