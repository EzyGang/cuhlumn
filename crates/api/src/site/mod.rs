//! Sites: one blog on its own domain.

pub mod accent;
pub mod announcement;
pub mod brand;
pub mod host;
pub mod indexnow_key;
pub mod lang;
pub mod link;
pub mod social;
#[cfg(test)]
mod tests;
pub mod theme;

use serde::{Deserialize, Serialize};

use crate::{
    post::slug::Slug,
    site::{
        announcement::Announcement, brand::Brand, host::Host, indexnow_key::IndexNowKey,
        lang::Lang, link::AbsoluteUrl,
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
    /// Sends IndexNow pings when a post is published.
    pub indexnow: bool,
    /// The announcement bar, shown while it is live.
    pub announcement: Option<Announcement>,
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
    /// Sends IndexNow pings when a post is published.
    #[serde(default = "indexnow_default")]
    pub indexnow: bool,
    #[serde(default)]
    pub announcement: Option<Announcement>,
}

const TITLE_MAX: usize = 200;

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
            indexnow: site.indexnow,
            announcement: site.announcement,
        }
    }
}

fn indexnow_default() -> bool {
    true
}
