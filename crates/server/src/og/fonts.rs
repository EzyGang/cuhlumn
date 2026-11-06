//! The fonts of share cards: the system fonts and the optional `CUHLUMN_FONTS` folder.

use std::{
    path::Path,
    sync::{Arc, OnceLock},
};

use resvg::usvg::fontdb::Database;

/// Returns the font database. The first call loads the fonts, so call it on the blocking thread pool.
///
/// `extra` is the folder from `CUHLUMN_FONTS`. Only the first call reads it.
pub fn database(extra: Option<&Path>) -> Arc<Database> {
    static FONTS: OnceLock<Arc<Database>> = OnceLock::new();
    let fonts = FONTS.get_or_init(|| {
        let mut database = Database::new();
        database.load_system_fonts();
        if let Some(folder) = extra {
            database.load_fonts_dir(folder);
        }
        tracing::info!(faces = database.len(), "loaded the fonts for share cards");
        Arc::new(database)
    });
    Arc::clone(fonts)
}
