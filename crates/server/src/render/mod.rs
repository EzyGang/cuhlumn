//! Pages, feeds, and the files that search engines and AI assistants read, made from a site and its posts.

pub mod article;
pub mod blog;
pub mod files;
pub mod home;
pub mod layout;
pub mod markdown;
pub mod meta;
pub mod not_found;
pub mod schema;
pub mod style;
pub mod tag;

use chrono::{DateTime, SecondsFormat};

/// Returns a unix time as an RFC 3339 UTC time, such as `2026-10-27T09:30:00Z`.
#[must_use]
pub fn iso(time: i64) -> String {
    DateTime::from_timestamp(time, 0)
        .map(|date| date.to_rfc3339_opts(SecondsFormat::Secs, true))
        .unwrap_or_default()
}

/// Returns a unix time as a UTC day, such as `October 27, 2026`.
#[must_use]
pub fn day(time: i64) -> String {
    DateTime::from_timestamp(time, 0)
        .map(|date| date.format("%B %-d, %Y").to_string())
        .unwrap_or_default()
}

/// Escapes text for XML and HTML content and double-quoted attributes.
///
/// Characters outside the XML 1.0 `Char` set, such as U+0001 or U+FFFE, are dropped.
#[must_use]
pub fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\t'
            | '\n'
            | '\r'
            | '\u{20}'..='\u{d7ff}'
            | '\u{e000}'..='\u{fffd}'
            | '\u{10000}'.. => {
                escaped.push(character);
            }
            _ => (),
        }
    }
    escaped
}

/// Returns the first letter or digit of a title in uppercase, for logo badges.
#[must_use]
pub fn initial(title: &str) -> String {
    title
        .chars()
        .find(|character| character.is_alphanumeric())
        .map(|character| character.to_uppercase().collect())
        .unwrap_or_default()
}

/// Shortens text to at most `limit` characters at a word boundary and adds an ellipsis.
#[must_use]
pub fn clip(text: &str, limit: usize) -> String {
    let Some((end, _)) = text.char_indices().nth(limit) else {
        return text.to_owned();
    };
    let head = &text[..end];
    let head = head.rfind(' ').map_or(head, |space| &head[..space]);
    let mut clipped = head
        .trim_end_matches(|character: char| !character.is_alphanumeric())
        .to_owned();
    clipped.push('…');
    clipped
}

#[cfg(test)]
mod tests {
    use crate::render::{clip, day, escape, initial, iso};

    #[test]
    fn dates_use_utc() {
        assert_eq!(iso(1_793_093_400), "2026-10-27T09:30:00Z");
        assert_eq!(day(1_793_093_400), "October 27, 2026");
    }

    #[test]
    fn clip_stops_at_a_word() {
        assert_eq!(clip("short", 10), "short");
        assert_eq!(clip("one two, three", 9), "one two…");
        assert_eq!(clip("héllo wörld again", 12), "héllo wörld…");
    }

    #[test]
    fn helpers_escape_and_find_initials() {
        assert_eq!(
            escape("<a href=\"x\">&</a>"),
            "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;"
        );
        assert_eq!(
            escape("a\u{1}b\u{b}c\u{1b}d\u{fffe}e\u{ffff}f\t\n\r😀"),
            "abcdef\t\n\r😀"
        );
        assert_eq!(initial("  émile notes"), "É");
        assert_eq!(initial("!!!"), "");
    }
}
