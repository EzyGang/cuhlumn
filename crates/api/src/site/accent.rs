//! Accent colors of sites.

use serde::{Deserialize, Serialize};

/// A hex color such as `#2563eb` or `#abc`, stored in lowercase.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(try_from = "String", into = "String")]
pub struct Accent(String);

/// A text that is not a hex color.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the accent {0:?} is not a color: use #rgb or #rrggbb, for example #2563eb")]
pub struct AccentError(String);

const DEFAULT: &str = "#2563eb";

impl Accent {
    /// Validates a hex color in the form `#rgb` or `#rrggbb`.
    ///
    /// # Errors
    ///
    /// Returns [`AccentError`] when the text is not a hex color.
    pub fn new(value: impl Into<String>) -> Result<Self, AccentError> {
        let mut value = value.into();
        let valid = match value.strip_prefix('#') {
            Some(digits) => {
                matches!(digits.len(), 3 | 6) && digits.bytes().all(|byte| byte.is_ascii_hexdigit())
            }
            None => false,
        };
        if !valid {
            return Err(AccentError(value));
        }
        value.make_ascii_lowercase();
        Ok(Self(value))
    }
}

string_type!(Accent, AccentError);

impl Default for Accent {
    fn default() -> Self {
        Self(DEFAULT.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::Accent;

    #[test]
    fn accepts_short_and_long_hex_colors() {
        assert_eq!(Accent::new("#abc").unwrap().as_str(), "#abc");
        assert_eq!(Accent::new("#0E7490").unwrap().as_str(), "#0e7490");
        assert_eq!(Accent::default().as_str(), "#2563eb");
    }

    #[test]
    fn rejects_other_text() {
        for bad in [
            "red;}",
            "red;}body{display:none",
            "#abcd",
            "abc",
            "#ggg",
            "",
            "#",
            "#12345",
        ] {
            assert!(Accent::new(bad).is_err(), "{bad} passed");
        }
    }
}
