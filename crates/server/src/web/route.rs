//! The public addresses of a blog, parsed from the request path.

use std::borrow::Cow;

use cuhlumn_api::{post::slug::Slug, site::indexnow_key::IndexNowKey};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};

use crate::media::name::MediaName;

/// A public address of a blog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    /// `/`
    Home,
    /// `/{slug}/`: a post or a page.
    Entry(Slug),
    /// `/{slug}`: redirects to `/{slug}/` when the entry exists.
    Bare(Slug),
    /// `/{slug}.md`: the Markdown copy.
    Markdown(Slug),
    /// `/tags/{tag}/`: the posts with a tag.
    Tag(String),
    /// `/tags/{tag}`: redirects to `/tags/{tag}/` when the tag exists.
    BareTag(String),
    /// `/feed.xml`
    Feed,
    /// `/sitemap.xml`
    Sitemap,
    /// `/robots.txt`
    Robots,
    /// `/llms.txt`
    Llms,
    /// `/llms-full.txt`
    LlmsFull,
    /// `/favicon.svg`
    Favicon,
    /// `/favicon.ico`: redirects to `/favicon.svg`.
    FaviconIco,
    /// `/og.png`: the share card of the site.
    SiteCard,
    /// `/og/{slug}.png`: the share card of an entry.
    EntryCard(Slug),
    /// `/media/{name}`: an uploaded image.
    Media(MediaName),
    /// `/{key}.txt`: the IndexNow key file when `key` is the key of the site.
    KeyFile(IndexNowKey),
    /// Any other path.
    NotFound,
}

/// Percent-decodes a request path. A path that does not decode to UTF-8 stays as it is.
#[must_use]
pub fn decode_path(path: &str) -> Cow<'_, str> {
    percent_decode_str(path)
        .decode_utf8()
        .unwrap_or(Cow::Borrowed(path))
}

/// Percent-encodes one path segment, so a `Location` header stays ASCII.
#[must_use]
pub fn encode_segment(segment: &str) -> String {
    utf8_percent_encode(segment, SEGMENT).to_string()
}

/// The characters that a path segment in a `Location` header encodes.
const SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'\\')
    .add(b'^')
    .add(b'|');

impl Route {
    /// Parses a percent-decoded path.
    #[must_use]
    pub fn parse(path: &str) -> Self {
        match path {
            "/" => return Self::Home,
            "/feed.xml" => return Self::Feed,
            "/sitemap.xml" => return Self::Sitemap,
            "/robots.txt" => return Self::Robots,
            "/llms.txt" => return Self::Llms,
            "/llms-full.txt" => return Self::LlmsFull,
            "/favicon.svg" => return Self::Favicon,
            "/favicon.ico" => return Self::FaviconIco,
            "/og.png" => return Self::SiteCard,
            _ => (),
        }
        let Some(rest) = path.strip_prefix('/') else {
            return Self::NotFound;
        };
        if let Some(file) = rest.strip_prefix("og/") {
            return slug(file.strip_suffix(".png")).map_or(Self::NotFound, Self::EntryCard);
        }
        if let Some(name) = rest.strip_prefix("media/") {
            return MediaName::new(name).map_or(Self::NotFound, Self::Media);
        }
        if let Some(tag) = rest.strip_prefix("tags/") {
            return match tag.strip_suffix('/') {
                Some(tag) if is_tag(tag) => Self::Tag(tag.to_owned()),
                None if is_tag(tag) => Self::BareTag(tag.to_owned()),
                _ => Self::NotFound,
            };
        }
        if let Some(stem) = rest.strip_suffix(".md") {
            return slug(Some(stem)).map_or(Self::NotFound, Self::Markdown);
        }
        if let Some(stem) = rest.strip_suffix(".txt") {
            return IndexNowKey::new(stem).map_or(Self::NotFound, Self::KeyFile);
        }
        match rest.strip_suffix('/') {
            Some(entry) => slug(Some(entry)).map_or(Self::NotFound, Self::Entry),
            None => slug(Some(rest)).map_or(Self::NotFound, Self::Bare),
        }
    }
}

fn slug(text: Option<&str>) -> Option<Slug> {
    Slug::new(text?).ok()
}

fn is_tag(tag: &str) -> bool {
    !tag.is_empty() && !tag.contains('/')
}

#[cfg(test)]
mod tests {
    use crate::web::route::{Route, decode_path, encode_segment};

    fn slug(value: &str) -> cuhlumn_api::post::slug::Slug {
        value.parse().unwrap()
    }

    #[test]
    fn parses_pages_and_files() {
        assert_eq!(Route::parse("/"), Route::Home);
        assert_eq!(Route::parse("/hello/"), Route::Entry(slug("hello")));
        assert_eq!(Route::parse("/hello"), Route::Bare(slug("hello")));
        assert_eq!(Route::parse("/hello.md"), Route::Markdown(slug("hello")));
        assert_eq!(
            Route::parse("/og/hello.png"),
            Route::EntryCard(slug("hello"))
        );
        assert_eq!(Route::parse("/og.png"), Route::SiteCard);
        assert_eq!(Route::parse("/llms-full.txt"), Route::LlmsFull);
        assert_eq!(Route::parse("/favicon.ico"), Route::FaviconIco);
        assert!(matches!(
            Route::parse("/0123456789abcdef.txt"),
            Route::KeyFile(_)
        ));
        assert!(matches!(
            Route::parse("/media/0123456789abcdef0123456789abcdef.png"),
            Route::Media(_)
        ));
    }

    #[test]
    fn parses_tags() {
        assert_eq!(Route::parse("/tags/thé/"), Route::Tag("thé".to_owned()));
        assert_eq!(
            Route::parse("/tags/rust"),
            Route::BareTag("rust".to_owned())
        );
        for path in ["/tags/", "/tags", "/tags//", "/tags/a/b/"] {
            assert_eq!(Route::parse(path), Route::NotFound, "{path}");
        }
    }

    #[test]
    fn rejects_odd_paths() {
        for path in [
            "/media/../../cuhlumn.db",
            "/media/",
            "/Hello/",
            "/og/",
            "/api/",
            "/a/b/",
            "/x.txt",
            "",
        ] {
            assert_eq!(Route::parse(path), Route::NotFound, "{path}");
        }
    }

    #[test]
    fn decodes_and_encodes() {
        assert_eq!(decode_path("/tags/th%C3%A9/"), "/tags/thé/");
        assert_eq!(decode_path("/%FF/"), "/%FF/");
        assert_eq!(encode_segment("thé au lait"), "th%C3%A9%20au%20lait");
        assert_eq!(encode_segment("a/b?c#d"), "a%2Fb%3Fc%23d");
    }
}
