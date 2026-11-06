//! IndexNow keys of sites.

use serde::{Deserialize, Serialize};

/// A key that proves to IndexNow engines that a site owns its pings.
///
/// The site serves the key at `/{key}.txt`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(try_from = "String", into = "String")]
pub struct IndexNowKey(String);

/// A text that is not an IndexNow key.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the IndexNow key is not valid: use 8 to 128 letters, digits, or hyphens")]
pub struct IndexNowKeyError;

impl IndexNowKey {
    /// Validates a key of 8 to 128 letters, digits, or hyphens.
    ///
    /// # Errors
    ///
    /// Returns [`IndexNowKeyError`] when the text is not a valid key.
    pub fn new(value: impl Into<String>) -> Result<Self, IndexNowKeyError> {
        let value = value.into();
        let valid = (8..=128).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');
        if !valid {
            return Err(IndexNowKeyError);
        }
        Ok(Self(value))
    }
}

string_type!(IndexNowKey, IndexNowKeyError);

#[cfg(test)]
mod tests {
    use super::IndexNowKey;

    #[test]
    fn accepts_valid_keys() {
        assert!(IndexNowKey::new("143f4ce9a39c0691d2e85e8b486880a4").is_ok());
        assert!(IndexNowKey::new("abcd-123").is_ok());
        assert!(IndexNowKey::new("a".repeat(128)).is_ok());
    }

    #[test]
    fn rejects_invalid_keys() {
        for bad in ["../x.txt", "short", "abc def ghi", ""] {
            assert!(IndexNowKey::new(bad).is_err(), "{bad} passed");
        }
        assert!(IndexNowKey::new("a".repeat(129)).is_err());
    }
}
