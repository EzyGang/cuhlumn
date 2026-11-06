//! The head tags that describe a page: title, description, canonical address, robots, Open Graph, and Twitter.

use cuhlumn_api::post::tags::Tag;
use maud::{Markup, html};
use serde_json::Value;

use crate::render::{blog::Image, iso};

/// What the head of a page says about the page.
pub struct Head<'a> {
    /// The `<title>` text.
    pub title: String,
    /// The title for share previews.
    pub headline: &'a str,
    pub description: String,
    /// The canonical address. Pages that do not exist have none.
    pub url: Option<String>,
    pub image: Image,
    pub kind: OgType,
    /// Lets search engines index the page.
    pub index: bool,
    pub published: Option<i64>,
    pub modified: Option<i64>,
    pub tags: &'a [Tag],
    /// The address of the Markdown copy.
    pub markdown: Option<String>,
    /// The JSON-LD data of the page.
    pub schema: Option<Value>,
}

/// The Open Graph type of a page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OgType {
    Website,
    Article,
}

impl<'a> Head<'a> {
    /// Returns an indexable `website` head without an address, dates, tags, or JSON-LD.
    #[must_use]
    pub fn new(title: String, headline: &'a str, description: String, image: Image) -> Self {
        Self {
            title,
            headline,
            description,
            url: None,
            image,
            kind: OgType::Website,
            index: true,
            published: None,
            modified: None,
            tags: &[],
            markdown: None,
            schema: None,
        }
    }

    /// Returns the title, description, canonical, and robots tags.
    #[must_use]
    pub fn document_tags(&self) -> Markup {
        let robots = match self.index {
            true => "index, follow, max-image-preview:large",
            false => "noindex, follow",
        };
        html! {
            title { (self.title) }
            meta name="description" content=(self.description);
            @if let Some(url) = &self.url { link rel="canonical" href=(url); }
            meta name="robots" content=(robots);
        }
    }

    /// Returns the Open Graph and Twitter tags.
    #[must_use]
    pub fn social_tags(&self, site_title: &str) -> Markup {
        html! {
            meta property="og:type" content=(self.kind.as_str());
            meta property="og:site_name" content=(site_title);
            meta property="og:title" content=(self.headline);
            meta property="og:description" content=(self.description);
            @if let Some(url) = &self.url { meta property="og:url" content=(url); }
            meta property="og:image" content=(self.image.url);
            @if self.image.card {
                meta property="og:image:width" content="1200";
                meta property="og:image:height" content="630";
            }
            meta property="og:image:alt" content=(self.headline);
            @if let Some(time) = self.published { meta property="article:published_time" content=(iso(time)); }
            @if let Some(time) = self.modified { meta property="article:modified_time" content=(iso(time)); }
            @for tag in self.tags { meta property="article:tag" content=(tag); }
            meta name="twitter:card" content="summary_large_image";
            meta name="twitter:title" content=(self.headline);
            meta name="twitter:description" content=(self.description);
            meta name="twitter:image" content=(self.image.url);
        }
    }
}

impl OgType {
    /// Returns the `og:type` value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Website => "website",
            Self::Article => "article",
        }
    }
}
