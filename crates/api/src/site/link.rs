//! Web addresses and header links.

use serde::{Deserialize, Serialize};

use crate::text::{self, TextError};

/// A link in the site header, such as the status page of the product.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct Link {
    /// The visible text, trimmed, 1 to 40 characters.
    #[serde(deserialize_with = "text::de_required::<_, LABEL_MAX>")]
    pub label: String,
    /// The target of the link.
    pub url: LinkUrl,
}

/// An `http` or `https` address with a host, such as `https://status.harbor.example.com`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(try_from = "String", into = "String")]
pub struct AbsoluteUrl(String);

/// An [`AbsoluteUrl`] or a root-relative path such as `/media/logo.png`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(try_from = "String", into = "String")]
pub struct LinkUrl(String);

/// A text that is not a permitted address.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum UrlError {
    /// The text is longer than 2048 bytes.
    #[error("the address is longer than 2048 bytes")]
    TooLong,
    /// The text contains whitespace, a quote, an angle bracket, or a backslash.
    #[error("the address contains the forbidden character {0:?}")]
    ForbiddenChar(char),
    /// The text does not start with `http://`, `https://`, or a single `/`.
    #[error("the address must start with https://, http://, or a single /")]
    Scheme,
    /// The text has a scheme but no host.
    #[error("the address has no host")]
    MissingHost,
}

/// A header link with an invalid label or address.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LinkError {
    /// The label breaks the text rules.
    #[error("label: {0}")]
    Label(#[from] TextError),
    /// The address is not permitted.
    #[error("url: {0}")]
    Url(#[from] UrlError),
}

const LABEL_MAX: usize = 40;
const URL_MAX: usize = 2048;
const FORBIDDEN: &[char] = &['"', '\'', '<', '>', '\\'];

impl Link {
    /// Validates a header link.
    ///
    /// # Errors
    ///
    /// Returns [`LinkError`] when the label or the address is not valid.
    pub fn new(label: &str, url: impl Into<String>) -> Result<Self, LinkError> {
        Ok(Self {
            label: text::required(label, LABEL_MAX)?,
            url: LinkUrl::new(url)?,
        })
    }
}

impl AbsoluteUrl {
    /// Validates an absolute `http` or `https` address.
    ///
    /// # Errors
    ///
    /// Returns [`UrlError`] when the text is not a permitted absolute address.
    pub fn new(value: impl Into<String>) -> Result<Self, UrlError> {
        let value = value.into();
        check_chars(&value)?;
        check_absolute(&value)?;
        Ok(Self(value))
    }
}

string_type!(AbsoluteUrl, UrlError);

impl LinkUrl {
    /// Validates an absolute address or a root-relative path.
    ///
    /// A root-relative path starts with `/` but not with `//`.
    ///
    /// # Errors
    ///
    /// Returns [`UrlError`] when the text is not a permitted address.
    pub fn new(value: impl Into<String>) -> Result<Self, UrlError> {
        let value = value.into();
        check_chars(&value)?;
        match value.strip_prefix('/') {
            Some(rest) if rest.starts_with('/') => return Err(UrlError::Scheme),
            Some(_) => (),
            None => check_absolute(&value)?,
        }
        Ok(Self(value))
    }

    /// Returns `true` when the address is a root-relative path.
    #[must_use]
    pub fn is_relative(&self) -> bool {
        self.0.starts_with('/')
    }
}

string_type!(LinkUrl, UrlError);

impl From<AbsoluteUrl> for LinkUrl {
    fn from(value: AbsoluteUrl) -> Self {
        Self(value.0)
    }
}

fn check_chars(value: &str) -> Result<(), UrlError> {
    if value.len() > URL_MAX {
        return Err(UrlError::TooLong);
    }
    match value
        .chars()
        .find(|c| c.is_whitespace() || FORBIDDEN.contains(c))
    {
        Some(c) => Err(UrlError::ForbiddenChar(c)),
        None => Ok(()),
    }
}

fn check_absolute(value: &str) -> Result<(), UrlError> {
    let rest = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
        .ok_or(UrlError::Scheme)?;
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if host.is_empty() {
        return Err(UrlError::MissingHost);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AbsoluteUrl, Link, LinkError, LinkUrl, UrlError};
    use crate::text::TextError;

    #[test]
    fn absolute_urls_need_scheme_and_host() {
        assert!(AbsoluteUrl::new("https://example.com").is_ok());
        assert!(AbsoluteUrl::new("http://example.com/a?b=c#d").is_ok());
        assert_eq!(
            AbsoluteUrl::new("javascript:alert(1)"),
            Err(UrlError::Scheme)
        );
        assert_eq!(AbsoluteUrl::new("/media/a.png"), Err(UrlError::Scheme));
        assert_eq!(AbsoluteUrl::new("https://"), Err(UrlError::MissingHost));
        assert_eq!(
            AbsoluteUrl::new("https:///path"),
            Err(UrlError::MissingHost)
        );
        assert_eq!(
            AbsoluteUrl::new("https://a.com/\"x"),
            Err(UrlError::ForbiddenChar('"'))
        );
        assert_eq!(
            AbsoluteUrl::new("https://a.com/ x"),
            Err(UrlError::ForbiddenChar(' '))
        );
        assert_eq!(
            AbsoluteUrl::new("https://a.com/<"),
            Err(UrlError::ForbiddenChar('<'))
        );
    }

    #[test]
    fn link_urls_accept_root_relative_paths() {
        assert!(LinkUrl::new("/media/logo.png").is_ok());
        assert!(LinkUrl::new("/").is_ok());
        assert!(LinkUrl::new("/media/logo.png").unwrap().is_relative());
        assert!(!LinkUrl::new("https://a.com").unwrap().is_relative());
        assert_eq!(LinkUrl::new("//x"), Err(UrlError::Scheme));
        assert_eq!(LinkUrl::new("//evil.com/a"), Err(UrlError::Scheme));
        assert_eq!(
            LinkUrl::new("/\\evil.com"),
            Err(UrlError::ForbiddenChar('\\'))
        );
        assert_eq!(LinkUrl::new("media/a.png"), Err(UrlError::Scheme));
        assert_eq!(LinkUrl::new(""), Err(UrlError::Scheme));
    }

    #[test]
    fn urls_have_a_length_limit() {
        let long = format!("https://a.com/{}", "a".repeat(2048));
        assert_eq!(AbsoluteUrl::new(long), Err(UrlError::TooLong));
    }

    #[test]
    fn links_check_label_and_url() {
        let link = Link::new("  Status ", "https://status.harbor.example.com").unwrap();
        assert_eq!(link.label, "Status");
        assert_eq!(
            Link::new(" ", "/a"),
            Err(LinkError::Label(TextError::Blank))
        );
        assert_eq!(
            Link::new(&"a".repeat(41), "/a"),
            Err(LinkError::Label(TextError::TooManyChars { max: 40 }))
        );
        assert!(
            serde_json::from_str::<Link>(r#"{"label":"x","url":"javascript:alert(1)"}"#).is_err()
        );
        assert!(serde_json::from_str::<Link>(r#"{"label":"x","url":"/a","extra":1}"#).is_err());
    }
}
