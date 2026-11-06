//! Sites: one blog on its own domain.

pub mod accent;
pub mod host;
pub mod indexnow_key;
pub mod lang;
pub mod link;
#[cfg(test)]
mod tests;

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    post::slug::Slug,
    site::{
        accent::Accent,
        host::Host,
        indexnow_key::IndexNowKey,
        lang::Lang,
        link::{AbsoluteUrl, Link, LinkUrl},
    },
    text,
};

/// A stored site, as the API returns it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Site {
    /// The canonical domain of the site.
    pub host: Host,
    pub title: String,
    pub description: String,
    /// Other domains that redirect to the canonical domain.
    pub aliases: Vec<Host>,
    pub author: String,
    pub author_url: Option<AbsoluteUrl>,
    /// The product that the blog sends readers to.
    pub project_name: String,
    pub project_url: Option<AbsoluteUrl>,
    /// The call to action after each post.
    pub cta: String,
    pub lang: Lang,
    /// The slug of the page that the site shows as its home page.
    pub home: Option<Slug>,
    pub indexnow_key: IndexNowKey,
    pub brand: Brand,
    /// The time of the last change, in unix seconds.
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub updated_at: i64,
}

/// The body of `PUT /api/sites/{host}`.
///
/// Omitted fields take their default values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct SiteInput {
    /// Trimmed, not blank, at most 200 characters.
    #[serde(deserialize_with = "text::de_required::<_, TITLE_MAX>")]
    pub title: String,
    #[serde(default, deserialize_with = "text::de_text")]
    pub description: String,
    /// Other domains of the site, without repetitions.
    #[serde(default, deserialize_with = "text::de_unique")]
    pub aliases: Vec<Host>,
    #[serde(default, deserialize_with = "text::de_text")]
    pub author: String,
    #[serde(default)]
    pub author_url: Option<AbsoluteUrl>,
    #[serde(default, deserialize_with = "text::de_text")]
    pub project_name: String,
    #[serde(default)]
    pub project_url: Option<AbsoluteUrl>,
    #[serde(default, deserialize_with = "text::de_text")]
    pub cta: String,
    #[serde(default)]
    pub lang: Lang,
    /// Must name an existing page of the site.
    #[serde(default)]
    pub home: Option<Slug>,
    /// `None` keeps the stored key. A new site without a key gets a random key.
    #[serde(default)]
    pub indexnow_key: Option<IndexNowKey>,
    #[serde(default)]
    pub brand: Brand,
}

/// The look settings of a site, shown as "Appearance" in the console.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(default, deny_unknown_fields)]
pub struct Brand {
    pub accent: Accent,
    pub font: Font,
    pub logo: Option<LinkUrl>,
    pub favicon: Option<LinkUrl>,
    /// The share image for pages that have no share image of their own.
    pub og_image: Option<LinkUrl>,
    /// Header links, at most 8.
    #[serde(deserialize_with = "text::de_max_items::<_, Link, LINKS_MAX>")]
    pub links: Vec<Link>,
    /// Raw HTML for the page head, at most 20 000 bytes.
    #[serde(deserialize_with = "text::de_max_bytes::<_, HEAD_MAX>")]
    pub head: String,
    /// Footer text, at most 500 characters.
    #[serde(deserialize_with = "text::de_max_chars::<_, FOOTER_MAX>")]
    pub footer: String,
    /// Shows the "Published with Cuhlumn" credit in the footer.
    pub credit: bool,
}

/// The font family of post text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", ts(rename_all = "lowercase"))]
pub enum Font {
    #[default]
    Serif,
    Sans,
}

/// A text that is not a font name.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the font {0:?} is not valid: use serif or sans")]
pub struct FontError(String);

const TITLE_MAX: usize = 200;
const LINKS_MAX: usize = 8;
const HEAD_MAX: usize = 20_000;
const FOOTER_MAX: usize = 500;

impl From<Site> for SiteInput {
    /// Returns the input that stores the same site again, with the same IndexNow key.
    fn from(site: Site) -> Self {
        Self {
            title: site.title,
            description: site.description,
            aliases: site.aliases,
            author: site.author,
            author_url: site.author_url,
            project_name: site.project_name,
            project_url: site.project_url,
            cta: site.cta,
            lang: site.lang,
            home: site.home,
            indexnow_key: Some(site.indexnow_key),
            brand: site.brand,
        }
    }
}

impl Default for Brand {
    fn default() -> Self {
        Self {
            accent: Accent::default(),
            font: Font::default(),
            logo: None,
            favicon: None,
            og_image: None,
            links: Vec::new(),
            head: String::new(),
            footer: String::new(),
            credit: true,
        }
    }
}

impl Font {
    /// Returns the name used in JSON and in the database.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Serif => "serif",
            Self::Sans => "sans",
        }
    }
}

impl fmt::Display for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Font {
    type Err = FontError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "serif" => Ok(Self::Serif),
            "sans" => Ok(Self::Sans),
            _ => Err(FontError(value.to_owned())),
        }
    }
}

impl TryFrom<String> for Font {
    type Error = FontError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
