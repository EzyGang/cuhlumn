//! The fonts of share cards: the system fonts and every font file in the font library folder.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use resvg::usvg::fontdb::Database;

/// Sets the font library folder. Call it at startup, before the first card.
///
/// A call with another folder drops the loaded fonts, so the next card reads the new folder.
pub fn init(folder: PathBuf) {
    let mut library = write();
    if library.folder.as_deref() == Some(folder.as_path()) {
        return;
    }
    library.folder = Some(folder);
    library.forget();
}

/// Returns the system fonts and the library fonts.
///
/// The first call after [`init`] or [`reload`] reads the font files, so call it on the blocking thread pool.
#[must_use]
pub fn database() -> Arc<Database> {
    let (folder, generation) = {
        let library = read();
        if let Some(database) = &library.database {
            return Arc::clone(database);
        }
        (library.folder.clone(), library.generation)
    };
    let database = Arc::new(load(folder.as_deref()));
    let mut library = write();
    if library.generation == generation && library.database.is_none() {
        library.database = Some(Arc::clone(&database));
    }
    database
}

/// Drops the loaded fonts, so the next card reads the library folder again.
pub fn reload() {
    write().forget();
}

/// The font library folder and the fonts loaded from it.
struct Library {
    folder: Option<PathBuf>,
    /// Changes each time the loaded fonts become stale, so a slow load does not store old fonts.
    generation: u64,
    database: Option<Arc<Database>>,
}

static LIBRARY: RwLock<Library> = RwLock::new(Library {
    folder: None,
    generation: 0,
    database: None,
});

/// The system fonts, read once.
static SYSTEM: LazyLock<Database> = LazyLock::new(|| {
    let mut database = Database::new();
    database.load_system_fonts();
    tracing::info!(
        faces = database.len(),
        "loaded the system fonts for share cards"
    );
    database
});

impl Library {
    fn forget(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.database = None;
    }
}

fn load(folder: Option<&Path>) -> Database {
    let mut database = SYSTEM.clone();
    if let Some(folder) = folder {
        let system = database.len();
        database.load_fonts_dir(folder);
        tracing::info!(
            faces = database.len() - system,
            folder = %folder.display(),
            "loaded the library fonts for share cards"
        );
    }
    database
}

fn read() -> RwLockReadGuard<'static, Library> {
    LIBRARY.read().unwrap_or_else(PoisonError::into_inner)
}

fn write() -> RwLockWriteGuard<'static, Library> {
    LIBRARY.write().unwrap_or_else(PoisonError::into_inner)
}
