//! Links to the profiles of the site owner on other networks.

use serde::{Deserialize, Serialize};

use crate::site::link::AbsoluteUrl;

text_enum! {
    /// A network that the site can link to.
    pub enum Network / NetworkError ("network") {
        #[default]
        Github = "github",
        Gitlab = "gitlab",
        Codeberg = "codeberg",
        Mastodon = "mastodon",
        Bluesky = "bluesky",
        X = "x",
        Threads = "threads",
        Linkedin = "linkedin",
        Youtube = "youtube",
        Instagram = "instagram",
        Discord = "discord",
        Telegram = "telegram",
        Email = "email",
    }
}

/// A link to a profile, shown as an icon.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(try_from = "RawSocialLink")]
pub struct SocialLink {
    pub network: Network,
    /// An `https://` address, or a `mailto:` address with one recipient for [`Network::Email`].
    pub url: String,
}

/// A social link with an address that is not permitted.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SocialLinkError {
    /// The address is longer than 500 characters.
    #[error("the address is longer than 500 characters")]
    TooLong,
    /// The address is not an absolute `https://` address.
    #[error("the address must be an absolute https:// address")]
    Https,
    /// The address is not a `mailto:` address with one recipient.
    #[error("the address must be a mailto: address with one recipient")]
    Mailto,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSocialLink {
    network: Network,
    url: String,
}

const URL_MAX: usize = 500;

impl SocialLink {
    /// Validates a social link.
    ///
    /// # Errors
    ///
    /// Returns [`SocialLinkError`] when the address does not match the network rules.
    pub fn new(network: Network, url: impl Into<String>) -> Result<Self, SocialLinkError> {
        let url = url.into();
        if url.chars().count() > URL_MAX {
            return Err(SocialLinkError::TooLong);
        }
        match network {
            Network::Email => check_mailto(&url)?,
            _ => check_https(&url)?,
        }
        Ok(Self { network, url })
    }
}

impl TryFrom<RawSocialLink> for SocialLink {
    type Error = SocialLinkError;

    fn try_from(raw: RawSocialLink) -> Result<Self, Self::Error> {
        Self::new(raw.network, raw.url)
    }
}

fn check_https(url: &str) -> Result<(), SocialLinkError> {
    if !url.starts_with("https://") || AbsoluteUrl::new(url).is_err() {
        return Err(SocialLinkError::Https);
    }
    Ok(())
}

fn check_mailto(url: &str) -> Result<(), SocialLinkError> {
    let address = url.strip_prefix("mailto:").ok_or(SocialLinkError::Mailto)?;
    let (local, domain) = address.split_once('@').ok_or(SocialLinkError::Mailto)?;
    let forbidden = |c: char| {
        c.is_whitespace()
            || c.is_control()
            || matches!(c, ',' | ';' | '?' | '"' | '\'' | '<' | '>' | '\\' | '@')
    };
    let valid = !local.is_empty()
        && !domain.is_empty()
        && !local.contains(forbidden)
        && !domain.contains(forbidden);
    if !valid {
        return Err(SocialLinkError::Mailto);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Network, SocialLink, SocialLinkError};

    #[test]
    fn web_networks_need_https() {
        assert!(SocialLink::new(Network::Github, "https://github.com/maralind").is_ok());
        assert_eq!(
            SocialLink::new(Network::Github, "http://github.com/maralind"),
            Err(SocialLinkError::Https)
        );
        assert_eq!(
            SocialLink::new(Network::Mastodon, "https://"),
            Err(SocialLinkError::Https)
        );
        assert_eq!(
            SocialLink::new(Network::X, "mailto:mara@harbor.example.com"),
            Err(SocialLinkError::Https)
        );
    }

    #[test]
    fn email_needs_one_mailto_address() {
        assert!(SocialLink::new(Network::Email, "mailto:mara@harbor.example.com").is_ok());
        for bad in [
            "https://harbor.example.com",
            "mailto:",
            "mailto:mara",
            "mailto:a@b.com,c@d.com",
            "mailto:a@b.com?cc=c@d.com",
            "mailto:a@b@c.com",
        ] {
            assert_eq!(
                SocialLink::new(Network::Email, bad),
                Err(SocialLinkError::Mailto),
                "{bad}"
            );
        }
    }

    #[test]
    fn urls_have_a_length_limit() {
        let long = format!("https://a.com/{}", "a".repeat(500));
        assert_eq!(
            SocialLink::new(Network::Github, long),
            Err(SocialLinkError::TooLong)
        );
    }

    #[test]
    fn deserialize_validates() {
        let ok = r#"{"network":"bluesky","url":"https://bsky.app/profile/mara"}"#;
        assert!(serde_json::from_str::<SocialLink>(ok).is_ok());
        let bad = r#"{"network":"bluesky","url":"javascript:alert(1)"}"#;
        assert!(serde_json::from_str::<SocialLink>(bad).is_err());
        let extra = r#"{"network":"x","url":"https://x.com/m","extra":1}"#;
        assert!(serde_json::from_str::<SocialLink>(extra).is_err());
    }
}
