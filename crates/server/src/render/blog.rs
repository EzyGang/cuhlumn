//! A site with its published posts: everything that one page build reads.

use std::sync::OnceLock;

use cuhlumn_api::{
    post::{Kind, Post, slug::Slug},
    site::{
        Site,
        link::{AbsoluteUrl, LinkUrl},
    },
};

use crate::render::{
    clip, day,
    markdown::{self, Doc},
};

/// A site and its published posts, newest first.
pub struct Blog {
    pub site: Site,
    /// The site root without a trailing slash, such as `https://blog.example.com`.
    pub origin: String,
    entries: Vec<Entry>,
}

/// A published post or page. Its Markdown renders at most once.
pub struct Entry {
    pub post: Post,
    doc: OnceLock<Doc>,
}

/// A share image and whether it is a generated 1200×630 card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub url: String,
    pub card: bool,
}

const SUMMARY_CHARS: usize = 155;
const WORDS_PER_MINUTE: usize = 220;

impl Blog {
    /// Holds `posts`, which must be the published posts of `site`, newest first.
    #[must_use]
    pub fn new(site: Site, origin: String, posts: Vec<Post>) -> Self {
        let entries = posts
            .into_iter()
            .map(|post| Entry {
                post,
                doc: OnceLock::new(),
            })
            .collect();
        Self {
            site,
            origin,
            entries,
        }
    }

    /// Returns every published post and page.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Returns the published entry with this slug.
    #[must_use]
    pub fn find(&self, slug: &Slug) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.post.slug == *slug)
    }

    /// Returns the dated posts without the pages.
    pub fn articles(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|entry| !entry.is_page())
    }

    /// Returns the pages for the header, without the home page.
    pub fn pages(&self) -> impl Iterator<Item = &Entry> {
        self.entries
            .iter()
            .filter(|entry| entry.is_page() && !self.is_home(&entry.post.slug))
    }

    /// Returns the page that the site shows as its home page.
    #[must_use]
    pub fn home_page(&self) -> Option<&Entry> {
        let home = self.site.home.as_ref()?;
        self.find(home).filter(|entry| entry.is_page())
    }

    /// Returns `true` when `slug` names the home page of the site.
    #[must_use]
    pub fn is_home(&self, slug: &Slug) -> bool {
        self.site.home.as_ref() == Some(slug)
    }

    /// Returns the address of the site root with its trailing slash.
    #[must_use]
    pub fn root_url(&self) -> String {
        format!("{}/", self.origin)
    }

    /// Returns the label and the address of the product that the blog promotes.
    #[must_use]
    pub fn project(&self) -> Option<(String, &AbsoluteUrl)> {
        let url = self.site.project_url.as_ref()?;
        let name = self.site.project_name.trim();
        let label = if name.is_empty() {
            let address = url.as_str();
            let bare = address.split_once("://").map_or(address, |(_, rest)| rest);
            bare.trim_end_matches('/').to_owned()
        } else {
            name.to_owned()
        };
        Some((label, url))
    }

    /// Returns a full address: `/media/x.png` becomes `https://host/media/x.png`.
    #[must_use]
    pub fn absolute(&self, url: &LinkUrl) -> String {
        match url.is_relative() {
            true => format!("{}{url}", self.origin),
            false => url.as_str().to_owned(),
        }
    }

    /// Returns the share image of the site: its own image, or the generated card.
    #[must_use]
    pub fn site_image(&self) -> Image {
        match &self.site.brand.og_image {
            Some(own) => Image {
                url: self.absolute(own),
                card: false,
            },
            None => Image {
                url: format!("{}/og.png?v={}", self.origin, self.site.updated_at),
                card: true,
            },
        }
    }

    /// Returns the share image of an entry: its own image, or the generated card.
    ///
    /// The card address changes when the post or the site changes, because both appear on the card.
    #[must_use]
    pub fn entry_image(&self, entry: &Entry) -> Image {
        let post = &entry.post;
        match &post.og_image {
            Some(own) => Image {
                url: self.absolute(own),
                card: false,
            },
            None => Image {
                url: format!(
                    "{}/og/{}.png?v={}",
                    self.origin,
                    post.slug,
                    post.updated_at.max(self.site.updated_at)
                ),
                card: true,
            },
        }
    }

    /// Returns the address of the Markdown copy of an entry.
    #[must_use]
    pub fn markdown_url(&self, entry: &Entry) -> String {
        format!("{}/{}.md", self.origin, entry.post.slug)
    }

    /// Returns the time of the latest change to the site or one of its published entries.
    #[must_use]
    pub fn updated_at(&self) -> i64 {
        self.entries
            .iter()
            .map(|entry| entry.post.updated_at)
            .fold(self.site.updated_at, i64::max)
    }
}

impl Entry {
    /// Returns the rendered Markdown. The first call renders it.
    #[must_use]
    pub fn doc(&self) -> &Doc {
        self.doc.get_or_init(|| markdown::render(&self.post.body))
    }

    /// Returns `true` for a page, `false` for a dated post.
    #[must_use]
    pub fn is_page(&self) -> bool {
        self.post.kind == Kind::Page
    }

    /// Returns the description, or the start of the text when the description is blank.
    #[must_use]
    pub fn summary(&self) -> String {
        let description = self.post.description.trim();
        if description.is_empty() {
            return clip(&self.doc().text, SUMMARY_CHARS);
        }
        description.to_owned()
    }

    /// Returns the reading time in minutes, at least 1.
    #[must_use]
    pub fn minutes(&self) -> usize {
        (self.doc().words / WORDS_PER_MINUTE).max(1)
    }

    /// Returns "October 27, 2026 · 4 min read" for a published post, and nothing for a page.
    #[must_use]
    pub fn card_meta(&self) -> String {
        match self.post.published_at {
            Some(time) if !self.is_page() => {
                format!("{} · {} min read", day(time), self.minutes())
            }
            _ => String::new(),
        }
    }
}
