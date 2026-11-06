//! Tags of posts.

use serde::{Deserialize, Serialize};

/// A trimmed tag of 1 to 50 characters, such as `Databases`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(try_from = "String", into = "String")]
pub struct Tag(String);

/// The tags of a post: at most 20, without case-insensitive repetitions.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(try_from = "Vec<Tag>", into = "Vec<Tag>")]
pub struct Tags(Vec<Tag>);

/// A text that is not a valid tag.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TagError {
    /// The text is empty or has only whitespace.
    #[error("the tag is blank")]
    Blank,
    /// The trimmed text has more than 50 characters.
    #[error("the tag is longer than 50 characters")]
    TooLong,
    /// The text contains a control character, such as a line break or U+001B.
    #[error("the tag contains the control character U+{0:04X}")]
    ControlChar(u32),
}

/// A list with more than 20 different tags.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("a post has at most 20 tags, not {0}")]
pub struct TagsError(usize);

const TAG_MAX: usize = 50;
const TAGS_MAX: usize = 20;

impl Tag {
    /// Validates a tag and removes the whitespace around it.
    ///
    /// # Errors
    ///
    /// Returns [`TagError`] when the trimmed text is blank, longer than 50 characters, or has a control character.
    pub fn new(value: impl Into<String>) -> Result<Self, TagError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(TagError::Blank);
        }
        if trimmed.chars().count() > TAG_MAX {
            return Err(TagError::TooLong);
        }
        if let Some(character) = trimmed.chars().find(|character| character.is_control()) {
            return Err(TagError::ControlChar(u32::from(character)));
        }
        if trimmed.len() == value.len() {
            return Ok(Self(value));
        }
        Ok(Self(trimmed.to_owned()))
    }
}

string_type!(Tag, TagError);

impl Tags {
    /// Removes case-insensitive repetitions and checks the count.
    ///
    /// The first spelling of each tag stays in its position.
    ///
    /// # Errors
    ///
    /// Returns [`TagsError`] when more than 20 different tags remain.
    pub fn new(tags: Vec<Tag>) -> Result<Self, TagsError> {
        let mut unique: Vec<Tag> = Vec::with_capacity(tags.len().min(TAGS_MAX));
        let mut keys: Vec<String> = Vec::with_capacity(unique.capacity());
        for tag in tags {
            let key = tag.as_str().to_lowercase();
            if !keys.contains(&key) {
                keys.push(key);
                unique.push(tag);
            }
        }
        if unique.len() > TAGS_MAX {
            return Err(TagsError(unique.len()));
        }
        Ok(Self(unique))
    }

    /// Returns the tags in their order.
    #[must_use]
    pub fn as_slice(&self) -> &[Tag] {
        &self.0
    }
}

impl TryFrom<Vec<Tag>> for Tags {
    type Error = TagsError;

    fn try_from(tags: Vec<Tag>) -> Result<Self, Self::Error> {
        Self::new(tags)
    }
}

impl From<Tags> for Vec<Tag> {
    fn from(tags: Tags) -> Self {
        tags.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Tag, TagError, Tags, TagsError};

    fn tags(values: &[&str]) -> Vec<Tag> {
        values
            .iter()
            .map(|value| Tag::new(*value).unwrap())
            .collect()
    }

    #[test]
    fn tags_are_trimmed_and_limited() {
        assert_eq!(Tag::new("  Rust ").unwrap().as_str(), "Rust");
        assert_eq!(Tag::new("   "), Err(TagError::Blank));
        assert_eq!(
            Tag::new("é".repeat(50)).unwrap().as_str().chars().count(),
            50
        );
        assert_eq!(Tag::new("a".repeat(51)), Err(TagError::TooLong));
        assert_eq!(Tag::new("a\nb"), Err(TagError::ControlChar(0x0a)));
        assert_eq!(Tag::new("esc\u{1b}"), Err(TagError::ControlChar(0x1b)));
    }

    #[test]
    fn repetitions_are_removed_case_insensitively() {
        let list = Tags::new(tags(&[
            "Databases",
            "Operations",
            "databases",
            "OPERATIONS",
        ]))
        .unwrap();
        let names: Vec<&str> = list.as_slice().iter().map(Tag::as_str).collect();
        assert_eq!(names, ["Databases", "Operations"]);
    }

    #[test]
    fn more_than_twenty_tags_fail() {
        let names: Vec<String> = (0..21).map(|n| format!("tag{n}")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        assert_eq!(Tags::new(tags(&refs)), Err(TagsError(21)));
        assert!(Tags::new(tags(&refs[..20])).is_ok());
        let mut repeated = refs[..20].to_vec();
        repeated.push("TAG0");
        assert!(Tags::new(tags(&repeated)).is_ok());
    }
}
