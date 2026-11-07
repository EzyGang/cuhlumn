//! Font import from Google Fonts: the stylesheet address, the font addresses in it, and the downloads.
//!
//! Requests go only to `fonts.googleapis.com` and `fonts.gstatic.com` over HTTPS, without redirects.

use bytes::Bytes;
use percent_encoding::percent_decode_str;
use reqwest::Url;

use crate::fonts::FontError;

/// The most font files that one import downloads.
pub const FILES_MAX: usize = 20;
/// The largest font file that an import downloads, in bytes.
pub const FILE_MAX: usize = 20 * 1024 * 1024;

const CSS_MAX: usize = 1024 * 1024;
const CSS_HOST: &str = "fonts.googleapis.com";
const FILE_HOST: &str = "fonts.gstatic.com";
const SPECIMEN_HOST: &str = "fonts.google.com";
const FAMILY_MAX: usize = 64;

/// Returns the stylesheet address for a specimen or stylesheet address of Google Fonts.
///
/// # Errors
///
/// Returns [`FontError::Url`] for another host, path, or a missing family.
pub fn stylesheet_url(input: &str) -> Result<Url, FontError> {
    let url = Url::parse(input).map_err(|err| FontError::Url(err.to_string()))?;
    let plain = url.scheme() == "https"
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none();
    if !plain {
        return Err(unsupported());
    }
    match url.host_str() {
        Some(SPECIMEN_HOST) => specimen(&url),
        Some(CSS_HOST) => stylesheet(&url),
        _ => Err(unsupported()),
    }
}

/// Returns the `fonts.gstatic.com` addresses in a stylesheet, without repeats, in their order.
#[must_use]
pub fn font_urls(css: &str) -> Vec<Url> {
    let mut urls: Vec<Url> = Vec::new();
    for (_, rest) in css
        .match_indices("url(")
        .map(|(at, _)| css.split_at(at + 4))
    {
        let Some(end) = rest.find(')') else {
            break;
        };
        let raw = rest
            .get(..end)
            .unwrap_or_default()
            .trim()
            .trim_matches(|ch| ch == '"' || ch == '\'');
        let Ok(url) = Url::parse(raw) else {
            continue;
        };
        if is_font_file(&url) && !urls.contains(&url) {
            urls.push(url);
        }
    }
    urls
}

/// Downloads the stylesheet of `input` and every font file in it.
///
/// # Errors
///
/// Returns [`FontError`] for a wrong address, an upstream failure, too many files, or a file that is too large.
pub async fn download(client: &reqwest::Client, input: &str) -> Result<Vec<Bytes>, FontError> {
    let css = fetch(client, stylesheet_url(input)?, CSS_MAX).await?;
    let css = String::from_utf8_lossy(&css);
    let urls = font_urls(&css);
    if urls.is_empty() {
        return Err(FontError::NoFiles);
    }
    if urls.len() > FILES_MAX {
        return Err(FontError::Upstream(format!(
            "the stylesheet lists {} files, and the limit is {FILES_MAX}",
            urls.len()
        )));
    }
    let mut files = Vec::with_capacity(urls.len());
    for url in urls {
        files.push(fetch(client, url, FILE_MAX).await?);
    }
    Ok(files)
}

fn specimen(url: &Url) -> Result<Url, FontError> {
    let mut segments = url.path_segments().ok_or_else(unsupported)?;
    let (Some("specimen"), Some(raw)) = (segments.next(), segments.next()) else {
        return Err(unsupported());
    };
    let family = percent_decode_str(raw)
        .decode_utf8()
        .map_err(|err| FontError::Url(err.to_string()))?
        .replace('+', " ");
    let valid = !family.trim().is_empty()
        && family.len() <= FAMILY_MAX
        && family
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == ' ');
    if !valid {
        return Err(FontError::Url(
            "the specimen address has no valid family name".to_owned(),
        ));
    }
    let base = format!("https://{CSS_HOST}/css2");
    Url::parse_with_params(&base, [("family", family.trim())])
        .map_err(|err| FontError::Url(err.to_string()))
}

