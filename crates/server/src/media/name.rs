//! Content-based file names of uploaded images.

use std::fmt;

use cuhlumn_api::{settings::Scheme, site::host::Host};
use sha2::{Digest, Sha256};

use crate::media::format::ImageFormat;

/// The file name of an uploaded image: 32 hex digits of its SHA-256 digest and an extension.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MediaName {
    name: String,
    format: ImageFormat,
}

/// A file name that is not a media name.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is not a media file name")]
pub struct MediaNameError(String);

const HASH_LEN: usize = 32;

impl MediaName {
    /// Validates a media file name such as `0123456789abcdef0123456789abcdef.png`.
    ///
    /// # Errors
    ///
    /// Returns [`MediaNameError`] when the name does not have 32 lowercase hex digits and a known extension.
    pub fn new(value: impl Into<String>) -> Result<Self, MediaNameError> {
        let name = value.into();
        let parsed = name.split_once('.').and_then(|(hash, extension)| {
            let hex = hash.len() == HASH_LEN
                && hash
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
            hex.then(|| ImageFormat::from_extension(extension))
                .flatten()
        });
        match parsed {
            Some(format) => Ok(Self { name, format }),
            None => Err(MediaNameError(name)),
        }
    }

    /// Returns the name of the image with these bytes.
    #[must_use]
    pub fn for_content(format: ImageFormat, bytes: &[u8]) -> Self {
        let mut name = hex::encode(Sha256::digest(bytes));
        name.truncate(HASH_LEN);
        name.push('.');
        name.push_str(format.extension());
        Self { name, format }
    }

    /// Returns the name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// Returns the image format that the extension names.
    #[must_use]
    pub fn format(&self) -> ImageFormat {
        self.format
    }

    /// Returns the public address of the image, such as `https://blog.example.com/media/{name}`.
    #[must_use]
    pub fn url(&self, scheme: Scheme, host: &Host) -> String {
        format!("{scheme}://{host}/media/{}", self.name)
    }
}

impl TryFrom<String> for MediaName {
    type Error = MediaNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<MediaName> for String {
    fn from(name: MediaName) -> Self {
        name.name
    }
}

impl fmt::Display for MediaName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use crate::media::{format::ImageFormat, name::MediaName};

    #[test]
    fn content_names_use_the_digest_prefix() {
        let name = MediaName::for_content(ImageFormat::Png, b"abc");
        assert_eq!(name.as_str(), "ba7816bf8f01cfea414140de5dae2223.png");
        assert_eq!(MediaName::new(name.as_str()), Ok(name));
    }

    #[test]
    fn rejects_other_names() {
        for bad in [
            "../../cuhlumn.db",
            "ba7816bf8f01cfea414140de5dae2223.exe",
            "BA7816BF8F01CFEA414140DE5DAE2223.png",
            "ba7816bf8f01cfea414140de5dae222.png",
            "ba7816bf8f01cfea414140de5dae2223.png.tmp",
            "ba7816bf8f01cfea414140de5dae2223",
            "..%2f..%2fcuhlumn.db",
        ] {
            assert!(MediaName::new(bad).is_err(), "{bad} passed");
        }
    }

    #[test]
    fn extension_gives_the_format() {
        let name = MediaName::new("ba7816bf8f01cfea414140de5dae2223.jpg").unwrap();
        assert_eq!(name.format(), ImageFormat::Jpeg);
    }
}
