//! Blank, length, and character rules for free text and list fields.

use serde::{Deserialize, Deserializer, de::Error as _};

/// A text or list value that breaks a blank or length rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TextError {
    /// The text is empty or has only whitespace.
    #[error("the value must not be blank")]
    Blank,
    /// The text has too many characters.
    #[error("the value is longer than {max} characters")]
    TooManyChars {
        /// The largest permitted number of characters.
        max: usize,
    },
    /// The text has too many bytes.
    #[error("the value is longer than {max} bytes")]
    TooManyBytes {
        /// The largest permitted number of bytes.
        max: usize,
    },
    /// The list has too many items.
    #[error("the list has more than {max} items")]
    TooManyItems {
        /// The largest permitted number of items.
        max: usize,
    },
    /// The text contains a control character other than tab, line feed, and carriage return.
    #[error("the value contains the control character U+{code:04X}")]
    ControlChar {
        /// The code point of the first control character.
        code: u32,
    },
}

/// Returns the first control character other than tab, line feed, and carriage return.
pub(crate) fn find_control(value: &str) -> Option<char> {
    value
        .chars()
        .find(|character| character.is_control() && !matches!(character, '\t' | '\n' | '\r'))
}

/// Returns an error when the text contains a control character other than tab, line feed, and carriage return.
pub(crate) fn check_controls(value: &str) -> Result<(), TextError> {
    match find_control(value) {
        Some(character) => Err(TextError::ControlChar {
            code: u32::from(character),
        }),
        None => Ok(()),
    }
}

/// Returns the trimmed text when it is not blank, has at most `max` characters, and has no control characters.
pub(crate) fn required(value: &str, max: usize) -> Result<String, TextError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(TextError::Blank);
    }
    if value.chars().count() > max {
        return Err(TextError::TooManyChars { max });
    }
    check_controls(value)?;
    Ok(value.to_owned())
}

/// Deserializes a text with [`required`].
pub(crate) fn de_required<'de, D, const MAX: usize>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    required(&value, MAX).map_err(D::Error::custom)
}

/// Deserializes a text that is not blank and has no control characters, and keeps it unchanged.
pub(crate) fn de_not_blank<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.trim().is_empty() {
        return Err(D::Error::custom(TextError::Blank));
    }
    check_controls(&value).map_err(D::Error::custom)?;
    Ok(value)
}

/// Deserializes a text without control characters.
pub(crate) fn de_text<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    check_controls(&value).map_err(D::Error::custom)?;
    Ok(value)
}

/// Deserializes a text that has at most `MAX` characters and no control characters.
pub(crate) fn de_max_chars<'de, D, const MAX: usize>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.chars().count() > MAX {
        return Err(D::Error::custom(TextError::TooManyChars { max: MAX }));
    }
    check_controls(&value).map_err(D::Error::custom)?;
    Ok(value)
}

/// Deserializes a text that has at most `MAX` bytes.
pub(crate) fn de_max_bytes<'de, D, const MAX: usize>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.len() > MAX {
        return Err(D::Error::custom(TextError::TooManyBytes { max: MAX }));
    }
    Ok(value)
}

/// Deserializes a list that has at most `MAX` items.
pub(crate) fn de_max_items<'de, D, T, const MAX: usize>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let items = Vec::<T>::deserialize(deserializer)?;
    if items.len() > MAX {
        return Err(D::Error::custom(TextError::TooManyItems { max: MAX }));
    }
    Ok(items)
}

/// Deserializes a list and removes repeated items, so the first of each stays in its position.
pub(crate) fn de_unique<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + PartialEq,
{
    let items = Vec::<T>::deserialize(deserializer)?;
    let mut unique = Vec::with_capacity(items.len());
    for item in items {
        if !unique.contains(&item) {
            unique.push(item);
        }
    }
    Ok(unique)
}

#[cfg(test)]
mod tests {
    use super::{TextError, check_controls, de_text, required};

    #[test]
    fn required_text_is_trimmed_and_limited() {
        assert_eq!(required("  Notes ", 5), Ok("Notes".to_owned()));
        assert_eq!(required(" \n ", 5), Err(TextError::Blank));
        assert_eq!(
            required("abcdef", 5),
            Err(TextError::TooManyChars { max: 5 })
        );
        assert_eq!(required("ééééé", 5), Ok("ééééé".to_owned()));
    }

    #[test]
    fn control_characters_are_rejected() {
        assert_eq!(check_controls("tab\tline\nreturn\r"), Ok(()));
        for (text, code) in [
            ("a\u{1}b", 1),
            ("\u{b}", 0xb),
            ("esc\u{1b}", 0x1b),
            ("\u{7f}", 0x7f),
            ("\u{85}", 0x85),
        ] {
            assert_eq!(
                check_controls(text),
                Err(TextError::ControlChar { code }),
                "{text:?}"
            );
        }
        assert_eq!(
            required("bad\u{0}title", 20),
            Err(TextError::ControlChar { code: 0 })
        );
        let mut json = serde_json::Deserializer::from_str("\"bell\\u0007\"");
        assert!(de_text(&mut json).is_err());
    }
}
