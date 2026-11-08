//! The font library: font files in `{data}/fonts/` that share images use next to the system fonts.

pub mod faces;
pub mod google;
pub mod inspect;

use std::{
    fmt, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use bytes::Bytes;
use cuhlumn_api::fonts::{FontItem, FontSource};

use crate::{db::unix_now, fonts::inspect::inspect};

/// The font folder and the HTTP client for imports.
#[derive(Clone, Debug)]
pub struct FontStore {
    root: PathBuf,
    client: reqwest::Client,
}

/// A failure in the font library.
#[derive(Debug, thiserror::Error)]
pub enum FontError {
    /// The bytes are not a font that the server can read.
    #[error("{0}")]
    NotAFont(String),
    /// A file is larger than the limit.
    #[error("the font file is too large")]
    TooLarge,
    /// The import address is not a permitted Google Fonts address.
    #[error("{0}")]
    Url(String),
    /// Google Fonts sent an answer that the server cannot use.
    #[error("{0}")]
    Upstream(String),
    /// The stylesheet lists no font file.
    #[error("the Google Fonts stylesheet lists no font file")]
    NoFiles,
    /// An HTTP request failed.
    #[error("the Google Fonts request failed: {0}")]
    Http(#[from] reqwest::Error),
    /// A file operation failed.
    #[error("font storage failed at {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    /// A blocking task did not finish.
    #[error("the font task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
}

/// The file name of a library font: ASCII letters, digits, and hyphens, then `.ttf`, `.otf`, or `.ttc`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontName(String);

/// A text that is not a library font file name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("a font name has letters, digits, and hyphens, then .ttf, .otf, or .ttc")]
pub struct FontNameError;

/// The largest font upload, in bytes.
pub const UPLOAD_LIMIT: usize = 20 * 1024 * 1024;

const TIMEOUT: Duration = Duration::from_secs(30);
const USER_AGENT: &str = concat!("cuhlumn-server/", env!("CARGO_PKG_VERSION"));

/// Makes temporary file names unique inside this process.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

impl FontStore {
    /// Uses `root` as the font folder.
    ///
    /// # Errors
    ///
    /// Returns [`reqwest::Error`] when the import HTTP client cannot be built.
    pub fn new(root: PathBuf) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(USER_AGENT)
            .build()?;
        Ok(Self { root, client })
    }

    /// Checks the font bytes and writes them to the folder. The database record is the caller's task.
    ///
    /// # Errors
    ///
    /// Returns [`FontError::NotAFont`] for bytes that are not a font, and [`FontError::Io`] when the write fails.
    pub async fn save(&self, bytes: Bytes, source: FontSource) -> Result<FontItem, FontError> {
        let root = self.root.clone();
        tokio::task::spawn_blocking(move || {
            let inspected = inspect(&bytes)?;
            write_file(&root.join(&inspected.name), &bytes)?;
            Ok(FontItem {
                name: inspected.name,
                families: inspected.families,
                source,
                size: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
                created_at: unix_now(),
            })
        })
        .await?
    }

    /// Downloads the font files of a Google Fonts address.
    ///
    /// # Errors
    ///
    /// Returns [`FontError`] for a wrong address, an upstream failure, or a file that is too large.
    pub async fn download_google(&self, url: &str) -> Result<Vec<Bytes>, FontError> {
        google::download(&self.client, url).await
    }

    /// Opens a library font file for reading. Returns `None` when it does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`FontError::Io`] when the file exists but cannot be opened.
    pub async fn open(&self, name: &FontName) -> Result<Option<tokio::fs::File>, FontError> {
        let path = self.root.join(name.as_str());
        match tokio::fs::File::open(&path).await {
            Ok(file) => Ok(Some(file)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(FontError::Io { path, source }),
        }
    }

    /// Returns the font folder.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Deletes a font file. A missing file is not an error.
    ///
    /// # Errors
    ///
    /// Returns [`FontError::Io`] when the file exists but cannot be deleted.
    pub async fn remove(&self, name: &FontName) -> Result<(), FontError> {
        let path = self.root.join(name.as_str());
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(FontError::Io { path, source }),
        }
    }
}

impl FontName {
    /// Validates a font file name.
    ///
    /// # Errors
    ///
    /// Returns [`FontNameError`] when the name could leave the font folder or has another extension.
    pub fn new(name: &str) -> Result<Self, FontNameError> {
        let (stem, extension) = name.rsplit_once('.').ok_or(FontNameError)?;
        let valid = !stem.is_empty()
            && stem.len() <= 128
            && stem
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            && matches!(extension, "ttf" | "otf" | "ttc");
        match valid {
            true => Ok(Self(name.to_owned())),
            false => Err(FontNameError),
        }
    }

    /// Returns the file name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the media type of the file, from its extension.
    #[must_use]
    pub fn content_type(&self) -> &'static str {
        match self.extension() {
            "otf" => "font/otf",
            "ttc" => "font/collection",
            _ => "font/ttf",
        }
    }

    /// Returns the format name of the CSS `src` descriptor, from the extension.
    #[must_use]
    pub fn css_format(&self) -> &'static str {
        match self.extension() {
            "otf" => "opentype",
            "ttc" => "collection",
            _ => "truetype",
        }
    }

    fn extension(&self) -> &str {
        self.0
            .rsplit_once('.')
            .map_or("", |(_, extension)| extension)
    }
}

impl fmt::Display for FontName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Writes `bytes` to a temporary file next to `path`, then renames it into place.
fn write_file(path: &Path, bytes: &[u8]) -> Result<(), FontError> {
    let at = |path: &Path| {
        let path = path.to_owned();
        move |source| FontError::Io { path, source }
    };
    let temp = path.with_extension(format!(
        "{}-{}.tmp",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&temp, bytes).map_err(at(&temp))?;
    if let Err(source) = std::fs::rename(&temp, path) {
        if let Err(cleanup) = std::fs::remove_file(&temp) {
            tracing::warn!(path = %temp.display(), error = %cleanup, "temporary font file stays on disk");
        }
        return Err(at(path)(source));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::fonts::FontName;

    #[test]
    fn font_names_stay_in_the_folder() {
        assert!(FontName::new("Inter-Regular-1a2b3c4d.ttf").is_ok());
        assert!(FontName::new("NotoSansJP-0f0f0f0f.otf").is_ok());
        for bad in [
            "../x.ttf",
            "a/b.ttf",
            "a.woff2",
            ".ttf",
            "a b.ttf",
            "a",
            "a.ttf.tmp",
        ] {
            assert!(FontName::new(bad).is_err(), "{bad}");
        }
    }
}
