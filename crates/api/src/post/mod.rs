//! Posts and pages of a site.

pub mod slug;
pub mod tags;
#[cfg(test)]
mod tests;

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    post::{slug::Slug, tags::Tags},
    site::link::LinkUrl,
    text,
};

/// A stored post or page, as the API returns it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Post {
    pub slug: Slug,
    pub title: String,
    pub description: String,
    /// The Markdown text.
    pub body: String,
    pub tags: Tags,
    pub kind: Kind,
    pub status: Status,
    pub cover: Option<LinkUrl>,
    /// The share image that replaces the generated card.
    pub og_image: Option<LinkUrl>,
    /// The first publication time, in unix seconds.
    #[cfg_attr(feature = "ts", ts(type = "number | null"))]
    pub published_at: Option<i64>,
    /// The time of the last change, in unix seconds.
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub updated_at: i64,
    /// The public address: `{scheme}://{host}/{slug}/`, or `{scheme}://{host}/` for the home page.
    pub url: String,
}

/// The body of `PUT /api/sites/{host}/posts/{slug}`.
///
/// Omitted fields take their default values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct PostInput {
    /// Trimmed, not blank, at most 300 characters.
    #[serde(deserialize_with = "text::de_required::<_, TITLE_MAX>")]
    pub title: String,
    #[serde(default, deserialize_with = "text::de_text")]
    pub description: String,
    /// The Markdown text, not blank.
    #[serde(deserialize_with = "text::de_not_blank")]
    pub body: String,
    #[serde(default)]
    pub tags: Tags,
    #[serde(default)]
    pub kind: Kind,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub cover: Option<LinkUrl>,
    #[serde(default)]
    pub og_image: Option<LinkUrl>,
    /// A publication time to set, in unix seconds from 0 to 253402300799 (the end of year 9999).
    /// `None` keeps the first publication time.
    #[serde(default, deserialize_with = "de_published_at")]
    #[cfg_attr(feature = "ts", ts(type = "number | null"))]
    pub published_at: Option<i64>,
}

/// The kind of a post.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", ts(rename_all = "lowercase"))]
pub enum Kind {
    /// A dated post that the feed and the post list show.
    #[default]
    Post,
    /// An undated page, such as "About".
    Page,
}

/// The visibility of a post.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", ts(rename_all = "lowercase"))]
pub enum Status {
    /// Saved and hidden from readers.
    #[default]
    Draft,
    /// Visible to readers.
    Published,
}

/// A text that is not a post kind.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the kind {0:?} is not valid: use post or page")]
pub struct KindError(String);

/// A text that is not a post status.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the status {0:?} is not valid: use draft or published")]
pub struct StatusError(String);

/// A publication time outside the years 1970 to 9999.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the publication time {0} is out of range: use unix seconds from 0 to {PUBLISHED_AT_MAX}")]
pub struct PublishedAtError(i64);

const TITLE_MAX: usize = 300;
/// The last second of the year 9999 in unix seconds.
const PUBLISHED_AT_MAX: i64 = 253_402_300_799;

impl Kind {
    /// Returns the name used in JSON and in the database.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Post => "post",
            Self::Page => "page",
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Kind {
    type Err = KindError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "post" => Ok(Self::Post),
            "page" => Ok(Self::Page),
            _ => Err(KindError(value.to_owned())),
        }
    }
}

impl TryFrom<String> for Kind {
    type Error = KindError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Status {
    /// Returns the name used in JSON and in the database.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Status {
    type Err = StatusError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "draft" => Ok(Self::Draft),
            "published" => Ok(Self::Published),
            _ => Err(StatusError(value.to_owned())),
        }
    }
}

impl TryFrom<String> for Status {
    type Error = StatusError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

/// Deserializes an optional publication time from 0 to [`PUBLISHED_AT_MAX`].
fn de_published_at<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let time = Option::<i64>::deserialize(deserializer)?;
    match time {
        Some(value) if !(0..=PUBLISHED_AT_MAX).contains(&value) => {
            Err(serde::de::Error::custom(PublishedAtError(value)))
        }
        _ => Ok(time),
    }
}
