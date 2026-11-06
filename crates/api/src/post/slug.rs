//! Slugs: the path names of posts and pages.

use serde::{Deserialize, Serialize};

/// The path name of a post or page, such as `how-we-back-up-sqlite`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(try_from = "String", into = "String")]
pub struct Slug(String);

/// A text that is not a valid slug.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SlugError {
    /// The text is empty.
    #[error("the slug is empty")]
    Empty,
    /// The text has more than 100 bytes.
    #[error("the slug is longer than 100 bytes")]
    TooLong,
    /// The text has characters or hyphens in places that the rules do not permit.
    #[error(
        "the slug {0:?} is not valid: use lowercase letters and digits in groups joined by single hyphens"
    )]
    Invalid(String),
    /// The text is a path that the server uses for other content.
    #[error("the slug {0:?} is reserved")]
    Reserved(String),
}

/// Slugs that the server uses for its own paths.
pub const RESERVED: [&str; 4] = ["api", "og", "media", "tags"];

const MAX_LEN: usize = 100;

impl Slug {
    /// Validates a slug of 1 to 100 bytes in the form `[a-z0-9]+(-[a-z0-9]+)*`.
    ///
    /// # Errors
    ///
    /// Returns [`SlugError`] when the text is not a valid slug or is in [`RESERVED`].
    pub fn new(value: impl Into<String>) -> Result<Self, SlugError> {
        let value = value.into();
        if value.is_empty() {
            return Err(SlugError::Empty);
        }
        if value.len() > MAX_LEN {
            return Err(SlugError::TooLong);
        }
        let valid = value.split('-').all(|group| {
            !group.is_empty()
                && group
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        });
        if !valid {
            return Err(SlugError::Invalid(value));
        }
        if RESERVED.contains(&value.as_str()) {
            return Err(SlugError::Reserved(value));
        }
        Ok(Self(value))
    }
}

string_type!(Slug, SlugError);

#[cfg(test)]
mod tests {
    use super::{Slug, SlugError};

    #[test]
    fn accepts_valid_slugs() {
        for good in [
            "a",
            "sqlite-backups",
            "2026-plans",
            "a1-b2-c3",
            "api2",
            "tagsx",
        ] {
            assert!(Slug::new(good).is_ok(), "{good} failed");
        }
        assert!(Slug::new("a".repeat(100)).is_ok());
    }

    #[test]
    fn rejects_invalid_slugs() {
        assert_eq!(Slug::new(""), Err(SlugError::Empty));
        assert_eq!(Slug::new("a".repeat(101)), Err(SlugError::TooLong));
        for bad in ["-a", "a-", "a--b", "A", "a_b", "a/b", "..", "a b", "é"] {
            assert!(
                matches!(Slug::new(bad), Err(SlugError::Invalid(_))),
                "{bad} passed"
            );
        }
    }

    #[test]
    fn rejects_reserved_slugs() {
        for reserved in ["api", "og", "media", "tags"] {
            assert_eq!(
                Slug::new(reserved),
                Err(SlugError::Reserved(reserved.to_owned()))
            );
        }
    }
}
