//! Uploaded images on disk, named by their content.

pub mod format;
pub mod name;

use std::{
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use bytes::Bytes;
use cuhlumn_api::site::host::Host;

use crate::media::{format::ImageFormat, name::MediaName};

/// The image folders of all sites: `{data}/media/{host}/`.
#[derive(Clone, Debug)]
pub struct MediaStore {
    root: PathBuf,
}

/// A file system failure in the media folders.
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    /// A file or folder operation failed.
    #[error("media storage failed at {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    /// The blocking write task did not finish.
    #[error("the media write task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
}

/// Makes temporary file names unique inside this process.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

impl MediaStore {
    /// Uses `root` as the folder that holds one image folder per site.
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Writes an image of `format` for the site at `host` and returns its name.
    ///
    /// The hash and the write run on the blocking thread pool.
    /// The bytes go to a temporary file first, then a rename puts them in place.
    /// An existing file with the same name has the same content, so it stays unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`MediaError`] when the folder or the file cannot be written.
    pub async fn save(
        &self,
        host: &Host,
        format: ImageFormat,
        bytes: Bytes,
    ) -> Result<MediaName, MediaError> {
        let dir = self.site_dir(host);
        tokio::task::spawn_blocking(move || write_content(&dir, format, &bytes)).await?
    }

    /// Opens an image of the site at `host`. Returns `None` when the file does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`MediaError`] when the file exists but cannot be opened.
    pub async fn open(
        &self,
        host: &Host,
        name: &MediaName,
    ) -> Result<Option<tokio::fs::File>, MediaError> {
        let path = self.site_dir(host).join(name.as_str());
        match tokio::fs::File::open(&path).await {
            Ok(file) => Ok(Some(file)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(MediaError::at(&path, source)),
        }
    }

    /// Deletes an image. A missing file is not an error.
    ///
    /// # Errors
    ///
    /// Returns [`MediaError`] when the file exists but cannot be deleted.
    pub async fn remove(&self, host: &Host, name: &MediaName) -> Result<(), MediaError> {
        let path = self.site_dir(host).join(name.as_str());
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(MediaError::at(&path, source)),
        }
    }

    /// Deletes the image folder of the site at `host`. A missing folder is not an error.
    ///
    /// # Errors
    ///
    /// Returns [`MediaError`] when the folder exists but cannot be deleted.
    pub async fn remove_site(&self, host: &Host) -> Result<(), MediaError> {
        let dir = self.site_dir(host);
        match tokio::fs::remove_dir_all(&dir).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(MediaError::at(&dir, source)),
        }
    }

    fn site_dir(&self, host: &Host) -> PathBuf {
        self.root.join(host.as_str())
    }
}

impl MediaError {
    fn at(path: &Path, source: io::Error) -> Self {
        Self::Io {
            path: path.to_owned(),
            source,
        }
    }
}

/// Hashes the bytes and writes them to `dir` unless a file with the same name exists.
fn write_content(dir: &Path, format: ImageFormat, bytes: &[u8]) -> Result<MediaName, MediaError> {
    let name = MediaName::for_content(format, bytes);
    std::fs::create_dir_all(dir).map_err(|source| MediaError::at(dir, source))?;
    let path = dir.join(name.as_str());
    if path
        .try_exists()
        .map_err(|source| MediaError::at(&path, source))?
    {
        return Ok(name);
    }
    let temp = dir.join(format!(
        "{name}.{}-{}.tmp",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&temp, bytes).map_err(|source| MediaError::at(&temp, source))?;
    if let Err(source) = std::fs::rename(&temp, &path) {
        if let Err(cleanup) = std::fs::remove_file(&temp) {
            tracing::warn!(path = %temp.display(), error = %cleanup, "temporary media file stays on disk");
        }
        return Err(MediaError::at(&path, source));
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use cuhlumn_api::site::host::Host;

    use crate::media::{MediaStore, format::ImageFormat};

    const PNG: Bytes = Bytes::from_static(b"\x89PNG\r\n\x1a\nbody");

    #[tokio::test]
    async fn saves_opens_and_removes_images() {
        let dir = tempfile::tempdir().unwrap();
        let store = MediaStore::new(dir.path().join("media"));
        let host = Host::new("blog.example.com").unwrap();
        let name = store.save(&host, ImageFormat::Png, PNG).await.unwrap();
        let again = store.save(&host, ImageFormat::Png, PNG).await.unwrap();
        assert_eq!(name, again);
        let path = dir
            .path()
            .join("media/blog.example.com")
            .join(name.as_str());
        assert_eq!(std::fs::read(&path).unwrap(), PNG);
        let files = std::fs::read_dir(path.parent().unwrap()).unwrap().count();
        assert_eq!(files, 1, "temporary files stay on disk");
        assert!(store.open(&host, &name).await.unwrap().is_some());
        store.remove(&host, &name).await.unwrap();
        assert!(store.open(&host, &name).await.unwrap().is_none());
        store.remove(&host, &name).await.unwrap();
        store.save(&host, ImageFormat::Png, PNG).await.unwrap();
        store.remove_site(&host).await.unwrap();
        assert!(!dir.path().join("media/blog.example.com").exists());
        store.remove_site(&host).await.unwrap();
    }
}
