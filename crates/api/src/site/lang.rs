//! Language tags of sites.

use serde::{Deserialize, Serialize};

/// A language tag such as `en` or `pt-BR`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(try_from = "String", into = "String")]
pub struct Lang(String);

/// A text that is not a language tag.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error(
    "the language tag {0:?} is not valid: use 2 to 15 letters, digits, or hyphens that start with a letter, for example en or pt-BR"
)]
pub struct LangError(String);

const DEFAULT: &str = "en";

impl Lang {
    /// Validates a language tag of 2 to 15 letters, digits, or hyphens that starts with a letter.
    ///
    /// # Errors
    ///
    /// Returns [`LangError`] when the text is not a language tag.
    pub fn new(value: impl Into<String>) -> Result<Self, LangError> {
        let value = value.into();
        let valid = (2..=15).contains(&value.len())
            && value.starts_with(|c: char| c.is_ascii_alphabetic())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');
        if !valid {
            return Err(LangError(value));
        }
        Ok(Self(value))
    }
}

string_type!(Lang, LangError);

impl Default for Lang {
    fn default() -> Self {
        Self(DEFAULT.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::Lang;

    #[test]
    fn accepts_language_tags() {
        for good in ["en", "pt-BR", "zh-Hant-TW", "abcdefghijklmno"] {
            assert!(Lang::new(good).is_ok(), "{good} failed");
        }
        assert_eq!(Lang::default().as_str(), "en");
    }

    #[test]
    fn rejects_other_text() {
        for bad in ["e", "", "1en", "-en", "en_US", "abcdefghijklmnop", "en us"] {
            assert!(Lang::new(bad).is_err(), "{bad} passed");
        }
    }
}
