//! The announcement bar at the top of every site page.

use serde::{Deserialize, Serialize};

use crate::{
    site::link::LinkUrl,
    text::{self, TextError},
};

text_enum! {
    /// The colour of the announcement bar.
    pub enum Tone / ToneError ("tone") {
        #[default]
        Info = "info",
        Accent = "accent",
        Warning = "warning",
    }
}

text_enum! {
    /// The state of an announcement at a given time.
    pub enum AnnouncementStatus / AnnouncementStatusError ("announcement status") {
        /// The start time is in the future.
        Scheduled = "scheduled",
        /// The site shows the bar.
        #[default]
        Live = "live",
        /// The end time is in the past.
        Ended = "ended",
    }
}

/// A short message that the site shows above every page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(try_from = "RawAnnouncement")]
pub struct Announcement {
    /// Plain text, trimmed, 1 to 160 characters.
    pub text: String,
    pub link: Option<LinkUrl>,
    /// The text of the link, trimmed, at most 40 characters.
    pub link_label: String,
    pub tone: Tone,
    /// The start time in unix seconds. `None` means that the bar starts now.
    #[cfg_attr(feature = "ts", ts(type = "number | null"))]
    pub starts_at: Option<i64>,
    /// The end time in unix seconds. `None` means that the bar has no end.
    #[cfg_attr(feature = "ts", ts(type = "number | null"))]
    pub ends_at: Option<i64>,
    /// Lets readers close the bar.
    pub dismissible: bool,
}

/// An announcement that breaks a validation rule.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AnnouncementError {
    /// The text breaks the text rules.
    #[error("text: {0}")]
    Text(TextError),
    /// The link label breaks the text rules.
    #[error("link_label: {0}")]
    LinkLabel(TextError),
    /// The end time is not after the start time.
    #[error("ends_at: the end time must be after the start time")]
    Bounds,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAnnouncement {
    text: String,
    #[serde(default)]
    link: Option<LinkUrl>,
    #[serde(default)]
    link_label: Option<String>,
    #[serde(default)]
    tone: Tone,
    #[serde(default)]
    starts_at: Option<i64>,
    #[serde(default)]
    ends_at: Option<i64>,
    #[serde(default = "dismissible_default")]
    dismissible: bool,
}

const TEXT_MAX: usize = 160;
const LABEL_MAX: usize = 40;
/// The link label when the input has none.
pub const DEFAULT_LINK_LABEL: &str = "Learn more";

fn dismissible_default() -> bool {
    true
}

impl Announcement {
    /// Returns `true` when `now` is in `[starts_at, ends_at)`. A missing bound is open.
    #[must_use]
    pub fn is_live(&self, now: i64) -> bool {
        self.status(now) == AnnouncementStatus::Live
    }

    /// Returns the state of the announcement at `now`.
    #[must_use]
    pub fn status(&self, now: i64) -> AnnouncementStatus {
        if self.starts_at.is_some_and(|start| now < start) {
            return AnnouncementStatus::Scheduled;
        }
        if self.ends_at.is_some_and(|end| now >= end) {
            return AnnouncementStatus::Ended;
        }
        AnnouncementStatus::Live
    }

    /// Returns the first start or end time after `now`.
    #[must_use]
    pub fn next_change(&self, now: i64) -> Option<i64> {
        [self.starts_at, self.ends_at]
            .into_iter()
            .flatten()
            .filter(|time| *time > now)
            .min()
    }
}

impl TryFrom<RawAnnouncement> for Announcement {
    type Error = AnnouncementError;

    fn try_from(raw: RawAnnouncement) -> Result<Self, Self::Error> {
        let text = text::required(&raw.text, TEXT_MAX).map_err(AnnouncementError::Text)?;
        if let Some(code) = text.chars().find(char::is_ascii_control) {
            return Err(AnnouncementError::Text(TextError::ControlChar {
                code: u32::from(code),
            }));
        }
        let link_label = label(raw.link_label.as_deref())?;
        if let (Some(start), Some(end)) = (raw.starts_at, raw.ends_at)
            && end <= start
        {
            return Err(AnnouncementError::Bounds);
        }
        Ok(Self {
            text,
            link: raw.link,
            link_label,
            tone: raw.tone,
            starts_at: raw.starts_at,
            ends_at: raw.ends_at,
            dismissible: raw.dismissible,
        })
    }
}

fn label(value: Option<&str>) -> Result<String, AnnouncementError> {
    match value.map(str::trim) {
        None | Some("") => Ok(DEFAULT_LINK_LABEL.to_owned()),
        Some(value) => text::required(value, LABEL_MAX)
            .and_then(|value| text::check_controls(&value).map(|()| value))
            .map_err(AnnouncementError::LinkLabel),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{Announcement, AnnouncementStatus, Tone};

    fn parse(value: Value) -> Result<Announcement, serde_json::Error> {
        serde_json::from_value(value)
    }

    fn window(starts_at: Option<i64>, ends_at: Option<i64>) -> Announcement {
        parse(json!({"text": "Backups moved", "starts_at": starts_at, "ends_at": ends_at})).unwrap()
    }

    #[test]
    fn defaults_and_trimming() {
        let bar = parse(json!({"text": "  How we back up SQLite  "})).unwrap();
        assert_eq!(bar.text, "How we back up SQLite");
        assert_eq!(bar.link_label, "Learn more");
        assert_eq!(bar.tone, Tone::Info);
        assert!(bar.dismissible);
        assert_eq!(bar.link, None);
    }

    #[test]
    fn text_and_label_rules() {
        assert!(parse(json!({"text": "  "})).is_err());
        assert!(parse(json!({"text": "a".repeat(161)})).is_err());
        assert!(parse(json!({"text": "a".repeat(160)})).is_ok());
        assert!(parse(json!({"text": "a\nb"})).is_err());
        assert!(parse(json!({"text": "a", "link_label": "b".repeat(41)})).is_err());
        assert!(parse(json!({"text": "a", "link": "javascript:x"})).is_err());
        assert!(parse(json!({"text": "a", "extra": 1})).is_err());
        let bar = parse(json!({"text": "a", "link_label": " Read "})).unwrap();
        assert_eq!(bar.link_label, "Read");
    }

    #[test]
    fn end_must_follow_start() {
        assert!(parse(json!({"text": "a", "starts_at": 10, "ends_at": 10})).is_err());
        assert!(parse(json!({"text": "a", "starts_at": 10, "ends_at": 5})).is_err());
        assert!(parse(json!({"text": "a", "starts_at": 10, "ends_at": 11})).is_ok());
    }

    #[test]
    fn status_follows_the_window() {
        let bar = window(Some(10), Some(20));
        assert_eq!(bar.status(9), AnnouncementStatus::Scheduled);
        assert_eq!(bar.status(10), AnnouncementStatus::Live);
        assert_eq!(bar.status(19), AnnouncementStatus::Live);
        assert_eq!(bar.status(20), AnnouncementStatus::Ended);
        assert!(!bar.is_live(9));
        assert!(bar.is_live(10));
        assert!(!bar.is_live(20));
        assert!(window(None, None).is_live(i64::MIN));
        assert!(window(None, Some(5)).is_live(4));
        assert!(window(Some(5), None).is_live(i64::MAX));
    }

    #[test]
    fn next_change_is_the_next_bound() {
        let bar = window(Some(10), Some(20));
        assert_eq!(bar.next_change(5), Some(10));
        assert_eq!(bar.next_change(10), Some(20));
        assert_eq!(bar.next_change(20), None);
        assert_eq!(window(None, None).next_change(0), None);
    }
}