fn stylesheet(url: &Url) -> Result<Url, FontError> {
    if !matches!(url.path(), "/css" | "/css2") {
        return Err(unsupported());
    }
    if !url
        .query_pairs()
        .any(|(key, value)| key == "family" && !value.is_empty())
    {
        return Err(FontError::Url(
            "the stylesheet address has no family".to_owned(),
        ));
    }
    let mut clean = url.clone();
    clean.set_fragment(None);
    Ok(clean)
}

fn is_font_file(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some(FILE_HOST)
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
}

fn unsupported() -> FontError {
    FontError::Url(format!(
        "use a https://{SPECIMEN_HOST}/specimen/ or https://{CSS_HOST}/css2 address"
    ))
}

/// Downloads `url` with a size limit. The client follows no redirects.
async fn fetch(client: &reqwest::Client, url: Url, limit: usize) -> Result<Bytes, FontError> {
    let mut response = client.get(url).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(FontError::Upstream(format!(
            "Google Fonts answered with HTTP {status}"
        )));
    }
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(FontError::TooLarge);
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len() + chunk.len() > limit {
            return Err(FontError::TooLarge);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(Bytes::from(body))
}

#[cfg(test)]
mod tests {
    use crate::fonts::google::{font_urls, stylesheet_url};

    const CSS: &str = "/* latin */
@font-face {
  font-family: 'Fira Sans';
  font-style: normal;
  font-weight: 400;
  src: url(https://fonts.gstatic.com/s/firasans/v17/va9E4kDNxMZdWfMOD5Vvl4jO.ttf) format('truetype');
}
@font-face {
  font-family: 'Fira Sans';
  font-weight: 700;
  src: url('https://fonts.gstatic.com/s/firasans/v17/va9B4kDNxMZdWfMOD5VnLK3uQQ.ttf') format('truetype');
}
@font-face {
  src: url(https://fonts.gstatic.com/s/firasans/v17/va9E4kDNxMZdWfMOD5Vvl4jO.ttf);
  src: url(https://evil.example.com/font.ttf), url(http://fonts.gstatic.com/s/plain.ttf);
  src: url(https://fonts.gstatic.com.evil.example.com/a.ttf), url(https://fonts.gstatic.com:8443/b.ttf);
}";

    #[test]
    fn stylesheet_lists_only_gstatic_files() {
        let urls: Vec<String> = font_urls(CSS).into_iter().map(String::from).collect();
        assert_eq!(
            urls,
            [
                "https://fonts.gstatic.com/s/firasans/v17/va9E4kDNxMZdWfMOD5Vvl4jO.ttf",
                "https://fonts.gstatic.com/s/firasans/v17/va9B4kDNxMZdWfMOD5VnLK3uQQ.ttf",
            ]
        );
        assert!(font_urls("src: url(https://fonts.gstatic.com/a.ttf").is_empty());
    }

    #[test]
    fn addresses_become_stylesheet_urls() {
        let url = |input: &str| stylesheet_url(input).map(String::from);
        assert_eq!(
            url("https://fonts.google.com/specimen/Fira+Sans?query=fira").unwrap(),
            "https://fonts.googleapis.com/css2?family=Fira+Sans"
        );
        assert_eq!(
            url("https://fonts.google.com/specimen/Noto%20Sans%20JP").unwrap(),
            "https://fonts.googleapis.com/css2?family=Noto+Sans+JP"
        );
        assert_eq!(
            url("https://fonts.googleapis.com/css2?family=Inter:wght@400;700&display=swap#x")
                .unwrap(),
            "https://fonts.googleapis.com/css2?family=Inter:wght@400;700&display=swap"
        );
        assert!(url("https://fonts.googleapis.com/css?family=Lato").is_ok());
        for bad in [
            "http://fonts.googleapis.com/css2?family=Inter",
            "https://fonts.googleapis.com/icon?family=Material+Icons",
            "https://fonts.googleapis.com/css2",
            "https://fonts.googleapis.com:444/css2?family=Inter",
            "https://fonts.google.com/specimen/",
            "https://fonts.google.com/specimen/%3Cscript%3E",
            "https://fonts.google.com/about",
            "https://harbor.example.com/specimen/Inter",
        ] {
            assert!(url(bad).is_err(), "{bad}");
        }
    }
}
