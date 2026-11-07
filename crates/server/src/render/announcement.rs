//! The announcement bar at the top of every page while the announcement is live.

use cuhlumn_api::site::announcement::Announcement;
use maud::{Markup, PreEscaped, html};
use sha2::{Digest, Sha256};

const CLOSE_SVG: &str = include_str!("../../assets/close.svg");
const CLOSE_JS: &str = include_str!("../../assets/announcement.js");

/// The parts of a live announcement that a page shows.
pub struct Bar<'a> {
    announcement: &'a Announcement,
    key: String,
}

impl<'a> Bar<'a> {
    /// Returns the bar when the announcement is live at `now`.
    #[must_use]
    pub fn live(announcement: Option<&'a Announcement>, now: i64) -> Option<Self> {
        let announcement = announcement.filter(|bar| bar.is_live(now))?;
        Some(Self {
            key: key(announcement),
            announcement,
        })
    }

    /// Returns the `<head>` script that hides a dismissed bar before the first paint.
    #[must_use]
    pub fn head(&self) -> Markup {
        if !self.announcement.dismissible {
            return html! {};
        }
        let script = format!(
            r#"try{{if(localStorage.getItem("announcement:{}"))document.documentElement.setAttribute("data-announcement-dismissed","")}}catch(e){{}}"#,
            self.key
        );
        html! { script { (PreEscaped(script)) } }
    }

    /// Returns the bar. It goes first in `<body>`.
    #[must_use]
    pub fn body(&self) -> Markup {
        let bar = self.announcement;
        html! {
            div class={ "announcement announcement-" (bar.tone) } role="region" aria-label="Announcement" data-announcement=(self.key) {
                p.announcement-text {
                    (bar.text)
                    @if let Some(link) = &bar.link {
                        " " a href=(link) { (bar.link_label) span.announcement-context { " about this announcement" } }
                    }
                }
                @if bar.dismissible {
                    button.announcement-close type="button" aria-label="Dismiss the announcement" { (PreEscaped(CLOSE_SVG)) }
                }
            }
        }
    }

    /// Returns the script of the close button, or nothing when the bar cannot be dismissed.
    #[must_use]
    pub fn script(&self) -> Markup {
        if !self.announcement.dismissible {
            return html! {};
        }
        html! { script { (PreEscaped(CLOSE_JS)) } }
    }
}

/// Returns the dismissal key: the first 8 hex digits of SHA-256 of the text, a line feed, and the link.
///
/// A new text or link gets a new key, so readers see the changed bar again.
#[must_use]
pub fn key(announcement: &Announcement) -> String {
    let mut hasher = Sha256::new();
    hasher.update(announcement.text.as_bytes());
    hasher.update(b"\n");
    if let Some(link) = &announcement.link {
        hasher.update(link.as_str().as_bytes());
    }
    hasher
        .finalize()
        .iter()
        .take(4)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::site::announcement::Announcement;
    use serde_json::{Value, json};

    use crate::render::announcement::{Bar, key};

    fn bar(value: Value) -> Announcement {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn key_is_the_start_of_the_sha256_of_text_and_link() {
        // SHA-256 of "Backups moved\n" and of "Backups moved\n/sqlite-backups/".
        let plain = bar(json!({"text": "Backups moved"}));
        assert_eq!(key(&plain), "e99123f8");
        let linked = bar(json!({"text": "Backups moved", "link": "/sqlite-backups/"}));
        assert_eq!(key(&linked), "4391f29f");
        let relabeled = bar(
            json!({"text": "Backups moved", "link": "/sqlite-backups/", "link_label": "Read", "tone": "warning"}),
        );
        assert_eq!(key(&relabeled), key(&linked));
    }

    #[test]
    fn bar_shows_only_while_live() {
        let window = bar(json!({"text": "Backups moved", "starts_at": 10, "ends_at": 20}));
        assert!(Bar::live(Some(&window), 9).is_none());
        assert!(Bar::live(Some(&window), 10).is_some());
        assert!(Bar::live(Some(&window), 20).is_none());
        assert!(Bar::live(None, 10).is_none());
    }

    #[test]
    fn fixed_bar_has_no_button_and_no_scripts() {
        let fixed = bar(json!({"text": "Backups moved", "dismissible": false}));
        let shown = Bar::live(Some(&fixed), 0).unwrap();
        assert!(!shown.body().into_string().contains("<button"));
        assert!(shown.head().into_string().is_empty());
        assert!(shown.script().into_string().is_empty());
    }
}
