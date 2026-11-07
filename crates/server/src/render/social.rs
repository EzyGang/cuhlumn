//! The social profile links of a site, with one icon for each network.
//!
//! The brand icons are paths from Simple Icons (CC0).
//! The LinkedIn and email icons are original drawings.

use cuhlumn_api::site::social::{Network, SocialLink};
use maud::{Markup, PreEscaped, html};

/// Returns the list of social links, or nothing when the site has none.
#[must_use]
pub fn links(social: &[SocialLink]) -> Markup {
    if social.is_empty() {
        return html! {};
    }
    html! {
        ul.social-links aria-label="Social links" { @for link in social {
            li { a href=(link.url) rel="me noopener" aria-label=(name(link.network)) { (PreEscaped(icon(link.network))) } }
        } }
    }
}

/// Returns the profile addresses for the JSON-LD `sameAs` list. Email addresses are not profiles.
#[must_use]
pub fn profiles(social: &[SocialLink]) -> Vec<&str> {
    social
        .iter()
        .filter(|link| link.network != Network::Email)
        .map(|link| link.url.as_str())
        .collect()
}

/// Returns the name of a network, as people write it.
#[must_use]
pub const fn name(network: Network) -> &'static str {
    match network {
        Network::Github => "GitHub",
        Network::Gitlab => "GitLab",
        Network::Codeberg => "Codeberg",
        Network::Mastodon => "Mastodon",
        Network::Bluesky => "Bluesky",
        Network::X => "X",
        Network::Threads => "Threads",
        Network::Linkedin => "LinkedIn",
        Network::Youtube => "YouTube",
        Network::Instagram => "Instagram",
        Network::Discord => "Discord",
        Network::Telegram => "Telegram",
        Network::Email => "Email",
    }
}

/// Returns the inline SVG icon of a network. It is hidden from assistive technology.
const fn icon(network: Network) -> &'static str {
    match network {
        Network::Github => include_str!("../../assets/social/github.svg"),
        Network::Gitlab => include_str!("../../assets/social/gitlab.svg"),
        Network::Codeberg => include_str!("../../assets/social/codeberg.svg"),
        Network::Mastodon => include_str!("../../assets/social/mastodon.svg"),
        Network::Bluesky => include_str!("../../assets/social/bluesky.svg"),
        Network::X => include_str!("../../assets/social/x.svg"),
        Network::Threads => include_str!("../../assets/social/threads.svg"),
        Network::Linkedin => include_str!("../../assets/social/linkedin.svg"),
        Network::Youtube => include_str!("../../assets/social/youtube.svg"),
        Network::Instagram => include_str!("../../assets/social/instagram.svg"),
        Network::Discord => include_str!("../../assets/social/discord.svg"),
        Network::Telegram => include_str!("../../assets/social/telegram.svg"),
        Network::Email => include_str!("../../assets/social/email.svg"),
    }
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::site::social::{Network, SocialLink};

    use crate::render::social::{icon, links, profiles};

    #[test]
    fn every_icon_is_a_hidden_svg() {
        for network in Network::ALL {
            let svg = icon(*network);
            assert!(
                svg.starts_with(r#"<svg viewBox="0 0 24 24" aria-hidden="true""#),
                "{network}"
            );
            assert!(svg.ends_with("</svg>"), "{network}");
        }
    }

    #[test]
    fn links_have_names_and_rel_me() {
        let social = [
            SocialLink::new(Network::Github, "https://github.com/maralind").unwrap(),
            SocialLink::new(Network::Email, "mailto:mara@harbor.example.com").unwrap(),
        ];
        let html = links(&social).into_string();
        assert!(html.contains(
            r#"<a href="https://github.com/maralind" rel="me noopener" aria-label="GitHub">"#
        ));
        assert!(html.contains(r#"aria-label="Email""#));
        assert_eq!(profiles(&social), ["https://github.com/maralind"]);
        assert!(links(&[]).into_string().is_empty());
    }
}
